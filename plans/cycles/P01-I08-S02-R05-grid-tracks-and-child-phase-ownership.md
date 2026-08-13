# P01-I08-S02-R05 Grid Tracks And Child Phase Ownership

Cycle ID: `P01/I08/S02/R05`

Owning repository: `surgeist-layout`

Status: complete

Cycle base: `4f36831d330fd89c60e027288409016e1166a785`

Reviewed specification: `plans/specs/P01-I08-grid-subgrid-and-grid-lanes-completeness.md`, normalized semantic-content SHA-256 `d9c6a61eae363331d7e8ce05d88916099111e11b8793b8dc31cc55e3e5c80a6a`, commit `b9cb82aadf70d5690d605bb9ffeaa6da9512bd3d`: `FRI-08.20` row `AR-004`, `FRI-08.21`, `FRI-08.24.3`, the grid projection boundary in `FRI-08.25`, the grid cases in `FRI-08.27`, the temporary-test-debt boundary in `FRI-08.27.1`, and `FRI-08.28(1)`, `(5)`, and `(8)` through `(10)`.

Reviewed sequence: `plans/sequences/P01-I08-S02-architectural-remediation.md`, normalized semantic-content SHA-256 `46d3563226ba6b91478bdc0b36273abb56644720774804b7c7a2ab9d0ca07251`, commit `2f097f4b9ac510df63e3e886e2f7a46f0312a701`, entry `P01/I08/S02/R05`.

Bounded outcome: grid track sizing and settled child layout become private phase-shaped module trees with one existing state model per responsibility, unchanged facade paths and behavior, and no duplicate topology, placement, baseline, subgrid, sizing, or scroll policy.

## 1 Boundary And Impacts

The published R04 candidate is immutable. Public API, errors, scalar lanes, topology, named lines, placement, lanes, standalone subgrid traversal, engine/session, sizing/scroll owners, cache/batch/rounding semantics, dependencies/features/MSRV, documentation, and browser artifacts remain exact. Node projections and the API map belong to R06; companion-test partitioning belongs to R07; whole-crate removal of source/workflow proxy tests belongs to R08.

Final track owners are `mod` (facade/composition and shared track geometry carriers), `validation` (track input validation and expansion), `intrinsic` (ordinary intrinsic contribution collection and baseline inputs), `subgrid_intrinsic` (subgrid projection and queried-axis constraints), `ordinary` (ordinary phase state and bound distribution), and `flexible` (flex fractions, auto-max stretch, final track sizes, sums, gutters, and offsets). Final child owners are `mod` (facade/composition and settled child layout entry), `baseline` (baseline groups/participation/placement), `subgrid_context` (child context and refresh), `absolute` (absolute grid area/axes/layout), and `scroll` (retained geometry, contributions, flow ends, and publication helpers).

Cross-owner visibility is `pub(super)` or narrower except unchanged crate-visible test facade contracts. Existing concrete track, geometry, baseline, pending-item, and context carriers move to one semantic owner; no parallel carrier or model is permitted. `src/grid/mod.rs` and sibling callers retain `use tracks::*` and `use child::*`; topology/lanes/subgrid callers retain their existing facade paths. No production function becomes public merely to compile the split.

Each task runs nonzero behavioral characterizations before mutation and proves its named external shell ownership probe RED at its assignment base and GREEN afterward. R05 adds no Rust source-parsing, file-placement, symbol-count, plan-state, or current-output test. Existing legacy source-inspection tests may receive only the minimum owner-path aggregation or recursive inventory adaptation required to preserve the historical suite; no assertion, test, or semantics may be added or strengthened. R08 remains responsible for deleting that entire class.

Public API classification: internal-only. Frozen artifacts: corpus `c6e6f1422e14a5e4aa474c143998063ce0de4d0a9123b69875b35a4ed009a8f6`, helper `c684c7f167d95997a4a9f0250467bbaf72c1b73e69e0f707a2ef32f4d25f7f36`, `all.json` `c10dc550d260a239c8bf9dd553f5272ca3bcc2826099bc182f800986b8b94c0e`, 1,448 HTML, and 5,776 comment-free XML. Out of scope: FRI-09; root/siblings; browser execution, generation, acquisition, generator/helper/manifest/report/XML changes; unsafe or new suppressions; `cargo clean` before publication.

## 2 Tasks

### 2.1 `P01/I08/S02/R05/T01` Track Validation And Expansion

**Area:** `src/grid/tracks/validation.rs`, `src/grid/tracks.rs`, `src/grid/topology.rs` direct import if required, embedded validation/expansion tests, and exact existing `src/lib_tests.rs` recursive-inventory adaptation only.

**Outcome:** move track calculation validation, component validation, auto-repeat expansion/origins/count/basis, subgrid/auto-repeat predicates, and reserved track space to one owner. Expansion retains the existing `TrackExpansionOf` model and topology consumes it through the private tracks facade.

**RED/acceptance:** `fri08_c01_topology_`, `fri08_c02_auto_fit_`, and `track_sizing_` pass nonzero. The external probe requires `validation.rs` and singular `TrackExpansionOf`; RED then GREEN. Preserve validation precedence, implicit/explicit identity, auto-fit/fill, named-line origins, errors, and both scalar lanes.

```sh
set -e; for f in fri08_c01_topology_ fri08_c02_auto_fit_ track_sizing_; do CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout "$f"; done
test -f src/grid/tracks/validation.rs; test "$(rg -l 'struct TrackExpansionOf' src/grid/tracks.rs src/grid/tracks/*.rs | sort -u | wc -l | tr -d ' ')" = 1
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout; CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings; cargo fmt --check; git diff --check
```

Dependency: none. Commit: `refactor(grid): extract track validation`.

### 2.2 `P01/I08/S02/R05/T02` Ordinary Intrinsic Contribution Collection

**Area:** `src/grid/tracks/intrinsic.rs`, `src/grid/tracks.rs`, direct private imports, embedded intrinsic tests, exact existing owner-path aggregation in `src/grid_tests.rs` only when required, and `src/lib_tests.rs` exact recursive production-inventory addition only.

**Outcome:** move intrinsic grid inputs/lower bounds, row/item contribution collection, intrinsic baseline members/targets, contribution margin/eligibility/distribution, and constrained row/column intrinsic sizing to one owner. Preserve the sole existing ancestor-baseline and contribution models.

**RED/acceptance:** `fri08_c03_intrinsic_`, `grid_spanning_item_`, `grid_row_intrinsic_`, and `fri08_c04_baseline_` pass. External probe requires `intrinsic.rs` and singular `IntrinsicGrid`; RED then GREEN. Preserve ordering, spans, margins, fit-content floors, percent reserves, baseline shims, failures, caches, and scalars.

The existing recursive source inventory may add and classify only `src/grid/tracks/intrinsic.rs`; no assertion, test, source-proxy semantics, or other `src/lib_tests.rs` content changes.

```sh
set -e; for f in fri08_c03_intrinsic_ grid_spanning_item_ grid_row_intrinsic_ fri08_c04_baseline_; do CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout "$f"; done
test -f src/grid/tracks/intrinsic.rs; test "$(rg -l 'struct IntrinsicGrid<' src/grid/tracks.rs src/grid/tracks/*.rs | sort -u | wc -l | tr -d ' ')" = 1
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout; CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings; cargo fmt --check; git diff --check
```

Dependency: T01. Commit: `refactor(grid): extract intrinsic collection`.

### 2.3 `P01/I08/S02/R05/T03` Subgrid Intrinsic Projection

**Area:** `src/grid/tracks/subgrid_intrinsic.rs`, `src/grid/tracks.rs`, direct private imports, embedded subgrid-intrinsic tests, and `src/lib_tests.rs` exact recursive production-inventory addition only.

**Outcome:** move intrinsic subgrid axis authority/constraints, child input and border-box constraints, recursive contribution projection, queried-axis dependency, inherited-axis checks, and percent/cyclic subgrid content handling to one owner. Standalone traversal remains in `grid/subgrid.rs` and topology remains unchanged.

**RED/acceptance:** `fri08_c03_nested_`, `fri08_c04_overflow_hidden_subgrid_`, `subgrid_intrinsic_`, and `fri08_c02_auto_fit_inherited_` pass. External probe requires `subgrid_intrinsic.rs` and singular `SubgridIntrinsicContributionInput`; RED then GREEN. Preserve reversal, inherited gaps/collapse, contexts, recursion, errors, scalar lanes, and failure atomicity.

The existing recursive source inventory may add and classify only `src/grid/tracks/subgrid_intrinsic.rs`; no assertion, test, source-proxy semantics, or other `src/lib_tests.rs` content changes.

```sh
set -e; for f in fri08_c03_nested_ fri08_c04_overflow_hidden_subgrid_ subgrid_intrinsic_ fri08_c02_auto_fit_inherited_; do CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout "$f"; done
test -f src/grid/tracks/subgrid_intrinsic.rs; test "$(rg -l 'struct SubgridIntrinsicContributionInput' src/grid/tracks.rs src/grid/tracks/*.rs | sort -u | wc -l | tr -d ' ')" = 1
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout; CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings; cargo fmt --check; git diff --check
```

Dependency: T02. Commit: `refactor(grid): extract subgrid intrinsic projection`.

### 2.4 `P01/I08/S02/R05/T04` Ordinary Track Phases

**Area:** `src/grid/tracks/ordinary.rs`, `src/grid/tracks.rs`, direct private imports, embedded ordinary-track tests, and `src/lib_tests.rs` exact recursive production-inventory addition only.

**Outcome:** move ordinary track state, min/base/growth-limit phases, ordinary intrinsic application, between-bound distribution, inline/axis track resolution, and intrinsic bound helpers to one owner. `OrdinaryTrackState` remains the single phase model.

**RED/acceptance:** `fri08_c02_fit_content_`, `grid_fraction_tracks_`, `grid_stretch_`, and `fri08_c02r_lanes_track_phase_` pass. External probe requires `ordinary.rs` and singular `OrdinaryTrackState`; RED then GREEN. Preserve phase order, floors/limits, fit-content, percent tracks, lanes composition, gutters, and both scalars.

The existing recursive source inventory may add and classify only `src/grid/tracks/ordinary.rs`; no assertion, test, source-proxy semantics, or other `src/lib_tests.rs` content changes.

```sh
set -e; for f in fri08_c02_fit_content_ grid_fraction_tracks_ grid_stretch_ fri08_c02r_lanes_track_phase_; do CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout "$f"; done
test -f src/grid/tracks/ordinary.rs; test "$(rg -l 'struct OrdinaryTrackState' src/grid/tracks.rs src/grid/tracks/*.rs | sort -u | wc -l | tr -d ' ')" = 1
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout; CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings; cargo fmt --check; git diff --check
```

Dependency: T03. Commit: `refactor(grid): extract ordinary track phases`.

### 2.5 `P01/I08/S02/R05/T05` Flexible Tracks Final Geometry And Facade

**Area:** `src/grid/tracks/flexible.rs`, delete `src/grid/tracks.rs`, create `src/grid/tracks/mod.rs`, direct sibling imports, exact legacy source-proxy/inventory path adaptations in `src/grid_tests.rs` and `src/lib_tests.rs`, and embedded flex/geometry tests.

**Outcome:** move flex fraction resolution, auto-max stretch, final sizes, sums, spans, gutters, offsets, and final used-axis geometry helpers to `flexible`; finalize composition-only `tracks/mod.rs` with intentional private reexports and shared geometry carriers.

**RED/acceptance:** `fri08_c02_fit_content_`, `fri08_c02_auto_fit_`, `fri08_c06_collapsed_gutter_`, `grid_fraction_tracks_`, and `fri08_remediation_public_api_inventory_is_compatible` pass. External probe requires all six track owners, absent legacy file, and one `UsedGridAxisGeometryOf`; RED then GREEN. Preserve every caller path, collapse identity, gutter masks, distribution, offsets, errors, and scalar lanes; add no source-parsing test.

```sh
set -e; for f in fri08_c02_fit_content_ fri08_c02_auto_fit_ fri08_c06_collapsed_gutter_ grid_fraction_tracks_ fri08_remediation_public_api_inventory_is_compatible; do CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout "$f"; done
for p in src/grid/tracks/{mod,validation,intrinsic,subgrid_intrinsic,ordinary,flexible}.rs; do test -f "$p"; done; test ! -e src/grid/tracks.rs; test "$(rg -l 'struct UsedGridAxisGeometryOf' src/grid/tracks/*.rs | wc -l | tr -d ' ')" = 1
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout; CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings; cargo fmt --check; git diff --check
```

Dependency: T04. Commit: `refactor(grid): finalize track phase ownership`.

### 2.6 `P01/I08/S02/R05/T06` Child Baseline Ownership

**Area:** `src/grid/child/baseline.rs`, `src/grid/child.rs`, direct private imports, embedded baseline tests, and exact existing `src/lib_tests.rs` owner-path aggregation only.

**Outcome:** move baseline group/participation/geometry/shims, ancestor member transport, aligned offsets, final groups, container baselines, and cycle checks to one owner. Preserve the existing `GridBaselineGroups` and ancestor-group models without duplicates.

**RED/acceptance:** `fri08_c04_baseline_`, `subgrid_baseline_`, `grid_lanes_reports_`, and `grid_reports_` pass. External probe requires `baseline.rs` and singular `GridBaselineGroups`; RED then GREEN. Preserve first/last roles, physical/logical axes, synthesized behavior, cycles, errors, and scalar lanes.

```sh
set -e; for f in fri08_c04_baseline_ subgrid_baseline_ grid_lanes_reports_ grid_reports_; do CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout "$f"; done
test -f src/grid/child/baseline.rs; test "$(rg -l 'struct GridBaselineGroups' src/grid/child.rs src/grid/child/*.rs | sort -u | wc -l | tr -d ' ')" = 1
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout; CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings; cargo fmt --check; git diff --check
```

Dependency: T05. Commit: `refactor(grid): extract child baselines`.

### 2.7 `P01/I08/S02/R05/T07` Child Subgrid Context And Refresh

**Area:** `src/grid/child/subgrid_context.rs`, `src/grid/child.rs`, direct private imports, embedded context/refresh tests, and `src/lib_tests.rs` exact recursive production-inventory addition only.

**Outcome:** move subgrid parent/axis context, inherited layout tracks/gaps, baseline inheritance, final axis constraints, subgrid item refresh, and typed context errors to one owner. Standalone traversal and parent topology remain with existing owners.

**RED/acceptance:** `fri08_c03_nested_`, `fri08_c02_auto_fit_inherited_`, `subgrid_child_`, and `fri08_c04_baseline_cache_` pass. External probe requires `subgrid_context.rs` and singular `SubgridChildContextError`; RED then GREEN. Preserve slicing/reversal, line offsets, active gutters, baseline transport, errors, cache/rollback, and scalars.

The existing recursive source inventory may add and classify only `src/grid/child/subgrid_context.rs`; no assertion, test, source-proxy semantics, or other `src/lib_tests.rs` content changes.

```sh
set -e; for f in fri08_c03_nested_ fri08_c02_auto_fit_inherited_ subgrid_child_ fri08_c04_baseline_cache_; do CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout "$f"; done
test -f src/grid/child/subgrid_context.rs; test "$(rg -l 'enum SubgridChildContextError' src/grid/child.rs src/grid/child/*.rs | sort -u | wc -l | tr -d ' ')" = 1
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout; CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings; cargo fmt --check; git diff --check
```

Dependency: T06. Commit: `refactor(grid): extract subgrid child context`.

### 2.8 `P01/I08/S02/R05/T08` Settled Absolute Scroll Layout And Facade

**Area:** `src/grid/child/{absolute,scroll}.rs`, delete `src/grid/child.rs`, create `src/grid/child/mod.rs`, direct imports, exact existing source-proxy/inventory path adaptations in `src/lib_tests.rs`, embedded child/absolute/scroll tests.

**Outcome:** retain settled-area child layout, pending items, item sizing/alignment, and area/offset helpers in `child/mod.rs`; move absolute grid area/axes/layout to `absolute`; move retained geometry, contributions, flow ends, and publication helpers to `scroll`; finalize the private child facade.

**RED/acceptance:** `grid_absolute_child_`, `grid_content_size_`, `fri05_c05_grid_contribution_`, `fri08_c04_overflow_`, and `fri08_c07_t04_grid_settlement_` pass. External aggregate probe requires all five child owners, absent legacy file, singular `PendingGridItem`/`AbsoluteGridContext`, direct canonical scroll consumption, and no local scroll factory; RED then GREEN. Preserve settled order, sizing/alignment, absolute/hidden handling, subgrid refresh, baselines, scroll geometry, errors, cache/batch atomicity, and scalars; add no source-parsing test.

```sh
set -e; for f in grid_absolute_child_ grid_content_size_ fri05_c05_grid_contribution_ fri08_c04_overflow_ fri08_c07_t04_grid_settlement_; do CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout "$f"; done
for p in src/grid/child/{mod,baseline,subgrid_context,absolute,scroll}.rs; do test -f "$p"; done; test ! -e src/grid/child.rs; test "$(rg -l 'struct PendingGridItem' src/grid/child/*.rs | wc -l | tr -d ' ')" = 1; test "$(rg -l 'struct AbsoluteGridContext' src/grid/child/*.rs | wc -l | tr -d ' ')" = 1; rg -q 'crate::scroll::' src/grid/child/scroll.rs; test -z "$(rg -n 'fn canonical_scroll_geometry_from_source' src/grid/child || true)"
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri08_remediation_; CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout; CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings; cargo fmt --check; git diff --check
```

Dependency: T07. Commit: `refactor(grid): finalize child phase ownership`.

## 3 Completion

R05 requires eight independently CLEAN task ranges, status complete, a GREEN final matrix and CLEAN holistic review, publication/readback, process hygiene, successful `cargo clean`, absent `target/`, and an immutable R06 handoff. Browser, generation, acquisition, and artifact writes remain prohibited. Missing/wrong pinned cache, nonzero behavior-filter failure, public/API/artifact drift, or implementation scope expansion is a stop.

```sh
set -e
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri08_remediation_
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
: "${TASK_SPANS:?set TASK_SPANS to the newline-delimited ordered exact full-SHA spans from the eight CLEAN task reviews}"
expected_paths="$({ printf '%s\n' plans/cycles/P01-I08-S02-R05-grid-tracks-and-child-phase-ownership.md; while IFS= read -r span; do git diff --name-only "$span"; done <<< "$TASK_SPANS"; } | LC_ALL=C sort -u)"; actual_paths="$(git diff --name-only 4f36831d330fd89c60e027288409016e1166a785..HEAD | LC_ALL=C sort -u)"; test "$actual_paths" = "$expected_paths"
base_suppressions="$(while IFS= read -r source_path; do git show "4f36831d330fd89c60e027288409016e1166a785:$source_path" | perl -0777 -ne 'while (/^[ \t]*#\s*\[\s*(?:allow|expect|cfg_attr)\b[^\]]*\]/gms) { $m = $&; $m =~ s/\s+/ /g; print "$m\n" }'; done < <(git ls-tree -r --name-only 4f36831d330fd89c60e027288409016e1166a785 | rg '\.rs$') | LC_ALL=C sort)"; current_suppressions="$({ git ls-files -z -- '*.rs'; git ls-files -z --others --exclude-standard -- '*.rs'; } | sort -zu | xargs -0 perl -0777 -ne 'while (/^[ \t]*#\s*\[\s*(?:allow|expect|cfg_attr)\b[^\]]*\]/gms) { $m = $&; $m =~ s/\s+/ /g; print "$m\n" }' | LC_ALL=C sort)"; test -z "$(comm -13 <(printf '%s\n' "$base_suppressions") <(printf '%s\n' "$current_suppressions"))"
if { git ls-files -z -- '*.rs'; git ls-files -z --others --exclude-standard -- '*.rs'; } | sort -zu | xargs -0 rg -n --pcre2 '#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{'; then exit 1; fi
test "$(shasum -a 256 tests/layout/browser_parity/corpus.toml | awk '{print $1}')" = c6e6f1422e14a5e4aa474c143998063ce0de4d0a9123b69875b35a4ed009a8f6
test "$(shasum -a 256 tests/layout/browser_parity/scripts/gentest/test_helper.js | awk '{print $1}')" = c684c7f167d95997a4a9f0250467bbaf72c1b73e69e0f707a2ef32f4d25f7f36
test "$(shasum -a 256 tests/layout/browser_parity/xml/generation-reports/all.json | awk '{print $1}')" = c10dc550d260a239c8bf9dd553f5272ca3bcc2826099bc182f800986b8b94c0e
test "$(find tests/layout/browser_parity/html -type f -name '*.html' | wc -l | tr -d ' ')" = 1448; test "$(find tests/layout/browser_parity/xml -type f -name '*.xml' | wc -l | tr -d ' ')" = 5776; test -z "$(git status --porcelain=v1)"
```

After publication/readback: prove no stale layout Cargo/Rust/generator process; run `cargo clean`; prove `target/` absent and Git clean. Record the published SHA, reviewed revisions, eight ordered task ranges and verdicts, final evidence, remote readback, and cleanup for R06.
