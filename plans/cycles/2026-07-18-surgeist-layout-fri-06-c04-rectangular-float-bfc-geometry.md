# FRI-06-C04 Rectangular Float And BFC Geometry

Status: in_progress

Cycle ID: `FRI-06-C04`

Owning repository: `surgeist-layout`

Cycle base: `5e4a0e97aeb40118ee1bbb660c0c93ea2058cde9`

Reviewed specification:
`plans/specs/2026-07-17-surgeist-layout-fri-06-inline-formatting-floats-bfcs.md`
at normalized SHA-256
`7cb09e0a8e9036a406b39115ed8f6392df805116a762905a3510c7fe7355f970`,
commit `64a9ca96be3b29765b0ec2e7fb13de7e96934866`, decisions `D-13` and
`D-15`; rectangular float, clear, BFC, sizing, baseline, scroll, cache,
rounding, evidence, and acceptance portions of `FRI-06.7` through `FRI-06.9`
and `FRI-06.14`.

Reviewed sequence:
`plans/sequences/2026-07-17-surgeist-layout-fri-06-inline-formatting-floats-bfcs.md`
at normalized SHA-256
`ed9b4a5bac63617ad5d7d3c76791dd42d93089a210ab16c930a7e727ed7edd57`,
commit `24bb3ccd0a4c9f54bc9eaa7958a9d2ea740bf859`, entry `FRI-06-C04`.

## Outcome

Complete rectangular float interaction over the C03 unified inline engine.
Make margin-box exclusions flow-relative and source ordered; place mapped
line-left/right, opposing, stacked, cleared, and overwide floats through
full-span finite-transition queries; give every ordinary line its own band;
enclose owned floats in auto block size; trap nested floats; and implement the
exact current flex/grid/grid-lanes plus non-replaced-overflow BFC avoidance and
auto-inline-size contract. Leave the same band-query ownership ready for C05 to
refine with provider-backed shapes.

## Boundary

At the remotely verified cycle base, C03 supplies one logical mixed line
builder and mapped post-line clear intent, but its normal input still starts
from the full containing inline extent. `FloatExclusions` records physical
left/right rectangles, queries one physical y coordinate rather than the full
candidate span, and retains a separate no-text clear bridge. Existing float
and BFC tests characterize useful horizontal behavior, but float sides, clear,
auto sizing, and placement are not closed over all containing flows or the
reviewed role predicate.

This cycle owns the private rectangular ledger/query, float placement, unified
line-band integration, exact BFC predicate and sizing, float-owned auto block
size, and directly required focused Rust tests in `src/block.rs`,
`src/inline.rs`, and existing Rust test modules. Small private geometry or
scroll helpers may change only when directly required by those paths.

C05 alone owns `FloatExclusion::Shape` provider invocation, interval
refinement, provider errors, query records, and provider invalidation. C06 owns
parser/helper/comparator, HTML, fixtures, XML, reports, provenance, and the one
final full regeneration. No generator command or architecture change, public
shape-provider expansion, authored CSS/text, shaping, manifest, dependency,
feature, lockfile, docs, root, sibling, or generated artifact enters C04. No
new lint allowance or Surgeist-owned `unsafe` is permitted.

The validated mechanical opportunities remain recorded in
`plans/2026-07-18-surgeist-layout-mechanical-refactoring-review-findings.md`.
MR-006 and the isolated scalar-generic `OracleTreeOf<S>` slice of MR-002 were
completed in the reviewed post-C02 containment cycle. MR-001, MR-004, and
MR-005 remain scheduled after C05; broader MR-002 migration and MR-003 remain
scheduled after C07. None enters C04, and C04 must not create new duplicate
axis algorithms, scanners, or fixture helpers.

## Impacts

Public API and type shape: unchanged; this cycle consumes the C01 float model
and C03 line reports. Dependencies, features, MSRV, lockfile, docs/examples,
root handoff, browser corpus, and generated artifacts: unchanged. Behavior
advances represented margin-box floats and current BFC roles from physical or
partial placement to the reviewed logical contract. Generator execution:
absent.

## Tasks

### `C04-T1` Make Rectangular Exclusions And Float Placement Flow-Relative

**Files:** `src/block.rs` and focused block/root Rust tests.

**Outcome:** Replace the physical-left/right point query with one source-ordered
rectangular ledger keyed by mapped line-start/line-end side, physical margin
box, logical block interval, and source order. Query a candidate's complete
block span for farthest inward start/end constraints and the next strictly later
finite transition. Place floats from the current clear-adjusted block position,
preserve source order, and retry only by committing placement or advancing to
that transition. An overwide float stays on its required mapped side and may
overflow without negative available size or retry bounds.

**RED:** Add production-front-door tests prefixed `fri06_c04_float_ledger_`,
`fri06_c04_float_place_`, and `fri06_c04_float_progress_` first. At the exact
task base, non-horizontal sides remain physical, a full-span collision can be
missed, opposing/stacked placement diverges, or overwide progress uses legacy
geometry. Preserve reconstructible test-only RED evidence.

**Acceptance:** Both scalar lanes prove all ten flows with mapped left/right,
same-side source ordering, opposing and stacked rectangles, asymmetric margins,
clear left/right/both, a collision that overlaps only part of the candidate
span, zero-width bands, exact next-transition progression, and overwide
termination. Private ledger tests supplement front-door geometry and prove a
float/span pair is evaluated once per candidate pass. No shape provider is
queried or approximated.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c04_float_ledger_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c04_float_place_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c04_float_progress_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout
CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
CARGO_NET_OFFLINE=true just fmt-check
```

**Dependency:** Reviewed plan at `in_progress`. **Intended commit:**
`feat(layout): make rectangular float placement logical`.

### `C04-T2` Drive Unified Inline Lines And Clear From Rectangular Bands

**Files:** `src/inline.rs`, `src/block.rs`, and focused inline/block/root Rust
tests.

**Outcome:** Give each C03 logical line candidate its own rectangular
start/end/block band and provenance. The unified builder owns one per-line band
retry callback: from the current source cursor and block start it selects a
provisional line, derives that line's block extent, queries the complete
`[block_start, block_end)` span, and reselects from the same source cursor
against the returned band before committing. If no non-negative band can accept
the candidate, the callback returns one strictly later finite transition; the
builder advances there, repeats same-cursor selection/span query, and publishes
only the final selected line. A changed band cannot reuse stale full-width
break selection or invoke a second line-axis algorithm. Apply C03's mapped
post-line clear intent before the following line. Remove the separate
`layout_inline_segments` rectangular-clear bridge so text, atomics, controls,
and atomic-only runs use the same builder and band source.

**RED:** Add front-door tests prefixed `fri06_c04_line_band_`,
`fri06_c04_line_clear_`, and `fri06_c04_line_alignment_` first. At the exact T2
base, ordinary lines still overlap floats, mixed clear does not advance against
active exclusions in every flow, or unequal lines align against the containing
width instead of their own bands. Preserve reconstructible RED evidence.

**Acceptance:** Both scalar lanes prove text/atomic/control lines beside
left/right/opposing floats; soft and forced lines whose block spans meet
different transitions; a transition-caused rewrap from the same source cursor;
no-space advancement and indivisible overwide progress;
all ten flows times every clear value with matching/nonmatching sides; and
legacy left/right/center on unequal float-adjusted bands. Source, visual, line,
baseline, and zero-size control identity remain exact. Ordinary block outer
edges remain at the containing edge while their internal lines see exclusions.
`layout_inline_segments` and any second band/projection path are absent.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c04_line_band_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c04_line_clear_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c04_line_alignment_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c03_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout
CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
CARGO_NET_OFFLINE=true just fmt-check
rg -n 'layout_inline_segments' src
```

The final search must return no matches. **Dependency:** T1 is task-clean.
**Intended commit:** `feat(layout): apply rectangular bands to inline lines`.

### `C04-T3` Close The Current BFC Avoidance And Sizing Predicate

**Files:** `src/block.rs` and focused block/root Rust tests.

**Outcome:** Centralize the exact private Boolean predicate: the subject is an
in-flow, non-floating, non-absolute, box-producing block-level child and
`(display is Flex, Grid, or GridLanes || (!item_is_replaced && normalized
computed overflow establishes an independent formatting context))`. Display
none, inline-level, absolute, floating, and the replaced-overflow branch are
false; ordinary Block with visible or clip overflow is false. Qualifying
children use the same full-span rectangular query; auto inline size receives
the saturated band width before child layout, while definite/overwide margin
boxes move through finite transitions and then overflow at the normal block
position if needed.

**RED:** Add front-door tests prefixed `fri06_c04_bfc_role_`,
`fri06_c04_bfc_size_`, and `fri06_c04_bfc_nested_` first. At exact T3 base,
some supported displays/overflow pairs are omitted or overaccepted, auto width
uses the containing extent, definite children overlap, or nested/floating/
atomic contexts leak exclusions. Preserve reconstructible RED evidence.

**Acceptance:** Both scalar lanes and representative parallel/opposing flows
prove the complete positive and negative predicate table; flex/grid/grid-lanes
including replaced flex/grid/grid-lanes, plus non-replaced hidden/scroll/auto
overflow ordinary blocks as positive cases; visible/clip ordinary blocks and
replaced ordinary overflow-established blocks as negative cases; and float,
atomic, absolute, none, and inline exclusions. Auto, definite, zero, and
overwide widths with margins and clear, ordinary outer-edge stability, and
nested BFC/float containment are exact. Atomic and floating boxes trap internal
floats through their own paths without entering block-child avoidance.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c04_bfc_role_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c04_bfc_size_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c04_bfc_nested_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c04_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout
CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
CARGO_NET_OFFLINE=true just fmt-check
```

**Dependency:** T2 is task-clean. **Intended commit:**
`fix(layout): close rectangular BFC avoidance`.

### `C04-T4` Close Float Size, Scroll, Cache, Rounding, And Invalidation

**Files:** `src/block.rs`, directly required `src/compute.rs` cleanup only, and
focused cache/contract/block/root Rust tests.

**Outcome:** Enclose owned in-flow floats in auto block size without counting
the container start inset twice; preserve intrinsic float contributions without
final placement or shape queries; publish each float and final line geometry
once to the canonical scroll accumulator; and close cold/warm, dirty,
transaction failure, rounding, baseline, and content-size behavior for the
rectangular float/BFC path.

**RED:** Add front-door tests prefixed `fri06_c04_float_size_`,
`fri06_c04_float_scroll_`, and `fri06_c04_float_lifecycle_` first. Reconstruct
RED only for genuine missing behavior at exact T4 base; record already-correct
cache/transaction diagnostics without claiming false RED.

**Acceptance:** Both scalar lanes prove float-only and mixed auto block size,
container insets, nested containment, min/max-content, clear and overwide
contributions, signed/fractional scroll geometry, source-ordered node output,
normal/rounded equality of side/band/position identity, real warm cache reuse,
exact dirty-path replacement, and layout/preparation failure atomicity. Static
evidence proves one rectangular ledger/query and no legacy physical-side or
second band table remains. No provider state or cache-key field is added.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c04_float_size_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c04_float_scroll_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c04_float_lifecycle_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c04_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c03_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout
CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
CARGO_NET_OFFLINE=true just fmt-check
```

**Dependency:** T3 is task-clean. **Intended commit:**
`fix(layout): close rectangular float lifecycle`.

## Completion

After all four task ranges are independently clean, make the plan's separate
status-only `complete` commit and set the immutable cycle head. Run:

```sh
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c04_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c03_
CARGO_NET_OFFLINE=true just fmt-check
rg -n 'layout_inline_segments' src
git diff --unified=0 5e4a0e97aeb40118ee1bbb660c0c93ea2058cde9..HEAD -- '*.rs' | rg '^\+.*#\s*\[\s*allow\s*\('
git diff --check 5e4a0e97aeb40118ee1bbb660c0c93ea2058cde9..HEAD
git diff --name-only --no-renames 5e4a0e97aeb40118ee1bbb660c0c93ea2058cde9..HEAD
git ls-files --others --exclude-standard
test -z "$( (git diff --name-only --no-renames 5e4a0e97aeb40118ee1bbb660c0c93ea2058cde9..HEAD; git ls-files --others --exclude-standard) | LC_ALL=C sort -u | rg -v '^(plans/cycles/2026-07-18-surgeist-layout-fri-06-c04-rectangular-float-bfc-geometry\.md|src/(block|block_tests|compute|contract_tests|inline|inline_tests|lib_tests|root_tests)\.rs)$')"
owned_rust_manifest="$(mktemp -t surgeist-layout-fri06-c04-owned-rust)"
trap 'rm -f "$owned_rust_manifest"' EXIT
git ls-files --cached --others --exclude-standard '*.rs' > "$owned_rust_manifest"
rg -n --pcre2 '#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{' $(cat "$owned_rust_manifest")
git status --short
test -z "$(git status --porcelain=v1)"
```

The sorted changed-path allowlist is exactly this plan plus `src/block.rs`,
`src/block_tests.rs`, `src/compute.rs`, `src/contract_tests.rs`,
`src/inline.rs`, `src/inline_tests.rs`, `src/lib_tests.rs`, and
`src/root_tests.rs`; any changed path outside that set fails completion. Allowed
paths need not all change. The executable allowlist covers both committed range
paths, with rename detection disabled so both deletion and addition sides are
checked, and every nonignored untracked path; the final assertion requires an
empty worktree. The legacy-bridge, newly added lint-allowance, and unsafe
searches must return no matches, with expected no-match statuses recorded. The
owned manifest
includes tracked and non-ignored untracked Rust, including test and generator
Rust, and excludes ignored build/dependency roots. Its assignment, cleanup
trap, construction, scan, and status check execute in one shell scope; record
and prove removal of the unique path. Specification, sequence, mechanical
report, parser, helper, HTML, fixture, XML, report, provenance, generator,
manifest, dependency, feature, lockfile, docs, root, sibling, and generated
artifact changes are forbidden.

A fresh `surgeist-holistic-reviewer` must return `CLEAN` for exact range
`5e4a0e97aeb40118ee1bbb660c0c93ea2058cde9..cycle_head`. Prove local `main` is
that candidate, rerun the complete final command set there without changing the
head, publish by fast-forward to the authority remote `main`, fetch/read back,
and prove local, tracking, `FETCH_HEAD`, and observed remote `main` agree with
the candidate or a normal descendant containing it. Remove every cycle-owned
temporary resource.

The handoff records the published C04 candidate and confirms margin-box
exclusion is ready for C05's bounded provider refinement. No C05 plan or
implementation begins before remote verification. Blocker: none at planning
time.
