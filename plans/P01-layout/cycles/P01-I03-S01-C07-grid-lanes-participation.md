# P01-I03-S01-C07 Grid-Lanes Participation
Status: complete
Cycle ID: `P01/I03/S01/C07`
Owning repository: `surgeist-layout`
Cycle base: `574cb4526a1d92b5d94b656cafd90c0dc35fc107`

Reviewed specification: `plans/P01-layout/initiatives/P01-I03-box-participation-contracts.md`
at `9482b43c7b3bed5355fa438a353c103625ff032a311a10b1a5c90c7e4f199d0b`,
commit `49ede2ba2672a91f99ba193651dbb1350ede7b80`, sections `FRI-03.2`,
`E-LANES-ORDER`, `E-GRID-REPLACED`, the grid-lanes portions of `D-02` and
`D-05`, the grid-lanes rows and cases in `FRI-03.6`, relevant `FRI-03.8` and
`FRI-03.9`, and acceptance items 2, 3, and 6.

Reviewed sequence: `plans/P01-layout/sequences/P01-I03-S01-box-participation-contracts.md`
at `db716f78093f71cc58daf3f1b889bce5687384948f8dbe0c22b1e2b533791518`,
commit `0a666f8f698703cd7979194a7f75f834e4c9b522`, entry `C07`.

C06 handoff: candidate `574cb4526a1d92b5d94b656cafd90c0dc35fc107`
was pushed to and read back from `origin/main`; local, tracking, and observed
remote `main` were equal and clean. `GridPlacementContext` carries one stable
in-flow permutation, and ordinary grid owns the replaced-aware normal-alignment
resolver.

Bounded outcome: grid-lanes production placement and sequential intrinsic
contributions consume that exact permutation, while pre-placement grid-axis
measurement reuses the replaced-aware normal-alignment resolver.

## 1 Boundary
This cycle owns only production grid-lanes iteration and pre-placement sizing in
`src/grid/lanes.rs`, focused real-tree tests in `src/grid_tests.rs`, and read-only
comparison of the four settled `fri03_order_modified_lanes` XML variants in
`tests/layout/browser_parity.rs`.

The writable implementation allowlist is exactly those three files. C06's
`src/grid/mod.rs`, `src/grid/placement.rs`, and `src/grid/child.rs` contracts are
consumed unchanged. Block, flex, ordinary grid, subgrid storage, and public lane
utilities remain unchanged.

Production running-offset placement traverses
`GridPlacementContext::order_modified_indexes`, reads each child and placement by
source slot, and keeps final layout/output lookup source-indexed. Production
intrinsic collection traverses the same indexes before sequential definite-span
contributions are applied, so measurement and final placement cannot disagree.
Display-none and absolute children remain outside both in-flow traversals.

The public caller-ordered `place_lanes`, `lane_intrinsic_sizing`, `LaneItemOf`,
and report types gain no CSS-order field and retain their current behavior. Only
the production adapter supplies order-modified inputs.

Pre-placement grid-axis measurement calls the existing
`resolve_grid_item_normal_alignment`. For an auto-sized replaced item, absent
item/container alignment resolves to `Start`; non-replaced normal and explicit
item or container `Stretch` retain their current behavior on either grid axis.

No HTML, XML, parser, generator, corpus metadata, or report changes are allowed,
and no generator or browser capture runs in this cycle. A read-only scoped
aggregate parity diagnostic reported four expected `x` mismatches across the
settled grid-lanes variants: LTR expected 40 and got 0; RTL expected 20 and got
60. That run is RED context only, not completion evidence.

Base evidence: `resolve_grid_lanes_placement_with_resolved_tracks` and
`lane_intrinsic_track_sizes` enumerate `checked_child_placements` in source
order. `measure_lane_axis_margin_box_with_grid_axis` independently defaults
absent self/container alignment to `Stretch` without consulting replacedness.

Impacts: API - unchanged; dependencies/features and MSRV - unchanged; artifacts
- unchanged and read-only; docs/examples - unchanged; root follow-up - deferred
to C08; unsafe - none.

## 2 Task
### 2.1 `P01/I03/S01/C07/T01` - Enforce Grid-Lanes Order And Replaced Pre-Placement Alignment
Files: `src/grid/lanes.rs`, `src/grid_tests.rs`, and
`tests/layout/browser_parity.rs`.
Dependencies: published/read-back C06 base, its stored permutation, and its
normal-alignment resolver.

Outcome: use one order-modified production sequence for running offsets and
sequential intrinsic contributions, then resolve replaced normal alignment in
pre-placement measurement without changing caller-ordered public utilities.

RED:
- running offsets and overlapping definite-span intrinsic contributions follow
  source order rather than signed/equal item order;
- all four settled browser variants fail their first child's `x`; and
- replaced normal pre-placement measurement injects the full grid-axis span as
  known size instead of retaining measured size at start.

Acceptance:
- both scalar lanes prove negative, zero, positive, and equal order values drive
  production running offsets and preserve equal-order source ties;
- overlapping definite-span intrinsic contributions prove sequential sizing
  consumes the same permutation as final placement;
- reports, subgrid facts, final child order, and every `SourceIndex` remain
  source-associated, while hidden and absolute children consume no lane slot;
- caller-ordered `place_lanes` and `lane_intrinsic_sizing` characterization stays
  green and no public order field is added;
- replaced absent/default alignment injects no grid-axis known size;
  non-replaced normal, explicit item stretch, and explicit container stretch
  remain effective for both possible grid axes and both scalar lanes;
- all four settled grid-lanes order XML variants run nonignored, retain exact
  topology, and match without HTML or XML changes; and
- no ordinary-grid, block, flex, public API, parser, fixture, or generator branch
  changes.

Focused tests:
- `grid::tests::grid_lanes_order_modified_sequence_drives_running_offsets_and_intrinsic_contributions_in_both_scalar_lanes`
- `grid::tests::grid_lanes_replaced_normal_preplacement_starts_while_explicit_stretch_remains_in_both_scalar_lanes`
- `layout::browser_parity::grid_lanes_item_order_variants_match_browser`

Inventory must match each exact name, and each execution must report one test:
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --lib -- --list | rg -x 'grid::tests::grid_lanes_order_modified_sequence_drives_running_offsets_and_intrinsic_contributions_in_both_scalar_lanes: test'
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --lib grid::tests::grid_lanes_order_modified_sequence_drives_running_offsets_and_intrinsic_contributions_in_both_scalar_lanes -- --exact
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --lib -- --list | rg -x 'grid::tests::grid_lanes_replaced_normal_preplacement_starts_while_explicit_stretch_remains_in_both_scalar_lanes: test'
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --lib grid::tests::grid_lanes_replaced_normal_preplacement_starts_while_explicit_stretch_remains_in_both_scalar_lanes -- --exact
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout -- --list | rg -x 'layout::browser_parity::grid_lanes_item_order_variants_match_browser: test'
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout layout::browser_parity::grid_lanes_item_order_variants_match_browser -- --exact
```

Task gates: focused tests; `CARGO_NET_OFFLINE=true just fmt-check`;
`CARGO_NET_OFFLINE=true just verify`; `CARGO_NET_OFFLINE=true just verify-generator`;
`CARGO_NET_OFFLINE=true just corpus-check`; strict locked Clippy; rustdoc with
warnings denied; repository-wide unsafe absence; diff checks; exact changed-file
inspection; protected-path identity; and clean status. No generation or capture.

Commit: `layout: enforce grid-lanes participation`.

## 3 Completion
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --lib -- --list | rg -x 'grid::tests::grid_lanes_order_modified_sequence_drives_running_offsets_and_intrinsic_contributions_in_both_scalar_lanes: test'
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --lib grid::tests::grid_lanes_order_modified_sequence_drives_running_offsets_and_intrinsic_contributions_in_both_scalar_lanes -- --exact
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --lib -- --list | rg -x 'grid::tests::grid_lanes_replaced_normal_preplacement_starts_while_explicit_stretch_remains_in_both_scalar_lanes: test'
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --lib grid::tests::grid_lanes_replaced_normal_preplacement_starts_while_explicit_stretch_remains_in_both_scalar_lanes -- --exact
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout -- --list | rg -x 'layout::browser_parity::grid_lanes_item_order_variants_match_browser: test'
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout layout::browser_parity::grid_lanes_item_order_variants_match_browser -- --exact
CARGO_NET_OFFLINE=true just fmt-check
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
CARGO_NET_OFFLINE=true RUSTDOCFLAGS='-D warnings' cargo doc --locked --offline --no-deps -p surgeist-layout
rg -n 'order_modified_indexes|resolve_grid_item_normal_alignment' src/grid
! rg -n --pcre2 '#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{' --glob '*.rs' --glob '!target/**' .
git diff --check
git diff --name-only 574cb4526a1d92b5d94b656cafd90c0dc35fc107
git diff --exit-code 574cb4526a1d92b5d94b656cafd90c0dc35fc107 -- src/grid/mod.rs src/grid/placement.rs src/grid/child.rs
git diff --exit-code 574cb4526a1d92b5d94b656cafd90c0dc35fc107 -- tests/layout/browser_parity/html tests/layout/browser_parity/xml
git diff --exit-code 574cb4526a1d92b5d94b656cafd90c0dc35fc107 -- tests/bin tests/layout/browser_parity/support.rs
git status --short
```

Cycle acceptance: the task range is independently `CLEAN`; the complete cycle
range is holistic `CLEAN`; final commands pass on local `main`; the immutable
candidate is pushed to authority `origin/main`; a fresh fetch and remote query
prove local `HEAD`, local `main`, `origin/main`, and observed remote `main`
agree; and C08 receives the published SHA with all FRI-03 algorithm consumers
complete.

Genuine blockers are limited to unavailable required tooling without authorized
acquisition, unowned dirty state, contradictory reviewed requirements, or a
required unsafe/ownership violation. A failing test or review finding returns
this plan to `in_progress` and is corrected inside C07.
