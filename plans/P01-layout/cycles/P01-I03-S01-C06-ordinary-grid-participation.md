# P01-I03-S01-C06 Ordinary-Grid Participation
Status: complete
Cycle ID: `P01/I03/S01/C06`
Owning repository: `surgeist-layout`
Cycle base: `001d8efda12f9672057a73bcd11c6e3178a8dd92`

Reviewed specification: `plans/P01-layout/initiatives/P01-I03-box-participation-contracts.md`
at `9482b43c7b3bed5355fa438a353c103625ff032a311a10b1a5c90c7e4f199d0b`,
commit `49ede2ba2672a91f99ba193651dbb1350ede7b80`, sections `FRI-03.2`,
`E-GRID-ORDER`, `E-GRID-REPLACED`, the ordinary-grid portions of `D-02` and
`D-05`, the ordinary-grid rows and cases in `FRI-03.6`, relevant `FRI-03.8`
and `FRI-03.9`, and acceptance items 2, 3, and 6.

Reviewed sequence: `plans/P01-layout/sequences/P01-I03-S01-box-participation-contracts.md`
at `db716f78093f71cc58daf3f1b889bce5687384948f8dbe0c22b1e2b533791518`,
commit `0a666f8f698703cd7979194a7f75f834e4c9b522`, entry `C06`.

C05 handoff: candidate `001d8efda12f9672057a73bcd11c6e3178a8dd92`
was pushed to and read back from `origin/main`; local, tracking, and observed
remote `main` were equal and clean. Flex participation is complete.

Bounded outcome: ordinary-grid order-sensitive placement traverses the stable
in-flow permutation while all storage and output identity remain source-indexed;
one replaced-aware normal-alignment resolver maps only absent/default alignment
to start and preserves explicit stretch.

## 1 Boundary
This cycle owns only ordinary-grid construction and consumption of the existing
`ItemOrder` permutation, ordinary-grid item alignment resolution, focused
real-tree tests, and read-only comparison of the four settled
`fri03_order_modified_grid` XML variants.

The writable implementation allowlist is `src/grid/mod.rs`,
`src/grid/placement.rs`, `src/grid/child.rs`, `src/grid_tests.rs`, and
`tests/layout/browser_parity.rs`. `src/grid/lanes.rs` remains unchanged for C07.

The placement context keeps `children`, item placements, computed areas,
subgrid reports, and final outputs source-indexed. It additionally carries one
permutation of visible in-flow source indexes formed with
`item_order_permutation`. Fully definite occupancy is marked independently of
that permutation. Definite-major and remaining auto-placement traverse it and
write each area back to the corresponding source slot. Row/column and
sparse/dense cursor semantics otherwise remain unchanged. Hidden and absolute
children are excluded from the permutation and keep their enumerated source
indexes.

The ordinary-grid child path owns a crate-private resolver that distinguishes an
explicit item/container alignment from the absent layout-ready `normal` state.
For an auto-sized replaced item, absent alignment resolves to `Start` on each
axis. Existing non-replaced normal rules stay intact, and explicit item or
container `Stretch` remains `Stretch`. The helper is reusable by C07, but this
cycle does not change a grid-lanes caller.

No HTML, XML, parser, generator, corpus metadata, or report changes are allowed,
and no generator or browser capture runs in this cycle. A read-only scoped
aggregate parity diagnostic reported four expected `x` mismatches across the
settled grid-order variants: LTR expected 40 and got 0; RTL expected 20 and got
60. That run is RED context only, not completion evidence.

Base evidence: `resolve_grid_child_areas` marks fully definite occupancy in
source order and both order-sensitive phases iterate `placements.items` in
source order. Ordinary-grid alignment currently defaults an auto-sized item to
stretch without consulting `item_is_replaced`.

Impacts: API - unchanged; dependencies/features and MSRV - unchanged; artifacts
- unchanged and read-only; docs/examples - unchanged; root follow-up - deferred
to C08; unsafe - none.

## 2 Task
### 2.1 `P01/I03/S01/C06/T01` - Enforce Ordinary-Grid Order And Replaced Normal Alignment
Files: `src/grid/mod.rs`, `src/grid/placement.rs`, `src/grid/child.rs`,
`src/grid_tests.rs`, and `tests/layout/browser_parity.rs`.
Dependencies: published/read-back C05 base and the existing C01 permutation.

Outcome: construct one stable permutation from visible in-flow grid children;
use it for definite-major and remaining placement without reindexing storage;
and use one ordinary-grid-owned resolver so replaced default/normal starts while
explicit stretch and existing non-replaced behavior remain intact.

RED:
- mixed fully definite, definite-major, and auto items are traversed in source
  order for row/column and sparse/dense placement;
- all four settled browser variants fail their first child's `x`; and
- an auto-sized replaced child with no explicit self/container alignment is
  stretched to its grid area instead of retaining its measured size at start.

Acceptance:
- both scalar lanes prove negative, zero, positive, and equal order values affect
  mixed-phase row/column and sparse/dense placement exactly once;
- fully definite occupancy remains order-independent, while definite-major and
  remaining placement consume the canonical permutation;
- areas, reports, final child order, and every `SourceIndex` remain source-aligned;
- hidden and absolute children consume no in-flow placement position;
- all four settled grid-order XML variants run nonignored, retain exact topology,
  and match without HTML or XML changes;
- replaced absent/default alignment resolves to start on both axes; non-replaced
  normal remains unchanged; explicit item stretch and explicit container stretch
  remain effective for replaced items; and
- no grid-lanes, block, flex, public API, parser, fixture, or generator branch
  changes.

Focused tests:
- `grid::tests::ordinary_grid_order_modified_placement_precedes_mixed_phases_and_preserves_source_identity_in_both_scalar_lanes`
- `grid::tests::ordinary_grid_replaced_normal_alignment_starts_while_explicit_stretch_remains_in_both_scalar_lanes`
- `layout::browser_parity::grid_item_order_variants_match_browser`

Inventory must match each exact name, and each execution must report one test:
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --lib -- --list | rg -x 'grid::tests::ordinary_grid_order_modified_placement_precedes_mixed_phases_and_preserves_source_identity_in_both_scalar_lanes: test'
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --lib grid::tests::ordinary_grid_order_modified_placement_precedes_mixed_phases_and_preserves_source_identity_in_both_scalar_lanes -- --exact
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --lib -- --list | rg -x 'grid::tests::ordinary_grid_replaced_normal_alignment_starts_while_explicit_stretch_remains_in_both_scalar_lanes: test'
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --lib grid::tests::ordinary_grid_replaced_normal_alignment_starts_while_explicit_stretch_remains_in_both_scalar_lanes -- --exact
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout -- --list | rg -x 'layout::browser_parity::grid_item_order_variants_match_browser: test'
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout layout::browser_parity::grid_item_order_variants_match_browser -- --exact
```

Task gates: the focused tests; `CARGO_NET_OFFLINE=true just fmt-check`;
`CARGO_NET_OFFLINE=true just verify`; `CARGO_NET_OFFLINE=true just verify-generator`;
`CARGO_NET_OFFLINE=true just corpus-check`; strict locked Clippy; rustdoc with
warnings denied; repository-wide unsafe absence; diff checks; exact changed-file
inspection; protected-path identity; and clean status. No generation or capture.

Commit: `layout: enforce ordinary-grid participation`.

## 3 Completion
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --lib -- --list | rg -x 'grid::tests::ordinary_grid_order_modified_placement_precedes_mixed_phases_and_preserves_source_identity_in_both_scalar_lanes: test'
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --lib grid::tests::ordinary_grid_order_modified_placement_precedes_mixed_phases_and_preserves_source_identity_in_both_scalar_lanes -- --exact
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --lib -- --list | rg -x 'grid::tests::ordinary_grid_replaced_normal_alignment_starts_while_explicit_stretch_remains_in_both_scalar_lanes: test'
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --lib grid::tests::ordinary_grid_replaced_normal_alignment_starts_while_explicit_stretch_remains_in_both_scalar_lanes -- --exact
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout -- --list | rg -x 'layout::browser_parity::grid_item_order_variants_match_browser: test'
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout layout::browser_parity::grid_item_order_variants_match_browser -- --exact
CARGO_NET_OFFLINE=true just fmt-check
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
CARGO_NET_OFFLINE=true RUSTDOCFLAGS='-D warnings' cargo doc --locked --offline --no-deps -p surgeist-layout
rg -n 'item_order_permutation|item_is_replaced' src/grid src/node_input.rs
! rg -n --pcre2 '#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{' --glob '*.rs' --glob '!target/**' .
git diff --check
git diff --name-only 001d8efda12f9672057a73bcd11c6e3178a8dd92
git diff --exit-code 001d8efda12f9672057a73bcd11c6e3178a8dd92 -- src/grid/lanes.rs
git diff --exit-code 001d8efda12f9672057a73bcd11c6e3178a8dd92 -- tests/layout/browser_parity/html tests/layout/browser_parity/xml
git diff --exit-code 001d8efda12f9672057a73bcd11c6e3178a8dd92 -- tests/bin tests/layout/browser_parity/support.rs
git status --short
```

Cycle acceptance: the task range is independently `CLEAN`; the complete cycle
range is holistic `CLEAN`; final commands pass on local `main`; the immutable
candidate is pushed to authority `origin/main`; a fresh fetch and remote query
prove local `HEAD`, local `main`, `origin/main`, and observed remote `main`
agree; and C07 receives the published SHA plus the canonical permutation and
replaced-aware normal-alignment helper.

Genuine blockers are limited to unavailable required tooling without authorized
acquisition, unowned dirty state, contradictory reviewed requirements, or a
required unsafe/ownership violation. A failing test or review finding returns
this plan to `in_progress` and is corrected inside C06.
