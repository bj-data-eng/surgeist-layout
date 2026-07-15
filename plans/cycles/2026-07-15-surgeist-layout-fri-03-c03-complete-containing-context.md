# FRI-03-C03 Complete Containing Context And Cache Identity
Status: in_progress
Cycle ID: `FRI-03-C03`
Owning repository: `surgeist-layout`
Cycle base: `127f20b4450e2196b768e78e0c97006e7ea0fc84`

Reviewed specification: `plans/specs/2026-07-15-surgeist-layout-fri-03-box-participation-contracts.md`
at `6ca195b4ba560ae49bc6963176234f8494cfb50a91674f6dcec358d19fa9769c`,
commit `52d87a75751f9987251ec2fdf8200e75eba3e17b`, sections `FRI-03.2`,
`E-PARENT-CONTEXT`, `D-03`, `FRI-03.5`, the `FRI-03.6` parent-context
matrix, relevant `FRI-03.7` through `FRI-03.9` and `FRI-03.11`, and
acceptance item 4.

Reviewed sequence: `plans/sequences/2026-07-15-surgeist-layout-fri-03-box-participation-contracts.md`
at `d59317e1b80337ff4041a034c062867dc7e744048eb7047d2b2e7b412aea130a`,
commit `03e7582565fa2d4f3aa7f71973f6dfebe273c4fb`, entry `C03`.

C02 handoff: candidate `127f20b4450e2196b768e78e0c97006e7ea0fc84`
was pushed to and read back from `origin/main`; local, tracking, and observed
remote `main` were equal and clean. Its 16 flex-item-root XML variants carry
strict parent axes, and the parser retains those axes.

Bounded outcome: one public resolved context value keeps containing flow and
parent formatting participation together through direct leaf input, recursive
algorithms, hidden layout, viewport/flex roots, and cache identity. Flex-item
roots use the explicit C02 parent axes rather than the item's axes.

## Boundary
This cycle owns public `ParentFormattingContext` and
`ContainingLayoutContext`, `ComputeInputOf` construction/accessors,
`CacheKeyOf`, algorithm-owned child contexts, hidden propagation, the
flex-item-root parent-axis API, parity consumer wiring, and focused tests.

`ParentFormattingContext` is exactly `NoParent`, `BlockFlow`, `Flex`, or
`Grid`; `Grid` includes grid-lanes scheduling. `ContainingLayoutContext` has
private flow/role fields, an infallible constructor, projections, and no
`Default`. No role is inferred from current `Display`, position, replacedness,
or measurement.

C04-C07 retain collapse, replacedness, and order consumption. C03 adds no
boolean, fallback, overload, alias, allocation, dependency, feature, module,
script, fixture, report, generated artifact, generator path, root/sibling edit,
or unsafe. The `support.rs` change only passes already-parsed axes to the new
consumer API; parser grammar and generator inputs do not change. No generator
command is permitted, and all XML/reports remain byte-identical to the base.

Base evidence: `ComputeInputOf` and `CacheKeyOf` store bare `FlowAxes`;
recursive callers cannot encode parent role; hidden recursion has no explicit
`NoParent`; `FlexItemRootContextOf::under_viewport` takes one argument; and
`compute_flex_item_root` substitutes item axes for parent axes.

Public impact: breaking pre-release complete-context leaf input, additive
context types, and the reviewed two-argument flex-root constructor. Everything
else named above is unchanged; root integration remains deferred to C08.

## Tasks
### C03-T1 - Public Resolved Context Types
Files: `src/output.rs`, `src/lib.rs`, `src/contract_tests.rs`.
Dependencies: published/read-back C02 base only.

Outcome: add the closed public enum and immutable public context with private
fields, exact constructor/projections, copy/debug/equality traits, docs,
reexports, no scalar parameter, and no default/conversion shortcut.

RED: both named tests fail to compile because the types do not exist.
Acceptance: all four roles and horizontal/vertical/sideways axes are covered;
compile-fail docs reject `Default`; no duplicate carrier or inference appears.

Commands: exact library list/run for
`contract_tests::parent_formatting_context_is_closed_and_exact` and
`contract_tests::containing_layout_context_keeps_flow_and_role_together`;
`CARGO_NET_OFFLINE=true just fmt-check`; `CARGO_NET_OFFLINE=true just verify`;
`git diff --check`.
Commit: `api: add complete containing layout context`.

### C03-T2 - Atomic Compute And Cache Migration
Files: `src/output.rs`, `src/cache.rs`, `src/compute.rs`, `src/block.rs`,
`src/flex.rs`, `src/grid/child.rs`, `src/grid/tracks.rs`,
`src/grid/lanes.rs`, `src/grid/subgrid.rs`, direct tests, and existing
test-support constructors.
Dependencies: C03-T1 task-review CLEAN.

Outcome: replace bare compute/cache axes with one complete context. Public leaf
and private child constructors take it; accessors expose context, role, and flow
projection. Viewport uses `NoParent`, block `BlockFlow`, flex `Flex`, and grid
plus lanes `Grid` for in-flow, intrinsic, sizing, layout, and absolute paths.
A hidden node keeps its supplied context and gives its descendants `NoParent`
with the same axes. Cache storage/manual comparison includes the whole value.
T2 may still use item axes for the flex-root `Flex` context; T3 immediately
removes that final base limitation once the root carrier gains parent axes.

RED: complete-input, role-only cache, and algorithm-capture tests fail before
the migration. Acceptance: role-only requests miss in both scalar lanes;
cold/warm size, content size, baselines, and collapse state agree; one role is
used across every run mode; no bare-flow overload, fallback, or boolean remains.

Commands: exact library list/run for
`contract_tests::compute_input_requires_complete_containing_layout_context`,
`cache_tests::cache_misses_for_parent_formatting_context_only_in_both_scalar_lanes`,
`block_tests::block_child_context_is_complete_for_layout_sizing_and_absolute_paths`,
`flex_tests::flex_child_context_is_complete_for_layout_sizing_and_absolute_paths`,
`grid_tests::grid_and_lanes_child_context_is_complete_for_layout_sizing_and_absolute_paths`,
and `root_tests::root_and_hidden_contexts_are_explicit_in_both_scalar_lanes`;
`just fmt-check`; `just verify`; `just verify-generator`; `just corpus-check`;
`git diff --check`, all with `CARGO_NET_OFFLINE=true` where applicable.
Commit: `layout: propagate parent formatting context`.

### C03-T3 - Explicit Flex-Parent Axes
Files: `src/output.rs`, `src/compute.rs`, `src/root_tests.rs`, affected contract
tests, and `tests/layout/browser_parity/support.rs`.
Dependencies: C03-T2 task-review CLEAN.

Outcome: `FlexItemRootContextOf<S>` stores scalar-independent parent axes; its
sole constructor is `under_viewport(viewport_available, parent_flow_axes)`.
Flex-root percentage/logical resolution and cache identity use
`ContainingLayoutContext::new(parent_flow_axes, Flex)`; item axes still schedule
descendants. The parity consumer passes the strict C02 axes.

RED: non-square orthogonal roots currently use item axes and the two-argument
contract is absent. Acceptance: both scalar lanes prove parent/item disagreement,
percentage/logical edges, role/axis cache misses, cold/warm equivalence, viewport
`NoParent`, flex-root `Flex`, and existing 5,268 XML parsing. Compile-fail docs
reject one-argument root and bare-flow leaf calls. No artifact changes.

Commands: exact library list/run for
`contract_tests::flex_item_root_context_requires_explicit_parent_axes` and
`root_tests::flex_item_root_uses_explicit_parent_axes_for_percentage_and_cache_in_both_scalar_lanes`;
exact layout-test list/run for
`layout::browser_parity::support::tests::viewport_parent_axes_schema_is_strict`;
`just fmt-check`; `just verify`; `just verify-generator`; `just corpus-check`;
`git diff --check`, all with `CARGO_NET_OFFLINE=true` where applicable.
Commit: `layout: require flex parent axes at root`.

## Exact Command Shapes
For each task's exact library name, run both commands separately:
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --lib -- --list | rg -x 'TEST_NAME: test'
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --lib TEST_NAME -- --exact
```
For the parity-support name:
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout -- --list | rg -x 'layout::browser_parity::support::tests::viewport_parent_axes_schema_is_strict: test'
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout layout::browser_parity::support::tests::viewport_parent_axes_schema_is_strict -- --exact
```

## Completion
```sh
CARGO_NET_OFFLINE=true just fmt-check
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
CARGO_NET_OFFLINE=true RUSTDOCFLAGS='-D warnings' cargo doc --locked --offline --no-deps -p surgeist-layout
rg -n 'ParentFormattingContext|ContainingLayoutContext' src/lib.rs src/output.rs
! rg -n 'containing_flow_axes: FlowAxes' src/output.rs src/cache.rs
! rg -n 'is_flex_or_grid_item|parent_is_flex|parent_is_grid' src
git diff --check
git diff --exit-code 127f20b4450e2196b768e78e0c97006e7ea0fc84 -- Cargo.toml Cargo.lock Justfile README.md scripts tests/bin/surgeist-layout-generate tests/layout/browser_parity/html tests/layout/browser_parity/xml tests/layout/browser_parity/corpus.toml tests/layout/browser_parity/README.md tests/layout/browser_parity/scripts
test -z "$(git status --porcelain)"
```
Also run the canonical owned-Rust unsafe scan; it must find no executable unsafe.

Acceptance: every task and holistic review is CLEAN; all gates pass; context is
complete and cached; explicit flex-parent axes work; generated state is unchanged;
local `main` is published and read back exactly. C04 may consume only the role
for boundary collapse. No root handoff is emitted before C08.

Blockers: any generator/schema/fixture/artifact need, new role, display inference,
external acquisition, unowned source change, unsafe, or geometry beyond the
reviewed flex-root correction. Revise/review the plan; do not regenerate.
