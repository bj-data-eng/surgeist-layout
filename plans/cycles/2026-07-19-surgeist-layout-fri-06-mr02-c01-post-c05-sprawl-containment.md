# FRI-06-MR02-C01 Post-C05 Sprawl Containment

Status: draft

Cycle ID: `FRI-06-MR02-C01`

Owning repository: `surgeist-layout`

Cycle base: `98b861133d873b387fb0b19891692a59ab7a6587`

Reviewed specification:
`plans/specs/2026-07-19-surgeist-layout-fri-06-mr02-post-c05-sprawl-containment.md`
at established-v1 status-normalized SHA-256
`b5ac1953bb49e0441432c5c1c523e8cc390056d922ef87917374007199b7e632`,
commit `6870e9732`, sections `FRI-06-MR02.1` through `FRI-06-MR02.9`.

Implementation sequence: none; this is a one-cycle, single-repository
sub-initiative in the recorded post-C05 insertion window.

## Outcome

Publish one behavior-preserving descendant before FRI-06-C06 that makes shaped
text processing linear and centralizes only the proven-identical scroll-padding,
geometry-error, signed-zero, layout-rounding, and physical-edge primitives.

## Boundary

The cycle starts from clean, remotely verified C05 candidate
`98b861133d873b387fb0b19891692a59ab7a6587`. The reviewed specification and
mechanical report authorize only MR-001, MR-004, and MR-005 here.

All seven tasks are refactors. Each worker first adds the smallest focused
characterization, records its test-only diff and passing pre-change result at the
exact task base, then extracts without changing behavior. No task claims RED.

Excluded: MR-002, MR-003, MR-006, public API or trait changes, layout or error
policy changes, dependencies, features, MSRV, manifests, lockfiles, scripts, CI,
docs, examples, benchmarks as targets, root/sibling changes, broad cleanup,
HTML, parser, helpers, fixtures, XML, reports, generated artifacts, and every
generator/browser/parity/corpus command. No new lint allowance, `expect`,
Surgeist-owned `unsafe`, production accounting state, cache, or side table.

## Impacts

Public API and compatibility: unchanged; new helpers are crate-private.
Dependencies, features, lockfile, MSRV, docs/examples, root follow-up, fixtures,
and generated artifacts: unchanged. Generator architecture and execution:
absent. Owned Rust remains free of `unsafe` and new lint allowances.

## Tasks

### `MR02-C01-T1` Validate Duplicate Segment IDs In One Linear Scan

**Files:** `src/node_input.rs` and its focused existing Rust tests only.

**Outcome:** Add `fri06_mr02_duplicate_id_` characterization for empty, unique,
first-possible, final, and competing duplicate families in both scalar lanes.
Replace the preceding-slice search with one local `HashSet<InlineSegmentId>` and
return the current segment's existing `DuplicateSegmentId` payload on failed
insertion. Retain the empty check first and retain no set in the public value.

**Pre-change characterization:** Commit no implementation until the focused
tests pass against the exact task base. Record the test-only diff digest and the
passing command. Do not weaken existing constructor tests.

**Acceptance:** The first repeated occurrence in source order is unchanged;
public representation, equality, cloning, segment order, errors, and both scalar
lanes are unchanged. The constructor contains no preceding-slice duplicate scan.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_mr02_duplicate_id_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c02_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout
CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
CARGO_NET_OFFLINE=true just fmt-check
```

**Dependency:** Reviewed plan at `in_progress`.

**Intended commit:** `refactor(layout): linearize shaped segment validation`.

### `MR02-C01-T2` Summarize And Select Inline Lines In Linear Time

**Files:** `src/inline.rs`, `src/inline_tests.rs`, and directly required focused
block/root inline regression tests only.

**Outcome:** Add `fri06_mr02_inline_linear_` characterization covering the full
FRI-06-MR02.4 matrix in both scalars. Introduce one private allocation-free line
summary shared by committed-line materialization and intrinsic sizing. Update
the selector to accumulate pending inline extent once per participant per
invocation. Remove `pending_inline_extent`; intrinsic paths no longer allocate a
selected-unit/discard vector.

**Pre-change characterization:** Record the passing test-only patch and focused
result at the exact post-T1 base. Deterministic test-only operation counting must
show the old growing-prefix scan before implementation; use no timing threshold.

**Acceptance:** Duplicate-free shaped text, leading/trailing discard, allowed and
replacement soft breaks, mandatory/forced breaks, empty and overwide lines,
first/last breaks, bidi, mixed atomics/controls, intrinsic extents, float-band
retry/reselection, fragments, anchors, visual order, metrics, baselines, source
association, and operation order remain exact. Doubling a long no-break run has
bounded linear measured work. No prefix cache or production counter is added.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_mr02_inline_linear_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c02_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c03_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c04_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c05_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout
CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
CARGO_NET_OFFLINE=true just fmt-check
```

**Dependency:** T1 is task-clean.

**Intended commit:** `refactor(layout): linearize inline line selection`.

### `MR02-C01-T3` Use One Physical Scroll-Padding Conversion

**Files:** `src/scroll.rs`, `src/compute.rs`, `src/block.rs`, `src/flex.rs`,
`src/grid/mod.rs`, `src/grid/child.rs`, and focused leaf/block/flex/grid tests.

**Outcome:** Add `fri06_mr02_scroll_padding_` characterization for Auto/Value
sentinels on all four physical edges and all five consumers in both scalar
lanes. Add crate-private `OptimalRegionInsetsOf::from_scroll_padding` and remove
the five local conversion functions. Do not add public `From` or move layout
dependencies into scroll.

**Acceptance:** Top/right/bottom/left and Auto/Value mapping are exact through
leaf, block, flex, grid, and grid-child front doors. Scroll geometry, snap
targets, errors, and outputs are unchanged. Only the owned conversion contains
the value match and four-edge construction.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_mr02_scroll_padding_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c0
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout
CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
CARGO_NET_OFFLINE=true just fmt-check
```

**Dependency:** T2 is task-clean.

**Intended commit:** `refactor(layout): centralize scroll padding conversion`.

### `MR02-C01-T4` Use Compute-Owned Geometry Error Adapters

**Files:** `src/compute.rs`, `src/block.rs`, `src/flex.rs`, `src/grid/mod.rs`,
`src/grid/child.rs`, `src/grid/lanes.rs`, and focused
root/block/flex/grid/grid-lanes error tests.

**Outcome:** Add `fri06_mr02_geometry_error_` characterization for every row in
FRI-06-MR02.5. Add crate-private `layout_own_geometry_error` and
`layout_child_geometry_error` in compute and remove the duplicated algorithm
helpers. Keep block's optional inline-subject choice local and leaf/standalone
mapping unchanged. Route every grid-child and grid-lanes direct caller through
the same child adapter without changing its container/subject pair.

**Acceptance:** Exact node or container/subject site, run-mode operation,
`InvalidRootScrollGeometry` versus `InvalidBlockScrollGeometry`, first failure,
and no-publication behavior remain unchanged. The safe source error is consumed
as before. Scroll gains no layout error dependency.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_mr02_geometry_error_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout scroll_geometry_error_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout
CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
CARGO_NET_OFFLINE=true just fmt-check
```

**Dependency:** T3 is task-clean.

**Intended commit:** `refactor(layout): centralize geometry error mapping`.

### `MR02-C01-T5` Canonicalize Signed Zero Through The Scalar Layer

**Files:** `src/scalar.rs`, `src/value.rs`, `src/sizing.rs`,
`src/node_input.rs`, `src/scroll.rs`, and focused scalar/domain tests.

**Outcome:** Add `fri06_mr02_signed_zero_` characterization for the complete
FRI-06-MR02.6 matrix. Move the identical body to crate-private
`scalar::canonical_zero` and remove the calc-size, exclusion, and scroll copies.

**Acceptance:** Positive and negative zero canonicalize to positive zero in both
scalars. Finite nonzero, infinity, and NaN pass through unchanged. Every caller
retains validation, clamping, failure, and operation order. Exactly one helper
body remains.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_mr02_signed_zero_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout signed_zero
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c02_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c05_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout
CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
CARGO_NET_OFFLINE=true just fmt-check
```

**Dependency:** T4 is task-clean.

**Intended commit:** `refactor(layout): centralize signed zero canonicalization`.

### `MR02-C01-T6` Use Exact Layout-Coordinate Rounding

**Files:** `src/scalar.rs`, `src/compute.rs`, `src/scroll.rs`, and focused
rounding/scroll/root tests.

**Outcome:** Add `fri06_mr02_layout_round_` characterization for negative and
positive integer/fraction/half boundaries, large finite values, and cumulative
origins in both scalars. Add crate-private `round_layout_coordinate` with exact
`(value + 0.5).floor()` order and remove the two local production helpers.

**Acceptance:** Results remain exact and are not replaced with
`LayoutScalar::round`. Cumulative subtraction, signed-zero canonicalization,
overflow/error handling, fragments, baselines, scroll ranges, and publication
remain at their callers and unchanged.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_mr02_layout_round_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout rounding_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout
CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
CARGO_NET_OFFLINE=true just fmt-check
```

**Dependency:** T5 is task-clean.

**Intended commit:** `refactor(layout): centralize layout coordinate rounding`.

### `MR02-C01-T7` Select Physical Edges Through One Typed Accessor

**Files:** `src/geometry.rs`, `src/block.rs`, `src/compute.rs`, `src/flex.rs`,
`src/scroll.rs`, and focused geometry/block/flex/scroll tests.

**Outcome:** Add `fri06_mr02_physical_edge_` characterization using four distinct
sentinels and every migrated call-site family. Add crate-private
`Edges::at_physical_side` and remove only the identical value-selection helpers.

**Acceptance:** Every side returns its physical field. Flow mapping, rect or
coordinate construction, progression, float-side policy, the Flex edge setter,
Flex axis policy, and other semantic matches stay local. The identical Flex
getter is removed. No logical-side overload or public method is added.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_mr02_physical_edge_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c04_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout
CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
CARGO_NET_OFFLINE=true just fmt-check
```

**Dependency:** T6 is task-clean.

**Intended commit:** `refactor(layout): centralize physical edge selection`.

## Completion

After all seven task ranges are independently clean, make the separate
status-only `complete` commit and set the immutable cycle head. Run:

```sh
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_mr02_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c0
CARGO_NET_OFFLINE=true just fmt-check
rg -n 'fn canonical_zero|fn round_layout_coordinate|fn at_physical_side|fn from_scroll_padding|fn layout_own_geometry_error|fn layout_child_geometry_error' src
rg -n 'pending_inline_extent|canonical_calc_size_zero|canonical_exclusion_zero|canonical_scroll_zero|fn (leaf|block|flex|grid)_scroll_padding|fn (block|flex|grid)_(own|child)_geometry_error|fn edge_at_physical_side|fn physical_edge_value|fn edge_at_side' src; test $? -eq 1
rg -n '^fn round<S: LayoutScalar>' src/compute.rs src/scroll.rs; test $? -eq 1
git diff --check 98b861133d873b387fb0b19891692a59ab7a6587..HEAD
git diff --name-only --no-renames 98b861133d873b387fb0b19891692a59ab7a6587..HEAD
git diff --unified=0 98b861133d873b387fb0b19891692a59ab7a6587..HEAD -- '*.rs' | rg '^\+.*#\s*\[\s*(allow|expect)\s*\('; test $? -eq 1
rg -n --pcre2 '#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{' $(git ls-files --cached --others --exclude-standard '*.rs'); test $? -eq 1
git status --short
```

The declaration search must show exactly the intended crate-private owners. The
legacy, local-rounding, new allowance/expect, and unsafe searches must return no
matches. The printed changed-path inventory may contain only this specification,
this plan, the production paths named by T1 through T7, and their directly
required focused existing Rust test modules. Any other path fails completion.
Status must be clean. No temporary manifest or generator command is used.

A fresh `surgeist-holistic-reviewer` must return `CLEAN` for exact range
`98b861133d873b387fb0b19891692a59ab7a6587..cycle_head`. Rerun the complete final
set on local `main`, publish the immutable SHA to authority remote `main` with
the standard leased fast-forward, fetch/read back, and prove local, tracking,
`FETCH_HEAD`, and live remote agreement. Remove every cycle-owned resource.

The handoff records the published candidate as FRI-06-C06's required base and
confirms that no fixture or generator input changed. MR-002 broad migration and
MR-003 remain after FRI-06-C07 and the leaf candidate handoff. Blocker: none at
planning time.
