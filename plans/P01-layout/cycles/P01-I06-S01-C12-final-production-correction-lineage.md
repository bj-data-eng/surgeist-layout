# P01-I06-S01-C12 Final Production Correction And Lineage

Status: in_progress

Cycle ID: `P01/I06/S01/C12`

Owning repository: `surgeist-layout`

Cycle base: `8ffb4bc551a24d2283ad54436870ab3f5e66a473`

Reviewed specification:
`plans/P01-layout/initiatives/P01-I06-inline-formatting-floats-bfcs.md`
at normalized SHA-256
`702fab3acac6c66b22333f5120212ab36e365c7b6a00734d70285c583fb3c212`,
commit `49ede2ba2672a91f99ba193651dbb1350ede7b80`: `FRI-06.4 D-01`,
`D-04`, `D-06`, `D-07`, `D-09`, `D-11`, `D-12`, `D-13`, and `D-16`;
line, metric-fragment, atomic-baseline, physical-placement, comparator,
fixture, and acceptance portions of `FRI-06.5`, `FRI-06.7`, `FRI-06.9`
through `FRI-06.11`, and `FRI-06.14`.

Reviewed implementation sequence:
`plans/P01-layout/sequences/P01-I06-S01-inline-formatting-floats-bfcs.md`
at normalized SHA-256
`b608fe1864b3bb34b4ef293055cc9d1015ec4fb6595295c7bd8f123d675c6b52`,
commit `0a666f8f698703cd7979194a7f75f834e4c9b522`, entry `P01/I06/S01/C12`.

## 1 Outcome

Replace C08's diagnostic name/expectation synthesis with the specification's
closed explicit fixture inputs, then correct the ten C08-characterized
production rows: two direct-root RTL percentage-block-size rows, four vertical
break/clear placement rows, and four float-line exclusion rows. Freeze the
reviewed inputs, run exactly one full unfiltered generation with the
already-installed pinned browser, and publish a lineage in which every 388-row
activation entry has browser-pass evidence or one reviewed Chrome-failure
substitute while the other 5,324 XML bodies preserve their entry semantics.

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

T02 owns only the eight exact HTML marker sources in `FRI-06.11`, helper and
narrow existing-generator serialization for the three closed XML forms, strict
parser support, and focused fixture-input tests. No task changes public API,
browser policy, launch profile, base style, Taffy source, dependency, feature,
MSRV, root integration, or later-owned behavior. Generator architecture
expansion is prohibited. Any generator change beyond those exact serializer
forms requires a focused test proving a genuine bug and returns the plan for
semantic revision and review before implementation.

Scoped generation remains an optional diagnostic during an implementation
iteration, never acceptance evidence. This cycle does not need it. T02 adds the
explicit path while retaining the exact old adapter only for stale committed XML.
After T02 through T04 are task-clean, T05 commits its final adapter removal and
input/check freeze, then runs one full unfiltered `generate-existing` invocation.
No task reruns generation over unchanged inputs. A failed process or unexpected
output returns the exact blocker and preserves the resulting evidence; it is not
retried. Any later authorized input change requires plan reconciliation before
one replacement full run.

## 3 Known Chrome Measurement Failures

None at this reviewed revision. Chrome remains authoritative. Adding an entry is
a material plan change requiring all exact `FRI-06.11` proof, substitute,
disposition, and revalidation evidence plus fresh plan review before
implementation or generation.

## 4 Impacts

- **Public API and compatibility:** unchanged.
- **Production:** narrow inline traversal, line-metric projection, and terminal
  auto-block corrections in existing internal paths.
- **Tests:** explicit-input honesty and malformed-input controls, exact ten-row
  GREEN transitions, and one final 388-row comparator and lineage/provenance.
- **Generated artifacts:** T05 alone replaces the 5,712 XML lineage and report;
  no scoped report is accepted.
- **Dependencies, features, docs, examples, MSRV, and root:** unchanged. This
  terminal cycle leaves the leaf inputs and outputs frozen; no later cycle
  receives them.
- **Safety:** Surgeist-owned code remains free of unsafe; no new `allow` or
  `expect` attribute is permitted.

## 5 Tasks

### 5.1 `P01/I06/S01/C12/T01` Correct Direct RTL Visual Traversal

**Files/area:** `src/inline.rs`; focused tests in `src/root_tests.rs`,
`src/grid_tests.rs`, and `tests/layout/browser_parity.rs`. The grid test change
may only correct the existing C07 RTL physical-origin expectation that encoded
the same redundant second mirror. No fixture, parser, helper, generator, or
generated output.

**Outcome:** Consume the visual order returned by bidi reordering exactly once.
Map logical inline starts through decreasing physical progression without a
direction-wide second reversal.

**RED:** Change the two exact direct-root RTL browser expectations first and
run the focused library and public-fixture-shaped tests. At the task base they
must fail with first Range start 180 instead of 73.296875, atomic x 102 instead
of 73, and trailing Range start 102.203125 instead of 180. Reconstruct the
test-only change against the exact task base if the existing expected-RED test
uses a panic wrapper; do not fabricate RED from the implementation diff.

**Acceptance:** Both `content-box` and `border-box` variants pass for f32 and
f64. First Range start is 73.296875, the atomic occupies the browser x/width,
and trailing Range start is 180 with all y, line, baseline, and root geometry
unchanged. Existing ten-flow mapping, mixed-level, source-index, and visual-index
controls pass. The C07 subgrid RTL physical origins follow the same single
visual-order projection. The direction-wide second reversal is absent.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib fri06_c08_recovery_characterization_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --test layout fri06_c08_recovery_characterization_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib fri06_c08_r1_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib fri06_c07_subgrid_rtl_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just fmt-check
```

**Dependency:** cycle plan is reviewed and in progress.

**Intended commit:** `fix(layout): consume RTL visual order once`.

### 5.2 `P01/I06/S01/C12/T02` Replace Synthetic Fixture Dispatch With Explicit Inputs

**Files/area:** `tests/layout/browser_parity/support.rs`,
`tests/layout/browser_parity/scripts/gentest/test_helper.js`, the narrow
serializer and focused tests in `tests/bin/surgeist-layout-generate/generator.rs`,
focused tests in `tests/layout/browser_parity.rs`, and only these HTML sources:
`subgrid/subgrid_baseline_auto_columns_{first,second}_item.html`,
`subgrid/subgrid_baseline_standalone_axis_{first,second}_item.html`,
`subgrid/subgrid_auto_track_sizing_min_content_text_runs.html`,
`block/{fri06_bidi_mixed_inline,fri06_inline_mixed_text_atomic_wrap}.html`, and
`float/fri06_float_line_exclusion.html`. No production or generated output.

**Outcome:** Validate the exact anonymous-wrapper, transparent-inline-container,
and containing-strut schema against Chrome's actual DOM, serialize those facts,
and strictly parse only the generated XML in `FRI-06.11`. The explicit path
parses normalized input and expectations independently. Retain
`apply_fri06_c08_finite_adapter` and its helpers unchanged and exact only to keep
stale committed XML executable until T05; no explicit-form test may enter that
compatibility path.

**Correction:** Remove the rejected Rust HTML source-preflight/tag/style/topology
parser while retaining authored markers, helper validation, closed XML
serialization/parsing, and input/expectation independence evidence.

**RED:** Add the focused `fri06_c08r_fixture_input_` equality and rejection tests
first. At the task base, the closed XML attributes/boundaries are unsupported,
helper marker validation is absent, renaming prevents synthesis, and a valid
expectation-only structural change can block adapter input lowering.

**Acceptance:** Helper tests reject invalid marker values, roles, metrics, and
actual-DOM topology. The three XML forms accept only the specified attributes,
placement, metrics, and payload absence. Renaming a test and arbitrary
expectation-only mutation produce identical normalized parsed input. Transparent
browser wrappers normalize input and expectation trees independently. Synthetic
serializer-to-parser fixtures cover all five former adapter source families from
explicit facts. No Rust code reconstructs HTML tags, attributes, styles, or DOM
topology; literal source checks are diagnostic only. Explicit lowering reads
neither fixture name nor expectations, while the old compatibility block and
hashes remain exact. No generation or `corpus-check` runs.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri06_c08r_fixture_input_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --test layout fri06_c08r_fixture_input_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just fmt-check
```

**Dependency:** T01 is task-clean.

**Intended commit:** `test(parity): serialize explicit C08R fixture inputs`.

### 5.3 `P01/I06/S01/C12/T03` Preserve Vertical Line-Band Phase

**Files/area:** line metric resolution and physical projection in
`src/inline.rs` and, only if the focused call path requires it, `src/block.rs`;
focused tests in `src/root_tests.rs` and `tests/layout/browser_parity.rs`. No
fixture, parser, helper, generator, or generated output.

**Outcome:** Resolve the forced-break strut and atomic fallback baseline within
the containing 24px vertical line bands, then project logical block positions
through `vertical-rl` without changing horizontal fallback behavior.

**RED:** Change all four exact vertical browser expectations first. At the task
base, both box models and both inline directions must fail at first atomic x 78
instead of 75, second atomic x 53 instead of 51, and cleared box x 28 instead
of 30 while retaining the characterized y and float geometry.

**Acceptance:** All four rows pass for f32 and f64 with first atomic x 75,
second atomic x 51, cleared box x 30, and the complete characterized browser
geometry. Metric fragments retain resolved 24px line height and 16.8 baseline;
D-11 atomic fallback, horizontal writing, top/bottom alignment, clear, and float
controls remain green. No writing-mode-specific coordinate patch or fixture-name
branch is introduced.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib fri06_c08_recovery_characterization_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --test layout fri06_c08_recovery_characterization_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --test layout fri06_c06_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just fmt-check
```

**Dependency:** T02 is task-clean.

**Intended commit:** `fix(layout): preserve vertical line band phase`.

### 5.4 `P01/I06/S01/C12/T04` Derive Float Continuation And Terminal Extent

**Files/area:** continuation-line placement in `src/inline.rs`, automatic block
size in `src/block.rs`, and focused tests in `src/root_tests.rs` and
`tests/layout/browser_parity.rs`. No fixture, parser, helper, generator, or
generated output.

**Outcome:** Carry the resolved line/strut phase through float exclusion and
derive terminal automatic block size from actual in-flow and float extents.
Remove the C08-owned content-type/rounding adjustment once geometry owns the
result.

**RED:** Change all four exact float-line browser expectations first. At the
task base they must expose root height 62 instead of 63, second-line y 24
instead of 21.2, third-line y 40 instead of 42, and LTR fourth-atomic x 42
instead of 90, with the complete characterized geometry retained.

**Acceptance:** Both box models and directions pass for f32 and f64. Browser
line phase is y 0, 21.2, 21.2, and 42; the fourth atomic has the expected
physical x in both directions; terminal unrounded extent is 62.5 and rounded
root height is 63. Pure-text, all-atomic, empty-inline, nonterminal-mixed, and
integral-height controls remain 62. No fixture-name, content-type, or integer-
rounding special case contributes the terminal 0.5px.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib fri06_c08_recovery_characterization_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --test layout fri06_c08_recovery_characterization_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib fri06_c08_float_line_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just fmt-check
```

**Dependency:** T03 is task-clean.

**Intended commit:** `fix(layout): derive terminal float line extent`.

### 5.5 `P01/I06/S01/C12/T05` Freeze Inputs And Derive One Final Lineage

**Files/area:** focused lineage/comparator tests in
`tests/bin/surgeist-layout-generate/generator.rs` and
`tests/layout/browser_parity.rs`, final compatibility removal in
`tests/layout/browser_parity/support.rs`; generated XML under
`tests/layout/browser_parity/xml/` and the authoritative generation report.
Production, HTML, helper, generated-XML parser behavior, manifest, launch
profile, base style, and generator behavior are frozen.

**Outcome:** Pin all generation inputs and the normalized 388-row activation
union, then remove the complete name/expectation compatibility adapter before
generation. Execute exactly one full unfiltered existing-pinned generation, then
use read-only tests to prove the complete final lineage.

**Evidence before the run:** Focused tests pin the exact task-clean T02 helper,
eight-source HTML inputs, generated-XML parser, serializer, and marker-accounting
contract;
corpus manifest
`99bb6fda5641c9f81704ddf391930934fb441f719090cf6ca4b84e31636c3701`,
base style
`5d00a3f3c55322b7002b065eacc6b4f3f14ecad83f757c79679b6ec6dee4fec6`,
launch profile
`9e2b5a4850e8d5ae31cf133c30f7129f1e214705f7a848697ca42c7c1b7551cb`,
browser `149.0.7827.115`, Taffy source
`d1ff7e339b9ee35b33858779f8d7653197e93d92`, and normalized census
`0630d2606f1e53c56b69cd226665b899bbfd96ed60ad7ac3c80ec5d9423b5691`.
They prove the filter variables are absent, explicit input and expectation
lowering are independent, all compatibility identifiers and calls are absent,
and each production row already browser-passes from frozen input or has the exact
reviewed substitute registry disposition. Commit all such test/parser changes
before the run; stale checked-in artifacts are the expected task-local RED and
no generated output is part of that preflight commit.

**Single generation command:**

```sh
env -u SURGEIST_BROWSER_CACHE -u SURGEIST_BROWSER_VERSION -u SURGEIST_LAYOUT_GENERATE_FILTER -u SURGEIST_LAYOUT_BROWSER_PARITY_ROOT CARGO_NET_OFFLINE=true SURGEIST_BROWSER_PATH='target/surgeist-browser/mac_arm-149.0.7827.115/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing' cargo run --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- generate-existing
```

This command runs once after the preflight commit. Do not run a scoped
generation before it, rerun it after it, or regenerate to repair a verification
failure. If it exits nonzero or produces unexpected output, stop with exact
status, output inventory, hashes, and worktree state.

**Acceptance after the run:** The authoritative report records `filter: null`,
5,712 generated variants, exactly 16 unsupported missing-root variants,
expected-fail inventory exactly equal to the reviewed plan registry (normally
zero), and zero quarantined, failed-to-generate, or other failure buckets. Exact
provenance matches the frozen inputs. Helper-reported source-local marker use
matches the exact eight-source inventory across all variants, with no missing,
extra, duplicate, misplaced, malformed, or elsewhere-used fact. Each of the 388
normalized activation rows is a browser pass or has its reviewed passing
synthetic substitute. The other 5,324 XML bodies preserve entry semantics;
provenance-only byte changes are allowed. No fixture/source-name dispatch or
expectation reader can select, create, or alter parsed layout input. No scoped
report or untracked generated output remains. Record the final report SHA-256,
5,712-file XML aggregate, 388-row comparison aggregate, and 5,324-body semantic
aggregate. `FRI-13` remains unclaimed.

**Commands after the run:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri06_c08r_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --test layout fri06_c08r_
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just fmt-check
```

**Dependency:** T01, T02, T03, and T04 are task-clean; T05 preflight inputs and tests
are committed and unchanged.

**Intended commits:** `test(parity): freeze C08R lineage inputs`, then
`test(parity): derive final C08R lineage`.

## 6 Completion

Before cycle completion, prove the single-generation command appears once in
execution evidence, generated inputs match the frozen preflight commit, and
every task acceptance command passes. Then run:

```sh
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
CARGO_NET_OFFLINE=true just fmt-check
git diff --check 8ffb4bc551a24d2283ad54436870ab3f5e66a473..HEAD
```

Build a nonempty manifest of every tracked and nonignored untracked Rust file.
Run the repository-wide executable-unsafe regex and the owning Clippy matrices
with `-F unsafe-code -D warnings`; classify every textual match. Scan every
task range for added `allow` and `expect` attributes. All gates must pass with a
clean worktree.

Apply the canonical task-review, cycle-review, landing, publication, remote
readback, closure, and handoff contracts after the cycle-specific evidence is
clean. The handoff records the immutable candidate, reviewed planning
revisions, task ranges, RED/GREEN evidence, final provenance and aggregate
hashes, single-generation evidence, all gates, safety evidence, and the
terminal frozen-input/output inventory. No later cycle is represented.

Blocker: none at planning time.
