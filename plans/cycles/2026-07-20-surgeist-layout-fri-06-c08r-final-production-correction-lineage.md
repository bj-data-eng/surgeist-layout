# FRI-06-C08R Final Production Correction And Lineage

Status: in_progress

Cycle ID: `FRI-06-C08R`

Owning repository: `surgeist-layout`

Cycle base: `8ffb4bc551a24d2283ad54436870ab3f5e66a473`

Reviewed specification:
`plans/specs/2026-07-17-surgeist-layout-fri-06-inline-formatting-floats-bfcs.md`
at normalized SHA-256
`5e89a8a81e5a5a62b38374d56d8dd89b7025e02efc65dfd73e33a887bcb3b87e`,
commit `213ac89f140465691e72d1569171a94346f5e42c`: `FRI-06.4 D-01`,
`D-04`, `D-06`, `D-07`, `D-09`, `D-11`, `D-12`, `D-13`, and `D-16`;
line, metric-fragment, atomic-baseline, physical-placement, comparator,
fixture, and acceptance portions of `FRI-06.5`, `FRI-06.7`, `FRI-06.9`
through `FRI-06.11`, and `FRI-06.14`.

Reviewed implementation sequence:
`plans/sequences/2026-07-17-surgeist-layout-fri-06-inline-formatting-floats-bfcs.md`
at normalized SHA-256
`6e07641d9d0aff921e15f8c94b8df729e7ca99ccb6c991c3d3524ead0e1edbe7`,
commit `cb7052923ca9d791e45af59b66f80b38cfddcdb8`, entry `FRI-06-C08R`.

## Outcome

Correct only the ten C08-characterized production rows: two direct-root RTL
percentage-block-size rows, four vertical break/clear placement rows, and four
float-line exclusion rows. Freeze the reviewed inputs, run exactly one full
unfiltered generation with the already-installed pinned browser, and publish a
lineage in which all 388 activated public rows pass while the other 5,324 XML
bodies preserve their entry semantics.

## Boundary And Evidence

C08 is complete and remotely verified at the cycle base. Its task-clean public
fixture-shaped characterization records every current and browser coordinate,
line band, baseline, clear effect, float endpoint, and root size for the ten
rows. The production-browser assertion
`fri06_c08_recovery_characterization_browser_geometry_remains_unaccepted_production`
is exact RED; the companion current-geometry assertion prevents an incomplete
diagnosis from being mistaken for acceptance.

The direct RTL defect is a redundant physical-direction reversal in
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

No task changes public API, fixture HTML, parser input, helper serialization,
manifest, browser policy, launch profile, base style, Taffy source, dependency,
feature, MSRV, root integration, or later-owned behavior. Generator
architecture expansion is prohibited. A generator change is allowed only if a
focused test proves a genuine generator bug blocks the frozen contract; that
finding returns the plan for semantic revision and review before implementation.

Scoped generation remains an optional diagnostic during an implementation
iteration, never acceptance evidence. This cycle does not need it. After T1
through T3 and every T4 input/check change are task-clean and frozen, T4 runs
one full unfiltered `generate-existing` invocation. No task reruns generation
over unchanged inputs. A failed process or unexpected output returns the exact
blocker and preserves the resulting evidence; it is not retried. Any later
authorized input change requires plan reconciliation before one replacement
full run.

## Impacts

- **Public API and compatibility:** unchanged.
- **Production:** narrow inline traversal, line-metric projection, and terminal
  auto-block corrections in existing internal paths.
- **Tests:** exact ten-row GREEN transitions plus focused controls; one final
  388-row generated comparator and lineage/provenance checks.
- **Generated artifacts:** T4 alone replaces the 5,712 XML lineage and report;
  no scoped report is accepted.
- **Dependencies, features, docs, examples, MSRV, and root:** unchanged; C09
  receives the frozen leaf artifacts.
- **Safety:** Surgeist-owned code remains free of unsafe; no new `allow` or
  `expect` attribute is permitted.

## Tasks

### `C08R-T1` Correct Direct RTL Visual Traversal

**Files/area:** `src/inline.rs`; focused tests in `src/root_tests.rs` and
`tests/layout/browser_parity.rs`. No fixture, parser, helper, generator, or
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
controls pass. The direction-wide second reversal is absent.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib fri06_c08_recovery_characterization_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --test layout fri06_c08_recovery_characterization_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib fri06_c08_r1_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just fmt-check
```

**Dependency:** cycle plan is reviewed and in progress.

**Intended commit:** `fix(layout): consume RTL visual order once`.

### `C08R-T2` Preserve Vertical Line-Band Phase

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
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib fri06_c06_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just fmt-check
```

**Dependency:** T1 is task-clean.

**Intended commit:** `fix(layout): preserve vertical line band phase`.

### `C08R-T3` Derive Float Continuation And Terminal Extent

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

**Dependency:** T2 is task-clean.

**Intended commit:** `fix(layout): derive terminal float line extent`.

### `C08R-T4` Freeze Inputs And Derive One Final Lineage

**Files/area:** focused lineage/comparator tests in
`tests/bin/surgeist-layout-generate/generator.rs` and
`tests/layout/browser_parity.rs`; generated XML under
`tests/layout/browser_parity/xml/` and the authoritative generation report.
Production, HTML, helper, parser, manifest, launch profile, base style, and
generator behavior are frozen.

**Outcome:** Pin all generation inputs and the normalized 388-row activation
union before generation. Execute exactly one full unfiltered existing-pinned
generation, then use read-only tests to prove the complete final lineage.

**Evidence before the run:** Focused tests pin helper SHA-256
`d4bc9ec937f5de860f737ff7d886384a861a52d7004b39551e13852a1378acdc`,
corpus manifest
`99bb6fda5641c9f81704ddf391930934fb441f719090cf6ca4b84e31636c3701`,
base style
`5d00a3f3c55322b7002b065eacc6b4f3f14ecad83f757c79679b6ec6dee4fec6`,
launch profile
`9e2b5a4850e8d5ae31cf133c30f7129f1e214705f7a848697ca42c7c1b7551cb`,
browser `149.0.7827.115`, Taffy source
`d1ff7e339b9ee35b33858779f8d7653197e93d92`, and normalized census
`0630d2606f1e53c56b69cd226665b899bbfd96ed60ad7ac3c80ec5d9423b5691`.
They prove the filter variables are absent and the ten production expectations
already pass from frozen serialized inputs. Commit all such test changes before
the run; no generated output is part of that preflight commit.

**Single generation command:**

```sh
env -u SURGEIST_BROWSER_CACHE -u SURGEIST_BROWSER_VERSION -u SURGEIST_LAYOUT_GENERATE_FILTER -u SURGEIST_LAYOUT_BROWSER_PARITY_ROOT CARGO_NET_OFFLINE=true SURGEIST_BROWSER_PATH='target/surgeist-browser/mac_arm-149.0.7827.115/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing' cargo run --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- generate-existing
```

This command runs once after the preflight commit. Do not run a scoped
generation before it, rerun it after it, or regenerate to repair a verification
failure. If it exits nonzero or produces unexpected output, stop with exact
status, output inventory, hashes, and worktree state.

**Acceptance after the run:** The authoritative report records `filter: null`,
5,712 generated variants, exactly 16 unsupported missing-root variants, and zero
expected-fail, quarantined, failed-to-generate, or other failure buckets. Exact
provenance matches the frozen inputs. All 388 normalized activation rows compare
against their corresponding generated XML and pass. The other 5,324 XML bodies
preserve entry semantics; provenance-only byte changes are allowed. No scoped
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

**Dependency:** T1 through T3 are task-clean; T4 preflight inputs and tests are
committed and unchanged.

**Intended commits:** `test(parity): freeze C08R lineage inputs`, then
`test(parity): derive final C08R lineage`.

## Completion

T1 through T4 each receive a fresh task review over their exact ordered task
range. Before status completion, prove the single-generation command appears
once in execution evidence, generated inputs match the frozen preflight commit,
and every task acceptance command passes. Make a separate status-only
`complete` commit, then run:

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

Obtain a fresh `surgeist-holistic-reviewer` CLEAN verdict over
`8ffb4bc551a24d2283ad54436870ab3f5e66a473..cycle_head`. Rerun the complete
commands on local `main`, publish the immutable candidate to authority `main`
with a proven fast-forward lease, fetch/read back, and prove local `main`, its
tracking ref, `FETCH_HEAD`, and live remote `main` agree. Hand C09 the immutable
candidate, exact reviewed planning revisions, task ranges and reviews, RED/GREEN
evidence, final provenance and aggregate hashes, single-generation evidence,
all gates, safety evidence, and frozen-input/output inventory.

Blocker: none at planning time.
