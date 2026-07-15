# FRI-03-C01 Exact Order And Source Identity

Status: complete
Cycle ID: `FRI-03-C01`
Owning repository: `surgeist-layout`
Cycle base: `05401beb53853a5eaf1c622050cfa0d7cebc0c4c`

Reviewed specification: `plans/specs/2026-07-15-surgeist-layout-fri-03-box-participation-contracts.md`
at `56efbca03febc725bee2d829da9bfdcf45f6194b24555eb22c1aa1082d9b12f2`,
commit `ad342c4526802460f89d6d02125f16e419b6f81b`, sections `FRI-03.1`,
order/source portions of `FRI-03.2`, `D-01`, helper ownership in `D-02`, `D-06`,
`FRI-03.5`, relevant `FRI-03.7`, `FRI-03.9`, and acceptance items 1 and 2.
Reviewed sequence: `plans/sequences/2026-07-15-surgeist-layout-fri-03-box-participation-contracts.md`
at `4ad6f9ffca47d7119c487da8be09f03bdebae269debd2a20dd502323ee43bdd2`,
commit `05401beb53853a5eaf1c622050cfa0d7cebc0c4c`, entry `C01`.

Bounded outcome: public scalar-independent `ItemOrder` and `SourceIndex` types
replace absent input and ambiguous output identity; one crate-private stable
`(item_order, source_index)` permutation is ready for later algorithms without
changing default geometry.

## Boundary

This cycle owns `src/node_input.rs`, `src/output.rs`, `src/lib.rs`, source-index
carrier renames in compute/block/inline/flex/grid/grid-lanes/subgrid code, and
focused tests proving source identity and permutation behavior. It may adjust
tests that construct or assert `NodeOutputOf::with_order`.

It must not make flex, ordinary-grid, or grid-lanes consume non-default
`ItemOrder` for geometry; that remains `C05` through `C07`. It must not change
fixtures, generator/parser/schema, reports, cache context, parent participation,
replaced sizing, dependencies, features, MSRV, root, siblings, or API artifacts.
No compatibility alias, old overload, scalar order conversion, software
acquisition, or `unsafe` is allowed.

Current evidence: `NodeInputOf` has no order value; `NodeOutputOf::order`,
`with_order`, and private `order: u32` carriers represent source sibling ordinal;
block, inline, hidden, flex, grid, lanes, subgrid, rounded, and batch outputs use
source traversal naming that conflicts with CSS order.
`Justfile` owns the locked default and generator verification matrices; all
recipes run with `CARGO_NET_OFFLINE=true` in this cycle.

## Impacts

Public API: breaking pre-release addition of `NodeInputOf::item_order` and
replacement of `NodeOutputOf::order`/`with_order` with `source_index`/
`with_source_index`. Additive public types: `ItemOrder` and `SourceIndex`.
Dependencies/features/artifacts/docs/MSRV/root: unchanged in this cycle. Unsafe:
owned Rust remains unsafe-free.

## Tasks

### C01-T1 - Public Order And Source Types

Files: `src/node_input.rs`, `src/output.rs`, `src/lib.rs`, and public contract
tests.
Outcome: `ItemOrder(i32)` and `SourceIndex(usize)` expose `ZERO`, `new`, `get`,
derive copy/debug/equality/ordering/hash traits, and are reexported. `NodeInputOf`
defaults `item_order` to `ItemOrder::ZERO`. The legacy output API remains only
until T2 so this task is independently buildable; it is not a compatibility
commitment.
RED: add `contract_tests::public_order_source_types_and_defaults_are_exact` for
default/min/zero/max order, source-index construction, reexports, and
`NodeInputOf` default order; it fails on the base because the types and field
are missing.
Acceptance: the exact new test and existing f64-lane test each execute once and
pass; no primitive conversion or scalar-generic order value is added.
Commands: first two exact focused-test list/run gates below;
`CARGO_NET_OFFLINE=true just verify`.
Intended commit: `api: distinguish item order from source index`.

### C01-T2 - Rename Source Identity Carriers

Files: `src/output.rs`, `src/compute.rs`, `src/block.rs`, `src/inline.rs`,
`src/flex.rs`, `src/grid/child.rs`, `src/grid/lanes.rs`,
`src/grid/subgrid.rs`, and affected tests.
Outcome: `NodeOutputOf` replaces `order`/`with_order` with
`source_index`/`with_source_index`, and every source sibling ordinal carrier
and output write uses `source_index` naming and either `SourceIndex` or
`usize`. Hidden, root, standalone, rounded, final, unrounded, grid, subgrid,
inline, and direct outputs preserve base source identity for default order.
RED: add `contract_tests::node_output_source_index_is_unambiguous` and
`root_tests::source_index_identity_survives_root_hidden_rounding_and_batch`;
they fail before the atomic output/call-site migration.
Acceptance: no production `order: u32`, source-ordinal `order` accessor, or
`with_order` call remains; existing block/flex/grid/root/inline behavior remains
source-identical for default order.
Commands: third and fourth exact focused-test list/run gates below; C01
legacy-name absence gate; `CARGO_NET_OFFLINE=true just verify`.
Intended commit: `layout: rename source identity carriers`.

### C01-T3 - Stable Permutation And Absence Gates

Files: helper and focused tests in `src/node_input.rs`; block non-consumption
test in `src/block_tests.rs`.
Outcome: one crate-private helper sorts in-flow `(ItemOrder, SourceIndex)` pairs
by ascending signed order then ascending source index without subtraction; all
zero/default input returns source order. It does not traverse trees or alter any
algorithm geometry in this cycle.
RED: add `node_input::tests::item_order_permutation_is_signed_total_and_stable`;
it fails before the helper exists. Characterization:
`block_tests::block_layout_ignores_item_order_for_geometry` passes before and
after the helper and proves the unchanged block boundary.
Acceptance: both exact tests execute once and pass; negative, positive, equal,
default, `i32::MIN`, and `i32::MAX` cases are covered; batch node/source
identity is preserved; absence gates prove old names are gone and production
geometry has no item-order consumer yet.
Commands: last two exact focused-test list/run gates and C01
absence/non-consumption gates below; `CARGO_NET_OFFLINE=true just verify`;
`CARGO_NET_OFFLINE=true just verify-generator`; `git diff --check`.
Intended commit: `layout: add stable item-order permutation`.

### Exact Focused-Test Gates

```sh
bash -lc 'CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --lib -- --list | rg -x "contract_tests::public_order_source_types_and_defaults_are_exact: test"'
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --lib contract_tests::public_order_source_types_and_defaults_are_exact -- --exact
bash -lc 'CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --lib -- --list | rg -x "contract_tests::node_input_and_output_support_f64_scalar_lane: test"'
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --lib contract_tests::node_input_and_output_support_f64_scalar_lane -- --exact
bash -lc 'CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --lib -- --list | rg -x "contract_tests::node_output_source_index_is_unambiguous: test"'
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --lib contract_tests::node_output_source_index_is_unambiguous -- --exact
bash -lc 'CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --lib -- --list | rg -x "root_tests::source_index_identity_survives_root_hidden_rounding_and_batch: test"'
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --lib root_tests::source_index_identity_survives_root_hidden_rounding_and_batch -- --exact
bash -lc 'CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --lib -- --list | rg -x "node_input::tests::item_order_permutation_is_signed_total_and_stable: test"'
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --lib node_input::tests::item_order_permutation_is_signed_total_and_stable -- --exact
bash -lc 'CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --lib -- --list | rg -x "block_tests::block_layout_ignores_item_order_for_geometry: test"'
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --lib block_tests::block_layout_ignores_item_order_for_geometry -- --exact
```

### C01 Exact Absence Gates

```sh
bash -lc 'if rg -n -e "with_order|\\.order\\b|order: u32|order: usize|fn order\\(|pub order" src; then exit 1; else rc=$?; test "$rc" -eq 1; fi'
bash -lc 'for term in ItemOrder SourceIndex item_order source_index with_source_index; do rg -q "\\b$term\\b" src/lib.rs src/node_input.rs src/output.rs || exit 1; done'
bash -lc 'if rg -n -e "ItemOrderOf|SourceIndexOf|impl From<.*ItemOrder|impl From<.*SourceIndex" src; then exit 1; else rc=$?; test "$rc" -eq 1; fi'
bash -lc 'actual=$(rg -l "\\bitem_order\\b" src --glob "*.rs" --glob "!*_tests.rs" --glob "!contract_tests.rs" | LC_ALL=C sort); test "$actual" = "src/node_input.rs"'
bash -lc 'if rg -n "\\.item_order\\b" src --glob "*.rs" --glob "!*_tests.rs" --glob "!contract_tests.rs"; then exit 1; else rc=$?; test "$rc" -eq 1; fi'
```

## Completion

Cycle acceptance: C01-T1 through C01-T3 are implemented, reviewed, and
source-clean; default/source-order geometry is unchanged; old source-order names
are absent; new public types and fields match `FRI-03`; later algorithm
consumption, fixture schema, parent context, collapse, and replaced behavior
remain unclaimed.

Final commands:

```sh
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
git diff --check
bash -lc 'for test in contract_tests::public_order_source_types_and_defaults_are_exact contract_tests::node_input_and_output_support_f64_scalar_lane contract_tests::node_output_source_index_is_unambiguous root_tests::source_index_identity_survives_root_hidden_rounding_and_batch node_input::tests::item_order_permutation_is_signed_total_and_stable block_tests::block_layout_ignores_item_order_for_geometry; do CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --lib -- --list | rg -x "$test: test" || exit 1; CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --lib "$test" -- --exact || exit 1; done'
bash -lc 'if rg -n -e "with_order|\\.order\\b|order: u32|order: usize|fn order\\(|pub order" src; then exit 1; else rc=$?; test "$rc" -eq 1; fi'
bash -lc 'for term in ItemOrder SourceIndex item_order source_index with_source_index; do rg -q "\\b$term\\b" src/lib.rs src/node_input.rs src/output.rs || exit 1; done'
bash -lc 'if rg -n -e "ItemOrderOf|SourceIndexOf|impl From<.*ItemOrder|impl From<.*SourceIndex" src; then exit 1; else rc=$?; test "$rc" -eq 1; fi'
bash -lc 'actual=$(rg -l "\\bitem_order\\b" src --glob "*.rs" --glob "!*_tests.rs" --glob "!contract_tests.rs" | LC_ALL=C sort); test "$actual" = "src/node_input.rs"'
bash -lc 'if rg -n "\\.item_order\\b" src --glob "*.rs" --glob "!*_tests.rs" --glob "!contract_tests.rs"; then exit 1; else rc=$?; test "$rc" -eq 1; fi'
bash -lc 'files=$(git ls-files "*.rs"; git ls-files --others --exclude-standard "*.rs"); test -n "$files"; printf "%s\n" "$files" | xargs rg -n --pcre2 '\''#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{'\''; rc=$?; test "$rc" -eq 1'
```

Required handoff: after publication of the complete initiative, C02 may consume
`ItemOrder` in fixture parser/serializer work and later algorithm cycles may
consume the stable permutation. No root handoff is emitted from this cycle alone.

Genuine blocker: if a default-order geometry change is required to complete this
cycle, stop and revise/review the plan because algorithm consumption belongs to
later cycles.
