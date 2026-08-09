# P01-I08-S01-C02R Grid-Lanes Track Phase Correction

Status: reviewed

Cycle ID: `P01/I08/S01/C02R`

Owning repository: `surgeist-layout`

Cycle base: `828f0204fc60393bc97d0ac9777eefdbdcb169cd`

Reviewed specification:
`plans/specs/P01-I08-grid-subgrid-and-grid-lanes-completeness.md`, normalized
semantic-content SHA-256
`a636dd9c9b896e2986fd13ab303f8506fba7eec6b0ba909e542eee9dc39770e6`,
commit `09bab4edc2bbff4aad42469937a328d0724989c0`: `FRI-08.5` decisions
`D-09` through `D-11` and `D-21`; track-sizing and lanes portions of
`FRI-08.8`, `FRI-08.10`, `FRI-08.12`, and `FRI-08.14` through `FRI-08.17`;
`GRID-003`, `GRID-007`, and the lanes sizing composition of `GRID-002` and
`GRID-010`; and final acceptance in `FRI-08.19`.

Reviewed implementation sequence:
`plans/sequences/P01-I08-S01-grid-subgrid-and-grid-lanes-completeness.md`,
normalized semantic-content SHA-256
`b8efd9c6ac235fa167fa06d80c8155b2d494187dac46c06f28985de77b02cbe9`,
commit `8778ae9487bc1880786a9710b00b7865755d8462`, entry
`P01/I08/S01/C02R`.

Bounded outcome: make the canonical per-track base/growth-limit/fit-content/
flex/auto-max phases the sole final track-sizing owner for ordinary grid and
grid-lanes, correct mixed lanes fit-content/flex and auto-max stretch behavior,
preserve the distinct lanes placement/auto-fit/intrinsic-projection policies,
and publish the corrected leaf candidate without changing C06 artifacts.

## 1 Boundary

The remotely verified C06R candidate at the cycle base is immutable. It closes
the inherited-capacity correction and carries 1,448 HTML sources, 5,776
comment-free XML outputs, the exact 18-source/72-row FRI-08 browser surface, and
the sole schema-3 `all.json` with buckets 5,776 generated, 16 unsupported, three
FRI-07 expected-fail, zero quarantined, and zero failed-to-generate.

The mandatory post-C06R full-range assessment returned exactly one residual:

- `src/grid/tracks.rs::resolve_lanes_inline_tracks` returns a collection-wide
  `resolve_fit_content_tracks` result when any track has a fit-content maximum,
  skipping flexible and stretch phases;
- the lanes block and inline final sizing helpers count stretch eligibility only
  for exact `min:auto/max:auto`, duplicating and narrowing the canonical
  maximum-is-`Auto` predicate; and
- `src/grid_tests.rs` freezes the contrary mixed lanes result `[30,0]` while
  ordinary-grid tests already prove the specified phase-composed behavior.

This is the exact reopened C02 sizing owner, not C07 mechanical consolidation.
Final track resolution uses one policy-free `GridTrackSizingPhases` route in
`src/grid/mod.rs`. That route consumes the already-settled lanes-specific
intrinsic contributions and gutter/collapse facts but delegates final base,
growth-limit, fit-content, flexible, and auto-maximum stretch phases to
`resolve_inline_tracks` and `resolve_tracks_with_gutters`. The
`GridTrackSizingPolicy` discriminator and lanes-only final sizing helpers are
removed. Lanes placement, pre-placement auto-fit collapse, hybrid containing
blocks, candidate projection, nested-subgrid flattening, and contribution
formation remain distinct and unchanged.

The public behavior oracles are exact:

- grid-lanes `[fit-content(50px),1fr]` in a definite `200px` axis with settled
  intrinsic bases `[20,0]` resolves to `[20,180]`;
- the existing `fit-content(20px)` plus `1fr` case in `100px` with intrinsic
  bases `[30,0]` resolves to `[30,70]`, preserving the minimum-content floor;
- one `minmax(0,auto)` lane track in a definite `100px` axis resolves to `100px`
  under normal/stretch; intrinsic-minimum/auto variants preserve their floor and
  receive the same positive remainder; and
- non-auto maxima, collapsed tracks, non-stretch alignment, indefinite space,
  and non-positive remainder receive no auto-max stretch.

Out of scope: placement, occupancy, lanes auto-fit, containing blocks,
intrinsic candidate/descendant projection, topology, names, areas, subgrid
traversal, gutter policy/carriers, child layout, scroll, baselines, public API/
types/errors/reexports, authored CSS, adapter, helper, generator, HTML/XML/
report/manifest, browser/generation, dependencies, features, lockfile, MSRV,
docs, task runner, root/sibling work, later FRI-09/F10 behavior, suppression,
unsafe, and unrelated cleanup. Stop before widening beyond the task files.

## 2 Impacts

Public API compatibility: internal-only; no signatures, types, variants,
reexports, defaults, or features change. Observable grid-lanes sizing is
corrected only for the specified mixed fit-content/flex and auto-maximum stretch
cases. Distinct lanes placement and auto-fit behavior is unchanged.

Dependencies, features, lockfile, MSRV, docs, examples, root integration, and
finding ownership: unchanged.

Generated artifacts and inputs: unchanged. No browser or generator command is
authorized. Frozen C06/C06R SHA-256 values are:

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

### 3.1 `P01/I08/S01/C02R/T01` Unify Final Grid Track Phases

**Files/area:** `src/grid/mod.rs` for a policy-free final track-phase router and
removal of the ordinary-versus-lanes sizing discriminator; `src/grid/tracks.rs`
for canonical per-track phase reuse and deletion of lanes-only final sizing and
fit-content shortcut helpers; `src/grid_tests.rs` for public-front-door behavior,
scalar/axis/cache controls, exact negative controls, and declared architecture
evidence.

**Outcome:** both ordinary grid and grid-lanes use the existing canonical
`resolve_inline_tracks` and `resolve_tracks_with_gutters` final phase model after
their policy-specific placement, collapse, and intrinsic contribution inputs
have settled. Retain `GridTrackSizingPhases` only as a policy-free iterative
phase carrier already consumed by subsequent sizing passes. Delete
`GridTrackSizingPolicy`, the lanes-only collection-wide fit-content branch,
`resolve_lanes_inline_tracks`, `resolve_lanes_tracks_with_intrinsics`,
`resolve_lanes_tracks_with_gutters`, and `resolve_fit_content_tracks` when no
production caller remains. Adapt helper-local nested fit-content calculation
tests to exercise the canonical phase owner rather than retaining an orphan.

**RED evidence:** first prove the existing lanes auto-fit, containing-block,
projection, nested-subgrid, cache/error, and 72-row controls pass unchanged on
the task base. Then add `fri08_c02r_lanes_track_phase_` public tests covering the
two exact fit-content/flex oracles and auto-max stretch eligibility/exclusion in
columns and rows, horizontal/vertical/sideways projection, normal/stretch/start
alignment, definite/indefinite/non-positive remainder, f32/f64, successful
completed batches, and cold/warm cache equivalence. They must fail on the task
base at the observable `[30,0]`/zero-stretch geometry. Add source-shape evidence
only for the declared `FRI-08.14(5)`, `(6)`, and `(14)` contract; it must fail on
the collection-wide shortcut, exact-auto predicate, and alternate final sizing
owner. Structural evidence supplements rather than substitutes for behavior.

**Acceptance:** the exact public behavior oracles in Section 1 pass in both
axes and scalars, including cache-equivalent completed batches. Ordinary-grid
track results remain byte-for-byte equivalent. Grid-lanes auto-fit, placement,
containing-block, intrinsic projection, nested-subgrid, gutter, and error/
transaction behavior remain exact. One canonical final phase model owns
fit-content, flex, growth limits, and maximum-is-`Auto` stretch; no lanes-only
collection-wide fit-content return, exact-auto stretch predicate, alternate
final sizing state machine, or orphan helper remains. Existing C02, C03, C05
composition, C06 gutter, C06R placement, exact 72-row parity, and FRI-09/F10
controls remain green. No artifact/input/hash or public/dependency/feature delta
exists.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri08_c02r_lanes_track_phase_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri08_c02_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri08_c03_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri08_c05_composition_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri08_c06_collapsed_gutter_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri08_c06r_inherited_placement_
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

Also prove exact three-file implementation scope, frozen artifact/input hashes,
no new `allow`/`expect`, and zero unsafe matches across every owned Rust file.

**Dependency:** published C06R candidate, reviewed C02R sequence entry, and the
single post-C06R sprawl finding recorded in Section 1.

**Intended commit:** `fix(grid): unify lanes track phases`.

## 4 Completion

The canonical worker, task-review, status, final-check, holistic-review,
publication, readback, and cleanup lifecycle applies. C02R acceptance is:

1. one policy-free final track-phase owner covers ordinary grid and grid-lanes,
   with no alternate fit-content/flex/stretch state machine;
2. mixed lanes fit-content/flex and auto-maximum stretch produce the exact
   required geometry in all axes/scalars/cache states;
3. distinct lanes placement, auto-fit, containing-block, projection, and nested
   subgrid policies remain unchanged;
4. all eight finding closures, C06 gutters, C06R inherited placement, all 72
   owned rows, FRI-09/F10 controls, centralized provenance, and frozen artifacts
   remain correct;
5. default/generator verification, corpus/Taffy, strict Clippy, formatting,
   diff, scope, suppression, unsafe, and clean-worktree gates pass without a
   browser or generator invocation; and
6. the corrected candidate is published and remotely read back by exact SHA,
   after which a fresh full-range sprawl assessment runs before replacement C07
   planning.

No blocker is currently known.
