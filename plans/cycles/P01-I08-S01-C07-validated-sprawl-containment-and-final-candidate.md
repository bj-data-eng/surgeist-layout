# P01-I08-S01-C07 Validated Sprawl Containment And Final Candidate

Status: in_progress

Cycle ID: `P01/I08/S01/C07`

Owning repository: `surgeist-layout`

Cycle base: `5504f2bb3eb8d79bff509077bcbc110858515a89`

Reviewed specification:
`plans/specs/P01-I08-grid-subgrid-and-grid-lanes-completeness.md`, normalized
semantic-content SHA-256
`a636dd9c9b896e2986fd13ab303f8506fba7eec6b0ba909e542eee9dc39770e6`,
commit `09bab4edc2bbff4aad42469937a328d0724989c0`: `FRI-08.14` through
`FRI-08.19`.

Reviewed implementation sequence:
`plans/sequences/P01-I08-S01-grid-subgrid-and-grid-lanes-completeness.md`,
normalized semantic-content SHA-256
`5c0cadc59d5aea8863c1038a7f131b2549f527fd0e74a18f2bac468974b20324`,
commit `dfb7768c68def72b19e08199b91ef65914e12a9a`, entry
`P01/I08/S01/C07`.

Bounded outcome: validate and remove the one confirmed residual ordinary-grid
placement-demand estimator identified by the required full-range sprawl
assessment, preserve behavior and all C06 artifacts, and publish the final
FRI-08 leaf candidate.

## 1 Boundary

The remotely verified C06 candidate at the cycle base is immutable. Local
`main`, `origin/main`, and remote `main` equal the base; the worktree is clean.
It closes all eight FRI-08 findings and carries 1,448 HTML sources, 5,776
comment-free XML outputs, the exact 18-source/72-row FRI-08 browser surface, and
the sole schema-3 `all.json` with buckets 5,776 generated, 16 unsupported, three
FRI-07 expected-fail, zero quarantined, and zero failed-to-generate.

The mandatory fresh holistic sprawl assessment reviewed the exact initiative
range
`238df34a713db4f90d7f194f6fdf89a994d34fa2..5504f2bb3eb8d79bff509077bcbc110858515a89`
and returned one finite finding:

| ID | Disposition |
| --- | --- |
| `SP-001` | Confirmed residual responsibility duplication, subject to behavior-preserving source validation. `src/grid/mod.rs` routes an ordinary grid with either inherited axis away from canonical `derive_grid_placement_demand`, then rebuilds demand with `visible_cell_count`, `placement_cell_span`, `prepend_auto_tracks`, and `div_ceil`. `src/grid/placement.rs` already models inherited axes as bounded/non-growable. This violates `FRI-08.14(1)`, `(2)`, `(3)`, and `(14)` while the distinct grid-lanes branch remains legitimate. |

C07 owns only validation and mechanical removal of that non-lanes estimator.
Before editing production, characterize one-inherited-axis ordinary grids through
the public front door for row/column flow, sparse/dense automatic placement,
spans, overlap and holes, leading/trailing demand, both scalars, and typed
inherited-capacity failure. Retain the published standalone-axis flow, intrinsic,
cache, rollback, and parity controls. If canonical routing changes accepted
geometry or error semantics, stop without implementing the consolidation and
reopen the exact C01/C04 behavior owner; do not relabel a behavior correction as
sprawl containment.

Out of scope: any other architecture search or consolidation; grid-lanes policy;
topology, placement algorithm, track sizing, names, areas, subgrid traversal,
gutter carriers, public API/types/errors/reexports, docs, fixtures, helper,
adapter, generator, HTML/XML/report/manifest, browser/generation, dependencies,
features, lockfile, MSRV, task runner, root/sibling work, later-owned FRI-09/F10
behavior, suppression, unsafe, and unrelated cleanup.

## 2 Impacts

Public API and observable behavior: unchanged. Production structure changes only
by routing ordinary inherited-axis grids through the existing canonical demand
owner and deleting the duplicate estimator branch. Grid-lanes retains its named
separate policy.

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

### 3.1 `P01/I08/S01/C07/T01` Remove The Residual Ordinary Demand Estimator

**Files/area:** `src/grid/mod.rs` for ordinary-versus-grid-lanes demand
orchestration and estimator removal; `src/grid/placement.rs` only to delete the
now-unreferenced `placement_cell_span` helper and its helper-local tests, without
changing canonical demand behavior; focused characterization and structural
evidence in `src/grid_tests.rs`. Stop before widening.

**Outcome:** route every ordinary grid, including one- and two-inherited-axis
subgrids, through `derive_grid_placement_demand`, settled areas, ordinary
auto-fit collapse, and canonical track-demand application. Retain the existing
separate grid-lanes pre-sizing and auto-fit branch. Delete ordinary use of
`visible_cell_count`, `placement_cell_span`, `auto_fit_limit`, and `div_ceil`
from orchestration and delete the orphaned span-estimator helper without moving
or recreating the heuristic.

**Pre-change characterization and structural RED:** first add
`fri08_c07_inherited_demand_` public tests for one inherited column with
standalone row and the transposed case, row/column sparse and dense flow,
automatic spans, definite overlap, holes, leading/trailing implicit demand,
f32/f64, and an item exceeding inherited capacity. They must pass on the task
base with exact geometry and typed error/rollback evidence. Add a focused source
shape test that fails because the ordinary inherited branch still contains the
four forbidden estimator operations. If any characterization fails before the
production edit, stop and diagnose rather than change behavior.

**Acceptance:** every characterization remains byte-for-byte equivalent after
the edit. The canonical demand owner receives inherited-axis facts and returns
settled integer areas; bounded inherited overflow retains its typed failure with
no partial publication/cache mutation. `src/grid/mod.rs` contains no non-lanes
child-count, total-cell-count, `placement_cell_span`, `auto_fit_limit`, or
`div_ceil` demand calculation, and no orphaned estimator helper remains in
`src/grid/placement.rs`. Grid-lanes retains its separate pre-sizing and auto-fit
policy. Existing C01 placement, C02 sizing/auto-fit, C04 standalone,
C06 gutter, 72-row parity, FRI-09/F10 controls, and all eight finding closures
remain green. No artifact/input/hash or public/dependency/feature delta exists.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri08_c07_inherited_demand_
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

Also prove exact three-file scope, frozen C06 artifact/input hashes, no new
`allow`/`expect`, and zero unsafe matches across every owned Rust file.

**Dependency:** published C06 candidate and the complete `SP-001` assessment in
Section 1.

**Intended commit:** `refactor(grid): unify inherited placement demand`.

## 4 Completion

The canonical worker, task-review, status, final-check, holistic-review,
publication, readback, and cleanup lifecycle applies. C07 acceptance is:

1. `SP-001` has a confirmed, source-validated disposition and the duplicate
   non-lanes estimator is absent;
2. one canonical placement-demand owner covers ordinary and inherited ordinary
   grids, while grid-lanes retains its distinct named policy;
3. all `FRI-08.14` structural invariants and eight finding closures hold;
4. public behavior, typed errors/rollback, all 72 owned rows, FRI-09/F10
   controls, centralized provenance, and frozen artifacts/inputs are unchanged;
5. default/generator verification, corpus/Taffy, strict Clippy, formatting,
   diff, scope, suppression, unsafe, and clean-worktree gates pass without a
   browser or generator invocation; and
6. the final FRI-08 candidate is published and remotely read back by exact SHA
   with complete leaf/root and later-P01 handoff evidence.

No blocker is currently known.
