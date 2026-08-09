# P01-I08-S01-C06R Inherited Placement Capacity Correction

Status: reviewed

Cycle ID: `P01/I08/S01/C06R`

Owning repository: `surgeist-layout`

Cycle base: `5504f2bb3eb8d79bff509077bcbc110858515a89`

Reviewed specification:
`plans/specs/P01-I08-grid-subgrid-and-grid-lanes-completeness.md`, normalized
semantic-content SHA-256
`a636dd9c9b896e2986fd13ab303f8506fba7eec6b0ba909e542eee9dc39770e6`,
commit `09bab4edc2bbff4aad42469937a328d0724989c0`: `FRI-08.5` decisions
`D-02`, `D-04`, `D-05`, and `D-21`; the ordinary and inherited placement,
error, verification, responsibility, finding-closure, handoff, and acceptance
contracts in `FRI-08.7`, `FRI-08.11`, `FRI-08.12`, and `FRI-08.14` through
`FRI-08.19`; `GRID-001`; and the standalone-axis portion of `GRID-010`.

Reviewed implementation sequence:
`plans/sequences/P01-I08-S01-grid-subgrid-and-grid-lanes-completeness.md`,
normalized semantic-content SHA-256
`bb61e73fe6fd4dd976b2a0ba0b5491bd77a91776a5052085b764a1bf169ded91`,
commit `8011b4c5a1ded53f8ded8578509efa5f26c8485d`, entry
`P01/I08/S01/C06R`.

Bounded outcome: make canonical placement demand the sole ordinary-grid owner
for local and inherited axes, reject inherited-axis capacity overflow through
the existing typed transaction-safe error path, remove the residual ordinary
estimator, preserve accepted geometry and the separate grid-lanes policy, and
publish the corrected leaf candidate without changing C06 artifacts.

## 1 Boundary

The remotely verified C06 candidate at the cycle base is immutable. It carries
1,448 HTML sources, 5,776 comment-free XML outputs, the exact 18-source/72-row
FRI-08 browser surface, and the sole schema-3 `all.json` with buckets 5,776
generated, 16 unsupported, three FRI-07 expected-fail, zero quarantined, and
zero failed-to-generate.

The superseded first C07 characterization established the behavior reopen:

- accepted one-inherited-axis sparse/dense row/column flows, spans, overlap,
  holes, leading/trailing demand, and f32/f64 geometry pass on the cycle base;
- an ordinary grid with one inherited four-track axis and an automatic
  five-track span incorrectly returns a completed batch and publishes a
  zero-size child; and
- `src/grid/mod.rs` routes an ordinary grid with either inherited axis around
  `derive_grid_placement_demand` into a child-count/span/`div_ceil` estimator,
  although `src/grid/placement.rs` already models inherited axes as bounded and
  maps excess capacity to the existing typed error envelope.

The cycle owns the exact GRID-001 placement/error correction composed through
GRID-010 standalone axes. Every non-grid-lanes ordinary grid, including grids
with both axes inherited, must consume the canonical integer placement-demand
result before collapse, sizing, and geometry materialization. Accepted requests
preserve their exact completed batch and cold/warm cache behavior. Inherited
capacity overflow on either axis returns
`LayoutErrorSiteOf::Node(container)`, `LayoutOperation::ChildLayout`, and
`LayoutErrorKindOf::InternalInvariant(InvalidBlockScrollGeometry)` with no
completed batch, partial publication, or cache mutation; a retry is
deterministic. Grid-lanes retains its distinct pre-sizing and auto-fit policy.

Out of scope: public API/type/error changes; topology, placement-algorithm,
track-sizing, named-line, area, subgrid-traversal, gutter-carrier, lanes-policy,
scroll, baseline, authored-CSS, adapter, generator, HTML/XML/report/manifest,
browser/generation, dependency, feature, lockfile, MSRV, docs, task-runner,
root/sibling, later FRI-09/F10 behavior, suppression, unsafe, and unrelated
cleanup. Stop before widening beyond the task files.

## 2 Impacts

Public API compatibility: internal-only; no signature, type, variant, reexport,
or feature changes. Observable behavior corrects the invalid inherited-capacity
success to the already-specified typed failure. Accepted geometry and all other
errors remain unchanged.

Dependencies, features, lockfile, MSRV, docs, examples, root integration, and
finding ownership: unchanged.

Generated artifacts and inputs: unchanged. No browser or generator command is
authorized. Frozen C06 SHA-256 values are:

- `corpus.toml`:
  `c6e6f1422e14a5e4aa474c143998063ce0de4d0a9123b69875b35a4ed009a8f6`;
- helper:
  `c684c7f167d95997a4a9f0250467bbaf72c1b73e69e0f707a2ef32f4d25f7f36`;
- sole `all.json`:
  `c10dc550d260a239c8bf9dd553f5272ca3bcc2826099bc182f800986b8b94c0e`;
- complete XML inventory:
  `a98d1ccceaeeb336ee3cb3c0151607589c0a4ae0376a46c560ba4341f95ad6ae`;
  and
- complete XML hash lineage:
  `bad8e418caee72cc62a123dc93efe89fdb07bfb5dee4345f3df7d8fd6fe44fdf`.

Owned Rust remains free of `unsafe`.

## 3 Task

### 3.1 `P01/I08/S01/C06R/T01` Enforce Canonical Inherited Placement Capacity

**Files/area:** `src/grid/mod.rs` for ordinary-versus-grid-lanes demand
orchestration and estimator removal; `src/grid/placement.rs` only to delete the
now-unreferenced `placement_cell_span` helper and helper-local tests, without
changing canonical placement behavior; `src/grid_tests.rs` for public-front-door
behavior, transaction, cache, scalar, transpose, and source-shape evidence.

**Outcome:** route every ordinary grid, including one- and two-inherited-axis
subgrids, through `derive_grid_placement_demand`, settled integer areas,
ordinary auto-fit collapse, and canonical track-demand application. Retain the
existing separate grid-lanes pre-sizing and auto-fit branch. Delete ordinary use
of `visible_cell_count`, `placement_cell_span`, `auto_fit_limit`, and `div_ceil`
from orchestration and delete the orphaned span-estimator helper without moving
or recreating the heuristic.

**RED evidence:** first add `fri08_c06r_inherited_placement_` public tests. The
accepted matrix covers one inherited column plus standalone row and the
transpose, both axes inherited, sparse/dense row/column flow, automatic spans
within capacity, definite overlap, holes, leading/trailing demand, and f32/f64;
it must pass on the task base and freeze exact geometry, completed batches, and
cold/warm cache equivalence. The overflow matrix uses four-track inherited axes
and automatic span five in both transposes, on either axis of a both-inherited
grid, and in both scalar lanes; it must fail on the task base because success
and zero geometry are observed where the exact typed error, no completed batch,
partial publication, or cache mutation, and deterministic retry are required.
A source-shape test must also fail because the ordinary inherited estimator and
orphan helper remain. Stop if an accepted characterization fails for any other
reason.

**Acceptance:** every accepted one- and both-inherited-axis characterization is
byte-for-byte equivalent after the edit, including exact successful geometry,
completed batches, and cold/warm cache equivalence in both scalar lanes. Every
inherited-capacity overflow on either axis returns the exact existing typed
error atomically and deterministically, including retry. No ordinary request
publishes sentinel zero geometry. Canonical demand receives inherited-axis
bounds and settles every ordinary in-flow area before collapse and sizing.
`src/grid/mod.rs`
contains no non-lanes child-count, total-cell-count, `placement_cell_span`,
`auto_fit_limit`, or `div_ceil` demand calculation, and no orphaned estimator
helper remains in `src/grid/placement.rs`. Grid-lanes retains its separate
pre-sizing and auto-fit policy. Existing C01 placement, C04 standalone, C06
gutter, exact 72-row parity, FRI-09/F10 controls, and all eight finding closures
remain green. No artifact/input/hash or public/dependency/feature delta exists.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri08_c06r_inherited_placement_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri08_c01_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri08_c04_standalone_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri08_c06_collapsed_gutter_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --test layout fri08_c06_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --test layout fri08_c0
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --features layout-golden-generate --all-targets -- -F unsafe-code -D warnings
cargo fmt --check
git diff --check
```

Also prove exact three-file implementation scope, frozen C06 artifact/input
hashes, no new `allow`/`expect`, and zero unsafe matches across every owned Rust
file.

**Dependency:** published C06 candidate, the reviewed C06R sequence entry, and
the stopped C07 characterization summarized in Section 1.

**Intended commit:** `fix(grid): reject inherited placement overflow`.

## 4 Completion

The canonical worker, task-review, status, final-check, holistic-review,
publication, readback, and cleanup lifecycle applies. C06R acceptance is:

1. canonical placement demand owns every ordinary grid, including inherited
   axes, and the residual estimator/helper is absent;
2. accepted one- and both-inherited-axis matrices preserve exact geometry,
   completed batches, and cold/warm cache behavior in all required flows and
   scalars;
3. inherited-capacity overflow on either inherited axis returns the existing
   typed error with atomic publication/cache semantics and deterministic retry;
4. GRID-001, composed GRID-010, all eight finding closures, C06 gutters, all 72
   owned rows, FRI-09/F10 controls, centralized provenance, and frozen artifacts
   remain correct;
5. default/generator verification, corpus/Taffy, strict Clippy, formatting,
   diff, scope, suppression, unsafe, and clean-worktree gates pass without a
   browser or generator invocation; and
6. the corrected candidate is published and remotely read back by exact SHA,
   after which a fresh full-range sprawl assessment runs before replacement C07
   planning.

No blocker is currently known.
