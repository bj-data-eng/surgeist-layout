# FRI-06-C08 Fixture Input Recovery And Characterization

Status: in_progress

Cycle ID: `FRI-06-C08`

Owning repository: `surgeist-layout`

Cycle base: `bcdba3c49be09ad119c03ecdc4c77da803159132`

Reviewed specification:
`plans/specs/2026-07-17-surgeist-layout-fri-06-inline-formatting-floats-bfcs.md`
at normalized SHA-256
`5e89a8a81e5a5a62b38374d56d8dd89b7025e02efc65dfd73e33a887bcb3b87e`,
commit `213ac89f140465691e72d1569171a94346f5e42c`: `FRI-06.4 D-01`,
`D-04`, `D-09` through `D-11`, and `D-16`; metric-fragment,
atomic-baseline, physical-placement, comparator, fixture, and acceptance
portions of `FRI-06.5`, `FRI-06.7`, `FRI-06.9` through `FRI-06.11`, and
`FRI-06.14`.

Reviewed implementation sequence:
`plans/sequences/2026-07-17-surgeist-layout-fri-06-inline-formatting-floats-bfcs.md`
at normalized SHA-256
`6e07641d9d0aff921e15f8c94b8df729e7ca99ccb6c991c3d3524ead0e1edbe7`,
commit `cb7052923ca9d791e45af59b66f80b38cfddcdb8`, entry `FRI-06-C08`.

This plan supersedes
`plans/cycles/2026-07-19-surgeist-layout-fri-06-c08-bounded-fixture-activation-final-lineage.md`
at normalized SHA-256
`9fddc9b488f071910401c038a41bb69018e85e24763efc255a6a7281b4eac3a5`.
Its task-clean T1, T2, R0, R1, and R2 commits remain implementation context;
their composed effects and all new tasks remain inside the original cycle range.

## Outcome

Discard the second diagnostic output, correct the exact 274 nonproduction rows,
separate preflight-only evidence from post-lineage checks, and publish complete
public-fixture-shaped characterization for the ten rows owned by C08R. C08 runs
no generator and hands C08R task-clean, hashed fixture inputs plus exact RED
contracts for two direct RTL, four vertical-placement, and four float-height rows.

## Boundary And Evidence

The retained second-lineage census is
`plans/2026-07-20-surgeist-layout-fri-06-c08-second-lineage-census.md`, SHA-256
`a56b09ed4d68ee901dbc385db3d78b66bf5faeb82f844f1d531c94aef10a23b9`,
commit `cb7052923ca9d791e45af59b66f80b38cfddcdb8`. It records one valid full
diagnostic invocation, report SHA-256
`81b29941e7925aa471bd7a96091fc35b4a0eb5ff389cea532c5f7086fce476bb`,
5,712-file XML aggregate
`c1568fca84a70956c73f8e19de18977113aa0c1074dee7dc6887dd2171adb877`,
5,324 preserved bodies, and 104 pass / 284 fail.

The 284 rows partition into 244 blockified-BR inputs, 18 explicit-root Range
translations, two direct-root RTL placements, four Range line identities, four
shape breaks, four continuation struts, four vertical placements, and four
float-line heights. C08 corrects the 274 nonproduction rows and characterizes,
but does not fix, the final ten production rows. No row is later-owned.

Before the first implementation write, a fresh worker validates the census
hashes, restores the two uncommitted T3 Rust drafts and every tracked XML/report
path to current HEAD, and removes only the 388 report-enumerated untracked XML
outputs. It then proves the report is restored to
`4f18b4299765d7f0cf996fa5c2510724cfadb577651c3a438c3f2904cc4b94ab`,
the 5,324-file sorted-shasum XML aggregate is
`d8fad6bbab9ad0b5bece5299a983e588935cfd591d9430d38ddac900ec9eea1d`,
all 388 untracked outputs are absent, and the worktree is clean. Restoration is
owned cleanup, creates no commit, and receives no implementation review.

No C08 task runs scoped or full generation. C08R alone may run the future full
lineage after every input and production correction is task-clean and frozen.
No task changes generator architecture, dependencies, features, manifest,
browser policy, launch profile, base style, Taffy, public API, MSRV, root, or a
later-owned subsystem.

## Impacts

- **Public API and compatibility:** unchanged.
- **Production:** unchanged; C08 adds characterization only for C08R.
- **Fixtures:** one named HTML break; narrow helper/serializer/parser/comparator
  facts. XML/report stay at committed entry hashes.
- **Generator:** serializer and test changes only; no architecture or run.
- **Docs/root:** this plan and evidence only; C08R owns lineage, C09 closure.
- **Safety:** no unsafe and no new `allow` or `expect` attribute.

## Tasks

### `C08-X1` Separate Phase Evidence And Characterize Ten Production Rows

**Files/area:** test modules in
`tests/bin/surgeist-layout-generate/generator.rs`,
`tests/layout/browser_parity.rs`, `tests/layout/browser_parity/support.rs`, and
`src/root_tests.rs`; no production or generated output.

**Outcome:** Give stale-entry preflight tests names outside the `fri06_c08_`
post-lineage filter. Preserve immutable-input checks as a separately selectable
post-lineage contract. Reconcile literal 36/352 entry evidence with the executed
104/284 census. Add exact public-fixture-shaped characterization for the two RTL
`fri06_atomic_inline_percentage_block_size` rows, all four
`fri06_vertical_break_clear` rows, and all four `fri06_float_line_exclusion`
rows. Record complete actual and browser geometry without changing production or
relaxing the comparator.

**RED:** Reconstruct the ten browser assertions at the cycle base: direct RTL
Range start is 180 rather than 73.296875; vertical atomic physical x is 78
rather than 75; float-line root height is 62 rather than 63. Existing broad
tests are controls, not substitutes. The passing characterization fixes actual
line bands, child positions, baselines, clear effects, float endpoints, and root
sizes so C08R cannot solve only the first mismatch.

**Acceptance:** Preflight-only stale assertions cannot match `fri06_c08_`;
post-lineage tests contain no stale-output or dirty-status predicate. Exact
fixture-shaped characterization passes for both scalar lanes and all named
variants, while a temporary expected-browser assertion supplies honest RED.
Horizontal fallback, top/bottom alignment, pure-text, all-atomic, nonterminal,
and integral-height controls remain unchanged.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri06_c08_recovery_characterization_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --test layout fri06_c08_recovery_characterization_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib fri06_c08_recovery_characterization_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
```

**Dependency:** diagnostic restoration is complete.

**Intended commit:** `test(layout): characterize final C08 production rows`.

### `C08-X2` Correct Blockified BR, Range-Line, And Shape Inputs

**Files/area:** `tests/layout/browser_parity/scripts/gentest/test_helper.js`,
narrow serialization and tests in
`tests/bin/surgeist-layout-generate/generator.rs`, exactly
`tests/layout/browser_parity/html/float/fri06_float_shape_exclusion.html`, and
focused test-only assertions in `tests/layout/browser_parity.rs`.

**Outcome:** For computed block BR without line-control participation, retain
`source-tag="br"`, `display="block"`, and finite nonnegative used width/height
from existing `unroundedLayout`; emit no line-control, inline metrics, or atomic
facts. It remains an ordinary box. Maintain one root-local Range line registry:
physical block-progress keys use top for horizontal-tb, right progression for
vertical/sideways-rl, and left progression for vertical/sideways-lr; source-order
anchors within 0.1px share an index, otherwise allocate the next. Nested roots
reset; missing, nonfinite, multiple, or ambiguous facts reject. Mark the exact
explicit inline root. Add only the existing-schema source-index-4 allowed break
after the 38px shape atomic and before the 42px atomic.

**RED:** Real serializer-to-parser fixtures reproduce zero-sized BR boxes, all
four independent runs at line zero, and prohibited shape breaks. Supplemental
schema tests alone do not satisfy RED.

**Acceptance:** Exact horizontal `0x10`, vertical `10x0`, and unequal-flex
`0x19` BR controls pass while inline BR remains a control and ordinary non-BR
boxes are unchanged. Four-run IDs `0,4,8,12` receive lines `0,1,2,3`; same-line,
nested-root, and invalid controls pass. Shape atomics place 34/38 on line zero
and 42/46 on line one. Helper/HTML/source inventories are otherwise unchanged.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri06_c08_recovery_inputs_
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just fmt-check
```

**Dependency:** X1 is task-clean.

**Intended commit:** `fix(parity): preserve C08 browser fixture inputs`.

### `C08-X3` Correct Range Translation And Continuation Strut

**Files/area:** `tests/layout/browser_parity/support.rs`, focused tests in
`tests/layout/browser_parity.rs`, and only the two manually constructed
parser-fixture roots in `tests/bin/surgeist-layout-generate/generator.rs`; no
production, HTML, helper, generator behavior, or artifacts.

**Outcome:** Parse the explicit-root marker and retain private parent/root
identity through ordinary and synthetic nodes. Translate each fragment from its
owner's parent through ancestor unrounded locations, stopping before the marked
explicit root and never adding the text-node union twice. Preserve strict
source, line, physical-edge, and advance checks. This corrects exactly 18 rows;
the two direct-root RTL characterization rows remain unchanged. For mixed-wrap,
derive the containing strut from serialized root metrics, baseline 14.8 and line
height 20, and insert it after the first 18px atomic before the 24px atomic. The
two existing direct-root parser fixtures carry the same explicit-root marker
required of serialized fixture input; this is test-input reconciliation only.

**RED:** Exact generated-shape snippets fail on 16 grid plus two nested RTL Range
starts and four 44-versus-46 mixed-wrap heights. The two direct-root RTL rows
remain characterized RED for C08R.

**Acceptance:** Nested and synthetic offsets add exactly once; direct-root LTR
adds zero; wrong root, cycles, missing marker, and mutated source/line/advance
fail, including an unmarked direct-root owner. Both manually constructed
direct-root parser fixtures include the explicit marker and retain their prior
layout assertions. All 18 translation rows compare. Both mixed lines are 23.2
unrounded, round to y 23 and total 46, and altered names/topologies do not
activate the finite strut.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --test layout fri06_c08_recovery_adapter_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just fmt-check
```

**Dependency:** X2 is task-clean.

**Intended commit:** `fix(parity): align C08 root-relative observations`.

## Completion

X1 through X3 each receive a fresh task review. The original task-clean C08
spans and these tasks remain disjoint review evidence for the composed cycle.
Before status completion, prove helper, HTML, serializer, parser, comparator,
adapter, production-characterization, sequence, manifest, browser, launch,
base-style, Taffy, stale report, and 5,324 XML hashes. Generated outputs and
production source remain unchanged from their committed C08 entry states.

Run:

```sh
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
CARGO_NET_OFFLINE=true just fmt-check
git diff --check bcdba3c49be09ad119c03ecdc4c77da803159132..HEAD
```

The tracked/nonignored Rust inventory must be nonempty. Repository-wide unsafe
and task-range added-`allow`/`expect` scans must have no matches. Make a separate
status-only `complete` commit with `cycle_head`, run final checks, and obtain a
fresh `surgeist-holistic-reviewer` CLEAN verdict for
`bcdba3c49be09ad119c03ecdc4c77da803159132..cycle_head`. Rerun the gates on
local `main`, fast-forward push authority `main`, fetch/read back, and prove
local, tracking, `FETCH_HEAD`, and live remote agreement. The handoff freezes
all corrected nonproduction inputs and exact ten-row characterization for C08R.

Blocker: none at planning time.
