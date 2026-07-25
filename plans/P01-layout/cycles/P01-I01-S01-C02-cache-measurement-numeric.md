# P01-I01-S01-C02 Cache, Measurement, And Numeric Property Contracts

Status: complete

Cycle ID: `P01/I01/S01/C02`

Owning repository: `surgeist-layout`

Cycle base: `a23282e4d9e3545c1a4ed5ea84cc1a0a6659324b`

Reviewed specification:
`plans/P01-layout/initiatives/P01-I01-compute-resolution-diagnostics.md`
at `38263f35b0e9782db12e28088e97e36a7f953db6af22a26b110cd004c8da51f9`
sections `FRI-01.3`, `D-06`, `D-07`, `D-08`, `FRI-01.8`,
`FRI-01.9`, numeric/cache/measurement portions of `FRI-01.12`,
`FRI-01.16`, `FRI-01.18`, and `FRI-01.19`.

Reviewed sequence:
`plans/P01-layout/sequences/P01-I01-S01-compute-resolution-diagnostics.md`
at `307ef2c0c6446a107e42f7a81d4e474dcde56fc509a4b4ed3cc76b1b82b89dbb`,
entry `P01/I01/S01/C02`.

Bounded outcome: cache storage, direct leaf measurement, and raw scrollbar/flex
numeric properties move to layout-owned validated contracts while the recursive
tree compute/session/batch/error front door remains for `P01/I01/S01/C03`.

## 1 Boundary

This cycle owns `src/cache.rs`, `src/compute.rs`, `src/node_input.rs`,
`src/lib.rs`, direct consumers of `scrollbar_width`, `flex_grow`, and
`flex_shrink`, layout-owned tests, and browser-parity fixture support needed to
compile and prove the C02 contracts.

It does not introduce `LayoutRootRequestOf`, `CompletedLayoutBatchOf`,
`LayoutResultOf`, session-staged tree writes, root compute failure atomicity,
root adapters, root API artifacts, docs/MSRV changes, or sibling edits. It does
not close later `FRI-04` property-family splits or later `FRI-05` scroll
geometry behavior.

Current evidence: `CacheOf` stores only `Size<S>` for compute-size entries and
reconstructs `ComputeOutputOf::from_outer_size`; `compute_leaf` passes raw
`Size<Option<S>>` and `Size<AvailableOf<S>>` to a `Size<S>` provider and can
send negative definite content-space constraints after inset subtraction;
`NodeInputOf` exposes raw scalar `scrollbar_width`, `flex_grow`, and
`flex_shrink` fields consumed directly by block, flex, grid, scroll, tests, and
browser-parity support.

## 2 Impacts

Public API: breaking pre-release changes to cache internals, `compute_leaf`,
leaf measurement input/result types, and `NodeInputOf` numeric property field
types. New public front-door types are limited to C02-owned wrappers and leaf
measurement contracts.

Dependencies/features/artifacts/docs/MSRV: no new dependency, feature,
generated artifact, documentation, or MSRV change in this cycle.

Root follow-up: root later constructs scrollbar/flex numeric wrappers and adapts
leaf measurement callers after this candidate is published.

Unsafe: no Surgeist-owned unsafe may be added or retained.

## 3 Tasks

### 3.1 `P01/I01/S01/C02/T01` - Store Complete Compute-Size Outputs

**Files/area:** `src/cache.rs`, `src/cache_tests.rs`, cache contract tests

**Intended behavior/outcome:** Compute-size cache entries store complete `ComputeOutputOf<S>` and cache hits return field-for-field equivalent outputs.

**RED evidence:** A focused cold/hit test fails on cycle base because cached compute-size output preserves size but loses content size, scroll geometry, baselines, collapsible margins, and collapse-through state.

**Acceptance criteria:** `CacheOf` compute-size storage no longer stores bare `Size<S>`; hit/miss matching still uses cached output size where needed; f32 and f64 tests prove complete-output equality and existing cache invalidation behavior remains.

**Commands:** `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout cache -- --nocapture`; `cargo fmt --check`

**Depends on:** Published `P01/I01/S01/C01`

**Intended commit:** `cache: store complete compute-size outputs`

### 3.2 `P01/I01/S01/C02/T02` - Validate Numeric Layout Properties

**Files/area:** `src/node_input.rs`, `src/lib.rs`, block/flex/grid/scroll/compute consumers, tests, browser-parity support

**Intended behavior/outcome:** Add private-field `ScrollbarWidthOf<S>`, `FlexGrowOf<S>`, and `FlexShrinkOf<S>` with finite non-negative construction and update `NodeInputOf` fields and algorithms to consume validated values.

**RED evidence:** Focused construction/default tests fail because wrappers do not exist and raw negative or non-finite fields can be assigned through public paths.

**Acceptance criteria:** Wrappers reject negative, NaN, and infinite input; defaults are scrollbar `0`, flex-grow `0`, flex-shrink `1`; `NodeInputOf` exposes wrapper fields; algorithms use explicit accessors; layout-owned parsing/tests construct wrappers through fallible constructors; no raw public assignment remains for the three properties.

**Commands:** `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout node_input -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout flex -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout scroll -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout grid_lanes -- --nocapture`; `bash -lc 'if rg -n -F -e "scrollbar_width: S" -e "flex_grow: S" -e "flex_shrink: S" -e "scrollbar_width = parse_number" -e "flex_grow = parse_number" -e "flex_shrink = parse_number" src tests; then exit 1; else rc=$?; test "$rc" -eq 1; fi'`; `cargo fmt --check`

**Depends on:** `P01/I01/S01/C02/T01`

**Intended commit:** `input: validate numeric layout properties`

### 3.3 `P01/I01/S01/C02/T03` - Validate The Leaf Measurement Boundary

**Files/area:** `src/compute.rs`, `src/lib.rs`, leaf/compute tests, direct `compute_leaf` call sites, browser-parity support

**Intended behavior/outcome:** Replace direct leaf measurement callback input/output with `LeafMeasureInputOf<S>`, `MeasurementAvailableOf<S>`, and `Result<ComputeOutputOf<S>, LeafMeasureErrorOf<S, M>>` while keeping recursive tree compute/session APIs for C03.

**RED evidence:** Focused leaf tests fail because current `compute_leaf` can pass negative definite content-space constraints and accepts negative, NaN, or infinite provider sizes as successful measurement.

**Acceptance criteria:** Provider input known/available sizes are content-space, finite, and floored at zero after padding, border, and scrollbar inset subtraction; intrinsic min/max availability remains symbolic; provider `Err(M)` is preserved as `LeafMeasureErrorOf::Provider(M)`; negative or non-finite provider dimensions return `LeafMeasureErrorOf::InvalidOutput` before padding, border, scrollbar, cache, or output construction; f32 and f64 tests cover the contract; existing callers are updated without duplicate measurement paths.

**Commands:** `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout leaf -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout compute -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout block -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout flex -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout grid -- --nocapture`; `bash -lc 'if rg -n -F -e "measure: impl FnOnce(Size<Option<S>>, Size<AvailableOf<S>>) -> Size<S>" -e "FnOnce(Size<Option<S>>, Size<AvailableOf<S>>)" src/compute.rs; then exit 1; else rc=$?; test "$rc" -eq 1; fi'`; `cargo fmt --check`

**Depends on:** `P01/I01/S01/C02/T02`

**Intended commit:** `leaf: validate measurement boundary`

## 4 Completion

Cycle acceptance:

1. compute-size cache cold and hit outputs are semantically equivalent for every
   `ComputeOutputOf<S>` field currently in the cache contract;
2. `ScrollbarWidthOf<S>`, `FlexGrowOf<S>`, and `FlexShrinkOf<S>` are the only
   public construction path for their `NodeInputOf` properties and reject
   negative or non-finite scalar input;
3. direct leaf measurement receives validated content-space constraints and
   returns either validated non-negative finite provider output or a typed
   provider-preserving local error;
4. recursive root/session/batch/error APIs remain deferred to `P01/I01/S01/C03`; and
5. root handoff notes that integration must construct numeric wrappers and use
   the new leaf measurement contract after publication.

Final command list:

```sh
CARGO_NET_OFFLINE=true cargo test -p surgeist-layout
CARGO_NET_OFFLINE=true cargo test -p surgeist-layout --features layout-golden-generate
SURGEIST_PARITY_FILTER=block/block_calc_width_margin CARGO_NET_OFFLINE=true cargo test -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored
SURGEIST_PARITY_FILTER=flex/flex_calc_basis_margin_gap CARGO_NET_OFFLINE=true cargo test -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored
SURGEIST_PARITY_FILTER=grid/grid_calc_track_and_item_margin CARGO_NET_OFFLINE=true cargo test -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored
CARGO_NET_OFFLINE=true cargo clippy -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
cargo fmt --check
git diff --check
bash -lc 'files=$(git ls-files "*.rs"; git ls-files --others --exclude-standard "*.rs"); test -n "$files"; printf "%s\n" "$files" | xargs rg -n --pcre2 '\''#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{'\''; rc=$?; test "$rc" -eq 1'
```

Required handoff: after publication, report the published SHA and root-facing
numeric wrapper plus leaf measurement contract changes. No sibling or root edits
occur in this cycle.

Genuine blockers: if changing `compute_leaf` exposes a dependency on the C03
unified error envelope, stop with the exact call path and failing evidence rather
than adding a duplicate root/session error model in C02.
