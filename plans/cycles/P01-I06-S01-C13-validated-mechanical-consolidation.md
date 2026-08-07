# P01-I06-S01-C13 Validated Mechanical Consolidation

Status: in_progress

Cycle ID: `P01/I06/S01/C13`

Owning repository: `surgeist-layout`

Cycle base: `dc82c4566ad0ab95c443638aab8fda15fca8db78`

Reviewed specification:
`plans/specs/P01-I06-inline-formatting-floats-bfcs.md`, normalized semantic-content
SHA-256 `6135d2c542967d938508f6a15b9940dbe02b3b15221109911bbad611c67c25a6`,
commit `45b6a9b987b37328963c8c1feff67368e91de70b`: complete disposition contract
`FRI-06.13.1`, module and test contracts in `FRI-06.9` and `.10`, and initiative
acceptance in `FRI-06.14`.

Reviewed implementation sequence:
`plans/sequences/P01-I06-S01-inline-formatting-floats-bfcs.md`, normalized
SHA-256 `43f9443afe6297d493967c640ccb13e23d0c6532a2529ec3aa99bb41f327a393`,
commit `89f9c02d23cba29d51a1f7e5aa88d85f6c9ef5ee`, entry
`P01/I06/S01/C13`.

Bounded outcome: validate every current-source `FRI-06.13.1` equivalence class;
consolidate the remaining ordinary MR-002 tree harnesses and exact MR-003 layout
math classes; preserve each specialized or policy-distinct counterexample; and
return the published final FRI-06 candidate with all six MR dispositions and all
59 initial finding assignments traceable.

## 1 Boundary

C12 candidate `dc82c4566ad0ab95c443638aab8fda15fca8db78` is the clean, published,
remotely read-back entry state. C03 already closed MR-006 and replaced the two
scalar-specific `OracleTreeOf` implementations with one generic implementation.
C07 already closed MR-001, MR-004, and MR-005. Those four results are immutable
behavior contracts to revalidate, not implementation work to reopen.

The durable mechanical-review input is
`plans/P01-layout/P01-I06-mechanical-refactoring-review-findings.md`, SHA-256
`11437dd9dfe83d41ae6b01e41453d9cc1a893172c6977e5b3d77346aa3948f34`.
MR-002 broad harness consolidation and MR-003 selective math consolidation are
the only remaining implementation fronts.

MR-002 has exactly two reusable ordinary harness roles:

- `OracleTreeOf<S>` remains the scalar-generic internal `Compute`/`Round` harness
  for map-backed children, layout input, deterministic measurement, staged output,
  and recorded compute input.
- one test-only `PublicLayoutTreeOf<S>` may own map-backed children, layout input,
  and deterministic optional leaf measurement for ordinary public
  `compute_layout` cases.

A local fake remains only when its source visibly provides at least one behavior
the ordinary harnesses must not absorb: injected failure, query/call observation,
call order, cache state or key behavior, invalid topology, identity other than the
ordinary `u32` map key, mutable publication/transaction state, or deliberately
nonstandard child/input behavior. Each retained fake keeps that distinction in its
type, fields, methods, or a succinct orienting comment. No macro inventories or
production-visible test surface are permitted.

MR-003 has one crate-private `layout_math` owner for only these exact classes:

- optional-size fallback, unwrapping, optional addition, and aspect-ratio
  projection shared by block, flex, grid, and leaf compute;
- unchecked optional subtraction shared by flex and grid;
- max-before-min optional clamping shared by block, flex, and leaf compute;
- resolution-to-zero and optional resolution shared by block, flex, and grid; and
- containing-flow padding/border resolution in block, flex, and grid constants
  only when flow axes, parent percentage basis, scalar operation order, and error
  conversion remain identical.

Block's zero-clamped subtraction, grid's min-before-max clamp, leaf compute's
missing-basis error, and child/absolute/leaf edge resolution with a different
percentage basis remain local, explicitly named counterexamples. The shared owner
uses free functions or narrowly sealed crate-private extension methods; it does not
create a broad helper trait or expose a public API.

Out of scope: any layout, error, cache, fragment, scalar-lane, or parity behavior
change; public API or trait changes; dependencies, features, MSRV, manifests,
lockfiles, scripts, CI, docs, examples, benchmarks, root or sibling changes;
fixture, HTML, parser, browser helper, report, XML, generated output, generator
logic, or generator execution; generator architecture; macro-driven wholesale
rewrites; new lint allowances or `expect` attributes; FRI-07 behavior; and
unrelated cleanup. No Surgeist-owned `unsafe` is permitted.

## 2 Impacts

Public API: internal-only; no production export changes.

Dependencies, features, manifests, lockfiles, and MSRV: unchanged.

Generated artifacts: unchanged. The browser helper, report, complete XML,
activation bodies, preserved bodies, and inventory remain respectively:

- `42bf9ff77810b2e9fb5a184f525d9e22f74abae12a09f9486b3b49dc620188c2`;
- `8d59c87d1fcc185bda0372968ae81dbeff74f241c17335db98629ad49f1f463f`;
- `d2530aa79f214b536e46aee263095a6e7c0a1d7a329bdce7baeb194af3670896`;
- `f3d9b41973e6b0e51e258f027496dc2651c4fba7d24567b05d4f088ee63de335`;
- `b2684877302ed7b1b6b1e52b5ae4c4ae4508ff425d6c34ff237b7e37440a3c79`;
  and
- `0c327c2d93b140ea5ed5660e45ad947a0afb583b9aa97b3163ea59b45d371715`.

Docs/examples: unchanged except this canonical cycle plan. Root follow-up is the
final published leaf-candidate handoff only. Owned Rust remains free of `unsafe`.

## 3 Tasks

Every task is a behavior-preserving refactor. No task claims a failing behavioral
RED. Its worker first runs the named focused characterization at the exact task
base, records the passing pre-change result, then repeats it after the mechanical
change. A failure at the task base is diagnosis, not permission to weaken a test.

### 3.1 `P01/I06/S01/C13/T01` Establish The Ordinary Public Tree Contract

**Files/area:** `src/test_support/layout_tree.rs`; `src/test_support/mod.rs` only if
the existing module export requires adjustment.

**Outcome:** add one test-only `PublicLayoutTreeOf<S>` implementing `Traverse` and
`LayoutTree` for ordinary map-backed public-front-door tests. It preserves source
child order, exact `LayoutInputOf<S>`, box-node access, absence versus presence of
deterministic leaf measurement, both scalar lanes, and existing panic/error
boundaries. `OracleTreeOf<S>` remains the sole ordinary internal compute harness.

**Pre-change characterization:**
`CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout test_support::layout_tree::tests`
passes at the task base. Add focused `fri06_c13_t01_` tests that fail to compile or
fail behavior if the new harness loses f32/f64 typing, child order, non-box input,
measurement absence/presence, or ordinary public layout.

**Acceptance:** the support type has no cache, observation, failure-injection,
topology, transaction, or publication policy; no macro is added; existing oracle
tests and new focused tests pass.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c13_t01_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout test_support::layout_tree::tests
cargo fmt --check
```

**Dependency:** none. **Intended commit:**
`test(layout): define ordinary public tree support`.

### 3.2 `P01/I06/S01/C13/T02` Consolidate Ordinary Block And Flex Trees

**Files/area:** `src/block_tests.rs`, `src/flex_tests.rs`, and only the T01 support
surface needed by their ordinary cases.

**Outcome:** classify every local tree in both modules. Replace each ordinary
map-backed compute tree with `OracleTreeOf<S>` and each ordinary public tree with
`PublicLayoutTreeOf<S>`. Retain specialized fakes only under the Section 1
predicate and keep their distinguishing behavior visible. Preserve test names,
scenario data, measurement matching, panic text when asserted, child order, and
phase boundaries.

**Pre-change characterization:**
`CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout block_tests::` and
`CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout flex_tests::` pass
at the task base.

**Acceptance:** no ordinary local `Traverse`/`Compute`/`LayoutTree` implementation
remains in either module; every retained implementation has a source-visible
specialization; both module suites preserve their exact pass/ignore totals; no
behavior assertion is weakened.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout block_tests::
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout flex_tests::
cargo fmt --check
```

**Dependency:** T01. **Intended commit:**
`test(layout): consolidate block and flex tree harnesses`.

### 3.3 `P01/I06/S01/C13/T03` Consolidate Ordinary Grid Trees

**Files/area:** `src/grid_tests.rs` and only the T01 support surface needed by its
ordinary cases.

**Outcome:** classify every local grid tree. Replace ordinary map-backed internal
and public trees with the two typed harnesses. Preserve dedicated subgrid,
recursive/topology, query-observation, measurement-history, failure, cache, and
non-`u32` identity fakes with their distinguishing behavior visible.

**Pre-change characterization:**
`CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout grid::tests::`
passes at the task base.

**Acceptance:** no ordinary local `Traverse`/`Compute`/`LayoutTree` implementation
remains; every retained implementation satisfies the specialization predicate;
the grid suite preserves its exact pass/ignore totals and all geometry/error
assertions.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout grid::tests::
cargo fmt --check
```

**Dependency:** T01. **Intended commit:**
`test(layout): consolidate grid tree harnesses`.

### 3.4 `P01/I06/S01/C13/T04` Close The Remaining Tree Inventory

**Files/area:** `src/root_tests.rs`; other tracked `src/*_tests.rs` files that
define local `Traverse`, `Compute`, or `LayoutTree` implementations; and only the
`#[cfg(test)]` modules in `src/compute.rs` and `src/grid/subgrid.rs`; T01 support
only where an ordinary case needs it.

**Outcome:** apply the same complete classification outside block, flex, and grid.
Migrate ordinary trees and retain specialized validation, cache, invalidation,
transaction, observation, failure, topology, and publication fakes. In
`compute::tests`, migrate ordinary `EmptyTree`; retain `BoundedDagTree` for its
adjacency-budget/order observation and `FragmentTree` for committed-fragment
readback state and call counting. Retain `grid::subgrid::tests::TraversalTree`
because its unreachable child-compute hook proves track-initialization failure is
propagated first. Production bodies in those two modules do not change. This task
does not pull specialized behavior into shared support merely to reduce a count.

**Pre-change characterization:**
`CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout root_tests::` and
`CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout compute::tests::`,
`CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout grid::subgrid::tests::`,
and `CARGO_NET_OFFLINE=true just verify` pass at the task base.

**Acceptance:** the tracked test source contains only the two ordinary typed
harnesses plus locally specialized fakes; every retained fake exposes its
distinguishing predicate; default verification preserves exact pass/ignore totals.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout root_tests::
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout compute::tests::
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout grid::subgrid::tests::
CARGO_NET_OFFLINE=true just verify
```

**Dependency:** T01-T03. **Intended commit:**
`test(layout): close ordinary tree harness inventory`.

### 3.5 `P01/I06/S01/C13/T05` Share Exact Optional-Size Arithmetic

**Files/area:** new private `src/layout_math.rs`; `src/lib.rs`; `src/block.rs`,
`src/flex.rs`, `src/grid/mod.rs`, `src/compute.rs`; focused module tests.

**Outcome:** move only the exact optional-size classes named in Section 1 to the
private owner. Preserve operand order and aspect-ratio projection. Share unchecked
subtraction only between flex/grid and max-before-min clamp only between
block/flex/compute. Keep block zero clamping and grid min-before-max clamping local
with explicit names.

**Pre-change characterization:** add `fri06_c13_t05_` tests for `None` fallback,
per-axis unwrap/add/subtract, width-to-height and height-to-width projection,
negative unchecked subtraction, block zero clamp, ordinary and conflicting
min/max, and both scalar lanes. Commit no extraction until the test-only diff
passes at the task base.

**Acceptance:** the four repeated common methods have one implementation; the two
shared policy classes have one implementation each; named counterexamples retain
their operation order; no call-site result or public surface changes.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c13_t05_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --lib
cargo fmt --check
```

**Dependency:** T04. **Intended commit:**
`refactor(layout): share exact optional size arithmetic`.

### 3.6 `P01/I06/S01/C13/T06` Share Exact Resolution And Edge Arithmetic

**Files/area:** `src/layout_math.rs`, `src/block.rs`, `src/flex.rs`,
`src/grid/mod.rs`, `src/compute.rs`; focused module tests.

**Outcome:** centralize block/flex/grid resolution-to-zero and optional-resolution
semantics. Consolidate their constants-construction padding/border resolution only
where containing flow, parent basis, operation order, and error mapping are exact.
Keep leaf compute's missing-basis error and every different child/absolute basis as
named local counterexamples.

**Pre-change characterization:** add `fri06_c13_t06_` tests covering resolved,
missing-basis, nonnumeric, invalid-numeric, zero, negative, horizontal/vertical
flow, definite/indefinite parent basis, edge mapping, and the leaf missing-basis
counterexample in both scalar lanes. Commit no extraction until the test-only diff
passes at the task base.

**Acceptance:** block/flex/grid share one status mapping and one exact
constants-edge path; compute and basis-distinct call sites remain local; existing
error sites and geometry are unchanged.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c13_t06_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --lib
cargo fmt --check
```

**Dependency:** T05. **Intended commit:**
`refactor(layout): share exact resolution arithmetic`.

## 4 Completion

Each implementation task receives a fresh `surgeist-worker`; every ordered task
range receives a distinct fresh `surgeist-task-reviewer` and must be `CLEAN`.
Record the current-source MR-002 classification counts and retained-specialization
predicates, the MR-003 merged classes and named counterexamples, and the unchanged
MR-001/MR-004/MR-005/MR-006 owners in the final completion evidence and candidate
handoff. The six rows together are the durable final disposition.

Revalidate the four earlier dispositions at their current owners:

- MR-001: `InlineTextInputOf::try_new` in `src/node_input.rs` retains source-order
  first-duplicate selection, while the private line summary and incremental scan
  in `src/inline.rs` retain allocation-free intrinsic sizing and deterministic
  linear scaling; `fri06_mr02_duplicate_id_` and `fri06_mr02_inline_linear_` pass.
- MR-004: `OptimalRegionInsetsOf::from_scroll_padding`,
  `layout_own_geometry_error`, and `layout_child_geometry_error` in
  `src/compute.rs` remain the single owners; `fri06_mr02_scroll_padding_` and
  `fri06_mr02_geometry_error_` pass.
- MR-005: `scalar::canonical_zero`, `scalar::round_layout_coordinate`, and
  `Edges::at_physical_side` remain the single exact primitive owners;
  `fri06_mr02_signed_zero_`, `fri06_mr02_layout_round_`, and
  `fri06_mr02_physical_edge_` pass.
- MR-006: private `compute::non_box_node_role_error` retains exact first-error
  order and role-specific parent handling; `fri06_mr01_non_box_` passes.

After all task reviews are clean, change only `Status` to `complete` in a separate
commit. At that exact head run:

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri06_c12_t09_final_lineage_hashes_match_preserved_run
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_mr01_non_box_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_mr01_oracle_generic_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_mr02_duplicate_id_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_mr02_inline_linear_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_mr02_scroll_padding_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_mr02_geometry_error_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_mr02_signed_zero_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_mr02_layout_round_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_mr02_physical_edge_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
test "$(shasum -a 256 plans/P01-layout/P01-I06-mechanical-refactoring-review-findings.md | cut -d ' ' -f 1)" = 11437dd9dfe83d41ae6b01e41453d9cc1a893172c6977e5b3d77346aa3948f34
git diff --exit-code dc82c4566ad0ab95c443638aab8fda15fca8db78..HEAD -- Cargo.toml Cargo.lock README.md tests/layout/browser_parity tests/layout/browser_parity.rs tests/bin/surgeist-layout-generate.rs tests/bin/surgeist-layout-generate/generator.rs plans/P01-layout/P01-index.md plans/P01-layout/P01-initial-review-findings.md
git ls-files --cached --others --exclude-standard '*.rs' > /tmp/surgeist-layout-c13-owned-rust.txt
! xargs rg -n --pcre2 '#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{' < /tmp/surgeist-layout-c13-owned-rust.txt
rm /tmp/surgeist-layout-c13-owned-rust.txt
git diff --check dc82c4566ad0ab95c443638aab8fda15fca8db78..HEAD
test -z "$(git status --short)"
```

The hash test proves all six frozen C12 artifact digests without generation. The
path diff proves no manifest, lockfile, crate docs, fixture, helper, generator,
generated output, authoritative ownership index, or 59-finding assignment
changed. The owned-Rust manifest includes every tracked and non-ignored untracked
Rust file and the scan must return no match; classify any match before advancing.
A fresh
`surgeist-holistic-reviewer` must return `CLEAN` for exact range
`dc82c4566ad0ab95c443638aab8fda15fca8db78..cycle_head`, including the complete
MR-002 source inventory and all MR-003 equivalence/counterexample predicates.

Rerun the same final set on canonical local `main`, publish the immutable candidate
to authority remote `main` with a leased proven fast-forward, fetch and query the
remote, prove local `main`, authority tracking ref, and observed remote `main`
agree, and remove every cycle-owned temporary resource. The final leaf handoff
records the reviewed planning revisions, exact task ranges, all 59 finding closure
source, six MR dispositions, artifact hashes, checks, reviews, candidate SHA,
remote readback, and root follow-up before FRI-07 planning. Blocker: none.
