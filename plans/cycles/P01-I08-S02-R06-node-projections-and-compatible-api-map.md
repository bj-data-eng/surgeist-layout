# P01/I08/S02/R06 Node Projections And Compatible Public API Map

Cycle ID: `P01/I08/S02/R06`

Owning repository: `surgeist-layout`

Status: `in_progress`

Cycle base: `05a531dd661937aa3518678524c9accb0a99063d`

Specification: `plans/specs/P01-I08-grid-subgrid-and-grid-lanes-completeness.md`,
reviewed semantic SHA-256
`d9c6a61eae363331d7e8ce05d88916099111e11b8793b8dc31cc55e3e5c80a6a`,
commit `b9cb82aadf70d5690d605bb9ffeaa6da9512bd3d`, sections `FRI-08.20`
row `AR-005`, `FRI-08.21`, `FRI-08.25`, `FRI-08.27`, and acceptance
rows `FRI-08.28(1)`, `FRI-08.28(6)`, and `FRI-08.28(8)` through
`FRI-08.28(10)`.

Sequence: `plans/sequences/P01-I08-S02-architectural-remediation.md`,
reviewed semantic SHA-256
`46d3563226ba6b91478bdc0b36273abb56644720774804b7c7a2ab9d0ca07251`,
commit `2f097f4b9ac510df63e3e886e2f7a46f0312a701`, entry
`P01/I08/S02/R06`.

Bounded outcome: algorithms construct role-specific crate-private projections
from borrowed `NodeInputOf<S>` at settled entry boundaries, the public node-input
types have semantic source owners, and one README API map documents the unchanged
root facade.

## 1 Boundary And Impacts

R03 through R05 are published and remotely verified, so the engine, scroll,
block, flex, grid-track, and grid-child phase boundaries are stable. The public
`NodeInputOf<S>` remains the sole normalized aggregate with its exact 55 public
fields, aliases, constants, defaults, constructors, and role behavior. A
projection is crate-private, borrowed where cloning is unnecessary, and created
only after the algorithm role is known. It is total for irrelevant fields and
may fail only for an already-defined invalid role combination, preserving the
same `LayoutErrorOf` site, operation, payload, and transaction behavior.

`CommonBoxProjection` contains only semantics genuinely shared by two or more
algorithms: validated size/min/max/aspect facts, margin/padding/border, box
sizing, flow axes, overflow, positioning/insets, and replaced/table facts.
Algorithm-specific alignment, flex, grid, inline, float, scroll, and ordering
facts remain in their role owner. Block owns container/child projections, flex
owns container/item projections, grid owns container/item projections, inline
owns participant projection, and scroll owns box/target projections. Core phase
functions consume those projections once the role is settled; direct
`NodeInputOf` access remains only at public/tree input lookup and projection
construction boundaries.

The `node_input` implementation becomes a private module tree with semantic
public-type families and a composition-only `mod.rs`; it does not add public
module paths. `lib.rs` root reexports, public names, signatures, default scalar
aliases, field shape, and compile contracts remain exact. Existing source-proxy
tests may receive only minimal path aggregation or recursive inventory updates;
no source/token/file-placement test is added or strengthened. R08 still removes
that entire test class.

T02 adds `scripts/audit-node-projection-boundaries.sh` as a workflow-only audit,
never a cargo test. It has fixed modes `scroll`, `block-inline`, `flex`,
`grid-container`, `grid`, and `all`. The only allowed complete-input owners in
algorithm trees are `src/{block,flex,grid,scroll,inline}/input.rs`; the shared
constructor owner is `src/node_projection.rs`. Fixed selected paths are: every
non-input `src/scroll/*.rs` for `scroll`; every non-input `src/block/*.rs` plus
`src/inline/mod.rs` for `block-inline`; every non-input `src/flex/*.rs` for
`flex`; `src/grid/topology.rs` plus non-input
`src/grid/tracks/{mod,validation,ordinary,flexible}.rs` for `grid-container`;
every non-input Rust file below `src/grid/` for `grid`; and the union for `all`.
The script uses a standard-library lexical scanner, not raw grep: it masks line
and nested block comments, ordinary/byte/raw strings, chars, and every balanced
item gated by exact `cfg(test)` before inspecting production tokens. Its
mandatory `--self-test` covers nested delimiters in literals/comments plus
test-gated `use`, function, and module items, and proves an adjacent production
violation plus an extracted UFCS lookup binding followed by invocation remain
visible. In each selected production token stream it rejects
every qualified or unqualified `NodeInput`/`NodeInputOf` token, every
`LayoutInput`/`LayoutInputOf` token, and every `node_input` identifier;
this covers dot, associated-function, UFCS, borrowed binding, direct field,
method-item extraction, default-scalar, variant extraction, and local alias spellings. Input owners are
separately rejected if they type-alias or reexport either complete aggregate;
they may only consume it to construct projections. `grid-container` is the
intentionally narrow pre-item stage; T06 explicitly owns `grid/mod.rs`, lanes,
subgrid, placement/named/axis, intrinsic/subgrid tracks, and every child path,
then `grid` closes the entire grid tree. The script has no line/text allowlist
and exits nonzero after printing every violation. T02 through T06 run the mode
for the boundary they close; T06 then proves their union, and T07 and final
acceptance repeat `all`.

Public API classification: source-compatible documentation/internal ownership
change only. Dependencies, features, Cargo files, MSRV, root integration, and
generated API artifacts: unchanged. Documentation: README gains one grouped API
map for computation/tree, node input, sizing, geometry/scroll, output, and finite
grid utilities. Generated artifacts remain frozen: corpus
`c6e6f1422e14a5e4aa474c143998063ce0de4d0a9123b69875b35a4ed009a8f6`,
helper `c684c7f167d95997a4a9f0250467bbaf72c1b73e69e0f707a2ef32f4d25f7f36`,
`all.json` `c10dc550d260a239c8bf9dd553f5272ca3bcc2826099bc182f800986b8b94c0e`,
1,448 HTML, and 5,776 comment-free XML. Surgeist-owned code remains free of
`unsafe` and new lint suppressions.

Out of scope: FRI-09; public facade/model redesign; nested public property
groups, builders, preludes, compatibility aliases, or duplicate module trees;
new errors or validation; algorithm behavior changes; R07 test-suite movement;
R08 test-conformance deletion; root/sibling work; browser execution, generation,
acquisition, generator/helper/manifest/report/XML changes; `cargo clean` before
publication.

## 2 Tasks

### 2.1 `P01/I08/S02/R06/T01` Partition Public Node-Input Type Ownership

**Area:** replace `src/node_input.rs` with private
`src/node_input/{mod,box_model,scroll,inline,alignment,flex,grid}.rs`, preserve
`src/lib.rs`, exact embedded doctests/unit tests, and make only required legacy
recursive-inventory/path adaptations in `src/lib_tests.rs` or
`src/contract_tests.rs`.

**Outcome:** `mod.rs` owns `NodeInputOf`, `LayoutInputOf`, defaults, non-box
construction, and intentional private reexports. Each other file owns its
semantic public family; `ScrollbarWidthOf` moves with scroll and `ItemOrder`
moves with shared alignment/order facts. Every existing root public name and
construction form remains exact.

**RED/acceptance:** `node_input_`, `computed_overflow_`,
`fri06_mr01_non_box_inline_text_`, `grid_template_area`, and the public API inventory pass nonzero before and
after. The external ownership probe is RED because the directory is absent and
GREEN only with all seven owners, absent legacy file, and singular
`NodeInputOf`. Full doctests prove public construction and compile-fail contracts.

```sh
set -e; for f in node_input_ computed_overflow_ fri06_mr01_non_box_inline_text_ grid_template_area fri08_remediation_public_api_inventory_is_compatible; do CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout "$f"; done
for p in src/node_input/{mod,box_model,scroll,inline,alignment,flex,grid}.rs; do test -f "$p"; done; test ! -e src/node_input.rs; test "$(rg -l 'pub struct NodeInputOf' src/node_input/*.rs | wc -l | tr -d ' ')" = 1
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout; CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --doc; CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings; cargo fmt --check; git diff --check
```

Dependency: none. Commit: `refactor(input): partition public type ownership`.

### 2.2 `P01/I08/S02/R06/T02` Common Box And Scroll Projections

**Area:** new `src/node_projection.rs`, new `src/scroll/input.rs`, new
`scripts/audit-node-projection-boundaries.sh`,
`src/scroll/mod.rs`, construction/model/box-geometry/contribution callers,
algorithm scroll-publication entry callers, focused scroll tests, and exact
production inventory additions only.

**Outcome:** one `CommonBoxProjection` borrows shared box facts; scroll owns
distinct `ScrollBoxProjection` and `ScrollTargetProjection`. Canonical scroll
source construction and target contribution consume projections rather than the
full public property bag, with no duplicate geometry or resolver.

**RED/acceptance:** `canonical_geometry_`, `scroll_projection_`,
`fri08_c07_t02_scroll_source_`, and `fri05_c01_scroll_input_` pass before and after. The
external probe is RED until both new owners exist and is GREEN with singular
projection declarations and no production `NodeInputOf` parameter in
`src/scroll/{box_geometry,construction,contribution}.rs`. Preserve all flow
mappings, clips, gutters, ranges, snap metadata, errors, and both scalar lanes.

```sh
set -e; for f in canonical_geometry_ scroll_projection_ fri08_c07_t02_scroll_source_ fri05_c01_scroll_input_; do CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout "$f"; done
test -f src/node_projection.rs; test -f src/scroll/input.rs; test "$(rg -l 'struct CommonBoxProjection' src --glob '*.rs' | wc -l | tr -d ' ')" = 1; test "$(rg -l 'struct ScrollBoxProjection' src/scroll --glob '*.rs' | wc -l | tr -d ' ')" = 1; test "$(rg -l 'struct ScrollTargetProjection' src/scroll --glob '*.rs' | wc -l | tr -d ' ')" = 1
scripts/audit-node-projection-boundaries.sh --self-test
scripts/audit-node-projection-boundaries.sh scroll
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout; CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings; cargo fmt --check; git diff --check
```

Dependency: T01. Commit: `refactor(input): add common and scroll projections`.

### 2.3 `P01/I08/S02/R06/T03` Block And Inline Role Projections

**Area:** new `src/block/input.rs`, `src/block/{mod,in_flow,inline_run,floats,
absolute,sizing,scroll}.rs`, replace `src/inline.rs` with
`src/inline/{mod,input}.rs`, focused block/inline tests, and exact production
inventory additions only. `src/contract_tests.rs` may change only its two
existing `include_str!("inline.rs")` owner paths to `inline/mod.rs`; test bodies,
assertions, and source-proxy semantics remain byte-identical otherwise.

**Outcome:** block owns `BlockContainerProjection` and `BlockChildProjection`;
inline owns `InlineParticipantProjection`. Entry/child lookup constructs them
from borrowed public input. Settled in-flow, float, inline-run, absolute, sizing,
and publication phases no longer pass a complete `NodeInputOf`.

**RED/acceptance:** `ordinary_block_flow_`, `block_atomic_inline_run_`,
`block_absolute_child_`, `fri06_c04_float_`, and `inline_layout_` pass before and
after. The external probe is RED until the three projection owners exist and
GREEN with one declaration each. Preserve margin collapse, BFC/float exclusion,
inline participation, errors, caches, scroll geometry, and scalar lanes.

```sh
set -e; for f in ordinary_block_flow_ block_atomic_inline_run_ block_absolute_child_ fri06_c04_float_ inline_layout_; do CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout "$f"; done
test -f src/block/input.rs; test -f src/inline/mod.rs; test -f src/inline/input.rs; test ! -e src/inline.rs; test "$(rg -l 'struct BlockContainerProjection' src/block --glob '*.rs' | wc -l | tr -d ' ')" = 1; test "$(rg -l 'struct BlockChildProjection' src/block --glob '*.rs' | wc -l | tr -d ' ')" = 1; test "$(rg -l 'struct InlineParticipantProjection' src/inline --glob '*.rs' | wc -l | tr -d ' ')" = 1
scripts/audit-node-projection-boundaries.sh block-inline
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout; CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings; cargo fmt --check; git diff --check
```

Dependency: T02. Commit: `refactor(block): consume role projections`.

### 2.4 `P01/I08/S02/R06/T04` Flex Container And Item Projections

**Area:** new `src/flex/input.rs`, all `src/flex/*.rs`, focused flex tests, and
exact production inventory additions only.

**Outcome:** flex constructs `FlexContainerProjection` at its entry and
`FlexItemProjection` at collection. Item collection, basis/automatic minimum,
line resolution, alignment, intrinsic, absolute, and scroll phases consume the
settled projection rather than the full public input.

**RED/acceptance:** `flex_order_modified_sequence_`,
`flex_replaced_automatic_minimum_`, `flex_row_aligns_`,
`fri07_c02_collapsed_output_`, and `fri05_c04_flex_contribution_` pass before and
after. The external probe is RED until both types have one owner. Preserve order,
collapse struts, sizing, wrapping, baselines, positioned children, errors,
caches/publication, and both scalar lanes.

```sh
set -e; for f in flex_order_modified_sequence_ flex_replaced_automatic_minimum_ flex_row_aligns_ fri07_c02_collapsed_output_ fri05_c04_flex_contribution_; do CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout "$f"; done
test -f src/flex/input.rs; test "$(rg -l 'struct FlexContainerProjection' src/flex --glob '*.rs' | wc -l | tr -d ' ')" = 1; test "$(rg -l 'struct FlexItemProjection' src/flex --glob '*.rs' | wc -l | tr -d ' ')" = 1
scripts/audit-node-projection-boundaries.sh flex
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout; CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings; cargo fmt --check; git diff --check
```

Dependency: T03. Commit: `refactor(flex): consume role projections`.

### 2.5 `P01/I08/S02/R06/T05` Grid Container Projection

**Area:** new `src/grid/input.rs`, `src/grid/topology.rs`,
`src/grid/tracks/{mod,validation,ordinary,flexible}.rs`, minimum constructor
wiring in `src/grid/mod.rs`, focused grid tests, and exact production inventory
additions only.

**Outcome:** one `GridContainerProjection` carries container-only topology,
track, area, auto-flow/tolerance, alignment, gap, sizing, flow, and scroll facts.
The fixed topology plus validation/ordinary/flexible track phase set consumes it;
broad ordinary-grid, lanes, subgrid, intrinsic, and child entry paths are
explicitly deferred to T06 rather than claimed by this task.

**RED/acceptance:** `fri08_c01_topology_`, `grid_fraction_tracks_`,
`grid_stretch_`, `grid_lanes_`, and `fri08_c02_auto_fit_` pass before and after.
The external probe is RED until the projection has one owner. Preserve placement
before sizing, named areas/lines, auto-repeat/collapse, lanes axis choice,
subgrid inheritance, errors, and scalar lanes.

```sh
set -e; for f in fri08_c01_topology_ grid_fraction_tracks_ grid_stretch_ grid_lanes_ fri08_c02_auto_fit_; do CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout "$f"; done
test -f src/grid/input.rs; test "$(rg -l 'struct GridContainerProjection' src/grid --glob '*.rs' | wc -l | tr -d ' ')" = 1
scripts/audit-node-projection-boundaries.sh grid-container
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout; CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings; cargo fmt --check; git diff --check
```

Dependency: T04. Commit: `refactor(grid): project container input`.

### 2.6 `P01/I08/S02/R06/T06` Complete Grid Container And Item Projection

**Area:** `src/grid/input.rs`, every deferred `src/grid/{mod,lanes,subgrid,axis,
placement,named}.rs` path, all remaining `src/grid/tracks/*.rs`, every
`src/grid/child/*.rs`, focused grid/subgrid tests, and exact owner-path
aggregation only when a legacy source proxy requires it.

**Outcome:** finish `GridContainerProjection` consumption across the deferred
ordinary-grid, lanes, subgrid, intrinsic, and child paths. `GridItemProjection`
owns item placement/raw placement, order,
replaced/table/positioned state, item alignment, sizing/box/flow, baseline,
overflow, and scroll facts. Collection constructs it once per settled child and
track-intrinsic, child, baseline, absolute, subgrid, and lanes phases consume it.

**RED/acceptance:** `grid_absolute_child_`, `subgrid_child_`,
`fri08_c04_baseline_`, `fri05_c05_grid_contribution_`, and
`fri08_c03_nested_` pass before and after. The external probe is RED until the
item projection is singular and core child/track signatures use it; final
`all` mode proves the complete block/flex/grid/inline/scroll union. Preserve
source identity/order, placement, inherited axes, sizing/alignment, baseline
transport, scroll publication, atomic failure, and both scalar lanes.

```sh
set -e; for f in grid_absolute_child_ subgrid_child_ fri08_c04_baseline_ fri05_c05_grid_contribution_ fri08_c03_nested_; do CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout "$f"; done
test "$(rg -l 'struct GridItemProjection' src/grid --glob '*.rs' | wc -l | tr -d ' ')" = 1
scripts/audit-node-projection-boundaries.sh grid
scripts/audit-node-projection-boundaries.sh all
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout; CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings; cargo fmt --check; git diff --check
```

Dependency: T05. Commit: `refactor(grid): project item input`.

### 2.7 `P01/I08/S02/R06/T07` Compatible README API Map And Final Inventory

**Area:** `README.md`, public/compile-contract documentation in `src/lib.rs` only
when needed, exact existing public and production inventory adaptations in
`src/lib_tests.rs`, and projection behavior tests only when they exercise a real
crate boundary.

**Outcome:** add one authoritative README API map grouped exactly as
computation/tree, node input, sizing, geometry/scroll, output, and finite grid
utilities. Every entry links an existing root name; the map creates no public
path. Reconcile the final source inventory and prove root facade/name/signature,
`NodeInputOf` field shape/defaults, compile contracts, and docs remain compatible.

**RED/acceptance:** README-map workflow probe is RED before the map exists and
GREEN with all six groups. `fri08_remediation_public_api_inventory_is_compatible`,
public construction tests, doctests, and full algorithm/browser-free suites pass.
No new cargo test reads Rust source, README, plans, Git, files, modules, symbols,
or counts; existing temporary source audits are not strengthened.

```sh
set -e; rg -q '^## Public API Map$' README.md; for h in 'Computation and tree' 'Node input' 'Sizing' 'Geometry and scroll' 'Output' 'Finite grid utilities'; do rg -q "^### $h$" README.md; done
scripts/audit-node-projection-boundaries.sh all
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri08_remediation_public_api_inventory_is_compatible; CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout node_input_; CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --doc
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout; CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings; cargo fmt --check; git diff --check
```

Dependency: T06. Commit: `docs(api): map compatible layout facade`.

## 3 Completion

R06 requires seven independently CLEAN task ranges, status `complete`, a GREEN
final matrix, CLEAN holistic review, publication/readback, process hygiene,
successful repository-root `cargo clean`, absent `target/`, and an immutable R07
handoff. Browser execution, generation, acquisition, and artifact writes remain
prohibited. Any public/API/default/error/geometry/artifact drift, added source-
parsing test, residual settled-phase property-bag dependency, unsafe match, or
unreviewed scope expansion is a stop.

```sh
set -e
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout node_input_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout compute_layout_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --doc
CARGO_NET_OFFLINE=true just verify
source_cache=/Users/codex/Development/surgeist/crates/surgeist-layout/target/surgeist-sources/taffy/d1ff7e339b9ee35b33858779f8d7653197e93d92
destination_cache=/Users/codex/Development/surgeist-layout/target/surgeist-sources/taffy/d1ff7e339b9ee35b33858779f8d7653197e93d92
test -d "$source_cache"; test "$(git -C "$source_cache" rev-parse HEAD)" = d1ff7e339b9ee35b33858779f8d7653197e93d92; test ! -e "$destination_cache"
mkdir -p "$(dirname "$destination_cache")"; cp -R "$source_cache" "$destination_cache"; test "$(git -C "$destination_cache" rev-parse HEAD)" = d1ff7e339b9ee35b33858779f8d7653197e93d92
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --features layout-golden-generate --all-targets -- -F unsafe-code -D warnings
cargo fmt --check; git diff --check
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri08_remediation_public_api_inventory_is_compatible
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri05_c05_grid_legacy_absence_inventories_every_production_source
rg -q '^## Public API Map$' README.md; for h in 'Computation and tree' 'Node input' 'Sizing' 'Geometry and scroll' 'Output' 'Finite grid utilities'; do rg -q "^### $h$" README.md; done
scripts/audit-node-projection-boundaries.sh all
scripts/audit-node-projection-boundaries.sh --self-test
: "${TASK_SPANS:?set TASK_SPANS to the newline-delimited ordered exact full-SHA spans from the seven CLEAN task reviews}"
expected_paths="$({ printf '%s\n' plans/cycles/P01-I08-S02-R06-node-projections-and-compatible-api-map.md; while IFS= read -r span; do git diff --name-only "$span"; done <<< "$TASK_SPANS"; } | LC_ALL=C sort -u)"; actual_paths="$(git diff --name-only 05a531dd661937aa3518678524c9accb0a99063d..HEAD | LC_ALL=C sort -u)"; test "$actual_paths" = "$expected_paths"
base_suppressions="$(while IFS= read -r p; do git show "05a531dd661937aa3518678524c9accb0a99063d:$p" | perl -0777 -ne 'while (/^[ \t]*#\s*\[\s*(?:allow|expect|cfg_attr)\b[^\]]*\]/gms) { $m=$&; $m=~s/\s+/ /g; print "$m\n" }'; done < <(git ls-tree -r --name-only 05a531dd661937aa3518678524c9accb0a99063d | rg '\.rs$') | LC_ALL=C sort)"; current_suppressions="$({ git ls-files -z -- '*.rs'; git ls-files -z --others --exclude-standard -- '*.rs'; } | sort -zu | xargs -0 perl -0777 -ne 'while (/^[ \t]*#\s*\[\s*(?:allow|expect|cfg_attr)\b[^\]]*\]/gms) { $m=$&; $m=~s/\s+/ /g; print "$m\n" }' | LC_ALL=C sort)"; test -z "$(comm -13 <(printf '%s\n' "$base_suppressions") <(printf '%s\n' "$current_suppressions"))"
if { git ls-files -z -- '*.rs'; git ls-files -z --others --exclude-standard -- '*.rs'; } | sort -zu | xargs -0 rg -n --pcre2 '#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{'; then exit 1; fi
test "$(shasum -a 256 tests/layout/browser_parity/corpus.toml | awk '{print $1}')" = c6e6f1422e14a5e4aa474c143998063ce0de4d0a9123b69875b35a4ed009a8f6
test "$(shasum -a 256 tests/layout/browser_parity/scripts/gentest/test_helper.js | awk '{print $1}')" = c684c7f167d95997a4a9f0250467bbaf72c1b73e69e0f707a2ef32f4d25f7f36
test "$(shasum -a 256 tests/layout/browser_parity/xml/generation-reports/all.json | awk '{print $1}')" = c10dc550d260a239c8bf9dd553f5272ca3bcc2826099bc182f800986b8b94c0e
test "$(find tests/layout/browser_parity/html -type f -name '*.html' | wc -l | tr -d ' ')" = 1448; test "$(find tests/layout/browser_parity/xml -type f -name '*.xml' | wc -l | tr -d ' ')" = 5776; test -z "$(git status --porcelain=v1)"
```

After publication/readback, prove no cycle-owned layout Cargo/Rust/generator
process remains; run `cargo clean`; prove `target/` absent and Git clean. Record
the published SHA, reviewed revisions, seven ordered task ranges and verdicts,
public compatibility, README map, frozen artifacts, remote readback, cleanup,
and the R07 handoff.
