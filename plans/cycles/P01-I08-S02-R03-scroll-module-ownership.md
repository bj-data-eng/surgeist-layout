# P01-I08-S02-R03 Scroll Module Ownership

Cycle ID: `P01/I08/S02/R03`

Owning repository: `surgeist-layout`

Status: reviewed

Cycle base: `e00b2f172943daa91b78f55d38ce0409e3c811f4`

Reviewed specification: `plans/specs/P01-I08-grid-subgrid-and-grid-lanes-completeness.md`, normalized semantic-content SHA-256 `e3ac1af46ff7868e12e8df01da1cd4b46edd972638f34483fedc524b9d830595`, commit `28d4016e7bf1005b8541868e8b1d251b0e03012c`: `FRI-08.20` row `AR-004`, `FRI-08.21`, `FRI-08.24.1`, the scroll projection boundary in `FRI-08.25`, the R03 anchors in `FRI-08.27`, and `FRI-08.28(1)`, `(5)`, and `(8)` through `(10)`.

Reviewed sequence: `plans/sequences/P01-I08-S02-architectural-remediation.md`, normalized semantic-content SHA-256 `6d08a4c1e63a2cfd5ab858757bd6e614c852749ce93bf54d31409aa5687b7c59`, commit `fcaf08b36149bc61f45d283759149ef8748401b8`, entry `P01/I08/S02/R03`.

Bounded outcome: scroll models, box geometry, contributions, canonical construction, and canonical rounding each have one private module owner; root reexports and all observable scroll behavior remain exact; no second construction path exists.

## 1 Boundary And Impacts

The clean published R02 candidate is immutable. Public scroll types, fields, accessors, errors, `FlowAxes` conversion methods, root reexports, layout geometry, clips, gutters, ranges, targets, settlement, cache identity, and rounding remain source- and behavior-compatible. `scroll_tests.rs` remains the companion suite until R07.

This cycle creates only `src/scroll/model.rs`, `box_geometry.rs`, `contribution.rs`, `construction.rs`, `rounding.rs`, and final `mod.rs`. During T01–T04, `src/scroll.rs` remains the private facade and declares the new submodules; T05 replaces it with byte-equivalent facade composition in `src/scroll/mod.rs`. Moved internal test modules follow their semantic owner. Cross-file visibility is `pub(super)` or narrower except intentional unchanged `pub(crate)` consumers. No public module path is added.

Exit ownership:

- `model`: immutable public rectangles, clips, targets, offsets, ranges, gutter rectangles, `ScrollGeometryOf`, accessors, and `FlowAxes` conversions;
- `box_geometry`: used overflow, auto settlement, reservations, clipping, gutters, optimal region, and measured content-box inset;
- `contribution`: intervals, accumulators, alignment subjects, final-flow ends, origin-aware range derivation;
- `construction`: canonical sources/builders, complete geometry construction and reconstruction;
- `rounding`: canonical source and geometry rounding; and
- `mod`: private composition and intentional reexports only.

Out of scope: behavior/API changes; scroll input projections (R06); block/flex/grid decomposition; companion-test partitioning; README/API map; dependencies/features/lockfile/MSRV; root/sibling work; FRI-09; generator architecture, helper, HTML, manifest, XML, report, browser execution, generation, acquisition, unsafe, suppression, or cargo clean before publication.

Public API classification: internal-only physical relocation. Frozen artifacts: corpus `c6e6f1422e14a5e4aa474c143998063ce0de4d0a9123b69875b35a4ed009a8f6`, helper `c684c7f167d95997a4a9f0250467bbaf72c1b73e69e0f707a2ef32f4d25f7f36`, `all.json` `c10dc550d260a239c8bf9dd553f5272ca3bcc2826099bc182f800986b8b94c0e`, 1,448 HTML, 5,776 comment-free XML.

Each task first runs nonzero existing characterizations, then adds its named structural anchor after the four quoted suppression fixtures and proves assertion-level RED on the assignment base. Structural evidence supplements rather than replaces behavioral evidence.

## 2 Tasks

### 2.1 `P01/I08/S02/R03/T01` Extract Scroll Models

**Files/area:** `src/scroll/model.rs`, `src/scroll.rs`, `src/lib.rs`, `src/lib_tests.rs`, direct imports, model tests.

**Outcome:** move all public immutable scroll carriers/errors/accessors, gutter rectangles, `ScrollGeometryOf`, and `FlowAxes` offset/range conversions to `model`. Private canonical source fields may remain referenced through the parent facade until T04; no public visibility changes.

**Characterization/RED:** run `fri05_c02_rect_`, `scroll_projection_`, `scroll_coordinate_`, `scroll_conversion_`, `physical_scroll_range_`, and `fri05_c02_carrier_public_aliases_`. Add `fri08_remediation_scroll_model_has_one_owner`; RED while model declarations remain in `scroll.rs`.

**Acceptance:** constructors, validation precedence, signed-zero behavior, scalar lanes, conversions, traits, docs, and root API are exact; one model owner and complete inventory.

```sh
set -e; for filter in fri05_c02_rect_ scroll_projection_ scroll_coordinate_ scroll_conversion_ physical_scroll_range_ fri05_c02_carrier_public_aliases_ fri08_remediation_scroll_model_has_one_owner; do CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout "$filter"; done
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout
CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
cargo fmt --check; git diff --check
```

Dependency: none. Commit: `refactor(scroll): extract models`.

### 2.2 `P01/I08/S02/R03/T02` Extract Box Geometry

**Files/area:** `src/scroll/box_geometry.rs`, `src/scroll.rs`, direct imports, `src/lib_tests.rs`, exact box/gutter tests.

**Outcome:** move used-overflow mapping, auto-scrollbar observation/settlement, physical reservation/insets, measured content-box inset, clip-margin/optimal-region carriers, canonical scroll box, clipping and gutter derivation. Construction consumes this owner; no algorithm duplicates reservation or clip rules.

**Characterization/RED:** run `fri05_c01_node_input_private_used_overflow_`, `fri05_c02_box_clip_gutter_`, `scrollbar_`, `content_box_inset_`, and `fri05_c03_block_reservation_`. Add `fri08_remediation_scroll_box_geometry_has_one_owner`; RED while box declarations remain in `scroll.rs`.

**Acceptance:** overflow mapping, settlement monotonicity, reservations, saturated clips, gutters, optimal region, errors, and scalar lanes are exact; one private owner.

```sh
set -e; for filter in fri05_c01_node_input_private_used_overflow_ fri05_c02_box_clip_gutter_ scrollbar_ content_box_inset_ fri05_c03_block_reservation_ fri08_remediation_scroll_box_geometry_has_one_owner; do CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout "$filter"; done
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout
CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
cargo fmt --check; git diff --check
```

Dependency: T01. Commit: `refactor(scroll): extract box geometry`.

### 2.3 `P01/I08/S02/R03/T03` Extract Contributions

**Files/area:** `src/scroll/contribution.rs`, `src/scroll.rs`, direct block/flex/grid imports, `src/lib_tests.rs`, exact contribution tests.

**Outcome:** move physical intervals/bounds, optional intervals, final in-flow ends, contribution errors/accumulator, child margin contribution, origin axes/progression, and origin-aware range derivation. Preserve one accumulator/state model and existing caller construction.

**Characterization/RED:** run `fri05_c02_contribution_range_`, `fri05_c03_block_contribution_`, `fri05_c04_flex_contribution_`, and `fri05_c05_grid_contribution_`. Add `fri08_remediation_scroll_contribution_has_one_owner`; RED while accumulator/range declarations remain in `scroll.rs`.

**Acceptance:** source categories, union order, alignment subjects, terminal padding, reversed origins, errors, and both scalar lanes are exact; no second range derivation.

```sh
set -e; for filter in fri05_c02_contribution_range_ fri05_c03_block_contribution_ fri05_c04_flex_contribution_ fri05_c05_grid_contribution_ fri08_remediation_scroll_contribution_has_one_owner; do CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout "$filter"; done
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout
CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
cargo fmt --check; git diff --check
```

Dependency: T02. Commit: `refactor(scroll): extract contributions`.

### 2.4 `P01/I08/S02/R03/T04` Extract Canonical Construction

**Files/area:** `src/scroll/construction.rs`, `src/scroll.rs`, model source-field import, direct algorithm callers, `src/lib_tests.rs`, exact construction tests.

**Outcome:** move canonical source carriers/builders, retained-source policy, measured-leaf source, geometry errors, scroll-box adapter, settlement geometry comparison, border-box reconstruction, complete source construction, measured-leaf construction, and measured-content rectangle. `ScrollGeometryOf` remains model-owned and stores the private construction source unchanged.

**Characterization/RED:** run `canonical_geometry_`, `fri08_c07_t02_scroll_source_`, `fri05_c03_leaf_geometry_`, `fri05_c05_grid_geometry_`, and `fri08_c04_overflow_canonical_`. Add `fri08_remediation_scroll_construction_has_one_owner`; RED while builder/factory declarations remain in `scroll.rs`.

**Acceptance:** exactly one canonical factory/builder/reconstruction path, all callers and local errors exact, settlement and cache behavior unchanged.

```sh
set -e; for filter in canonical_geometry_ fri08_c07_t02_scroll_source_ fri05_c03_leaf_geometry_ fri05_c05_grid_geometry_ fri08_c04_overflow_canonical_ fri08_remediation_scroll_construction_has_one_owner; do CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout "$filter"; done
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout
CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
cargo fmt --check; git diff --check
```

Dependency: T03. Commit: `refactor(scroll): extract construction`.

### 2.5 `P01/I08/S02/R03/T05` Extract Rounding And Finalize Facade

**Files/area:** `src/scroll/rounding.rs`, delete `src/scroll.rs`, create `src/scroll/mod.rs`, direct engine import, `src/lib.rs`, `src/lib_tests.rs`, exact rounding tests and inventory.

**Outcome:** move canonical geometry/source/edge/padding/contribution/final-end rounding to `rounding`; replace the old monolith with `mod.rs` declarations and intentional private/public reexports. Move remaining internal tests to owners. No implementation declaration remains in `mod.rs` beyond facade composition.

**Characterization/RED:** run `source_rounding_`, `scroll_geometry_projects_`, `fri06_c02_rounding_`, and the exact flex/grid canonical publication anchors. Add `fri08_remediation_scroll_construction_and_rounding_equivalence`; RED while rounding remains in `scroll.rs` or facade is not finalized.

**Acceptance:** cumulative rounding, typed failures, fragments/callers, source reconstruction, scalar/writing modes, public API, and all earlier R03 anchors pass; `src/scroll.rs` absent and five owners singular.

```sh
set -e; for filter in source_rounding_ scroll_geometry_projects_ fri06_c02_rounding_ fri05_c04_flex_round_cache_publication_has_one_canonical_geometry_path fri05_c05_grid_round_cache_has_no_independent_scrollbar_projection fri08_remediation_scroll_construction_and_rounding_equivalence fri08_remediation_public_api_inventory_is_compatible; do CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout "$filter"; done
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout
CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
cargo fmt --check; git diff --check
```

Dependency: T04. Commit: `refactor(scroll): extract canonical rounding`.

## 3 Completion

R03 requires five independently CLEAN task ranges, status complete, CLEAN final matrix and holistic review, exact public/API/behavior/dependency/MSRV/safety/artifact invariants, publication/readback, no stale process, successful `cargo clean`, absent `target/`, and an immutable R04 handoff.

```sh
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
expected_paths="$({ printf '%s\n' 'plans/cycles/P01-I08-S02-R03-scroll-module-ownership.md'; while IFS= read -r span; do git diff --name-only "$span"; done <<< "$TASK_SPANS"; } | LC_ALL=C sort -u)"; actual_paths="$(git diff --name-only e00b2f172943daa91b78f55d38ce0409e3c811f4..HEAD | LC_ALL=C sort -u)"; test "$actual_paths" = "$expected_paths"
if git diff --word-diff=porcelain --word-diff-regex='[[:alpha:]_][[:alnum:]_]*' e00b2f172943daa91b78f55d38ce0409e3c811f4..HEAD -- '*.rs' | rg '^\+.*\b(allow|expect)\b'; then exit 1; fi
normalized_allow_inventory="$({ git ls-files -z -- '*.rs'; git ls-files -z --others --exclude-standard -- '*.rs'; } | sort -zu | xargs -0 rg -n -U --pcre2 '#\s*\[\s*allow\b' | sed -E 's/^([^:]+):[0-9]+:/\1:/' | LC_ALL=C sort || true)"; expected_allow_inventory="$(printf '%s\n' 'src/contract_tests.rs:        !text_source.contains("#[allow(dead_code)]"),' 'src/lib_tests.rs:        "#[allow(clippy::too_many_arguments)]",' 'src/lib_tests.rs:        "#[allow(dead_code)] /* between attributes */ #[cfg_attr(not(test), cfg(test))] pub(crate) fn hidden() { scrollbar_size; }",' 'src/lib_tests.rs:        "#[allow(dead_code)]",' | LC_ALL=C sort)"; test "$normalized_allow_inventory" = "$expected_allow_inventory"
unsafe_hits="$({ git ls-files -z -- '*.rs'; git ls-files -z --others --exclude-standard -- '*.rs'; } | sort -zu | xargs -0 rg -n '\bunsafe\b' | rg -v 'safe_fallback returns unsafe|Some\("async" \| "unsafe" \| "default" \| "extern"\)|removed phase-unsafe surface remains|starts_with\("unsafe "\)|strip_prefix\("unsafe "\)|parse_align_content\("unsafe end"\)|parse_align_items\("unsafe first baseline"\)' || true)"; test -z "$unsafe_hits"
if { git ls-files -z -- '*.rs'; git ls-files -z --others --exclude-standard -- '*.rs'; } | sort -zu | xargs -0 rg -n -U '\b(no_mangle|export_name|link_section|naked)\b|(^|[^[:alnum:]_\"])extern[[:space:]]*\"'; then exit 1; fi
test "$(shasum -a 256 tests/layout/browser_parity/corpus.toml | awk '{print $1}')" = c6e6f1422e14a5e4aa474c143998063ce0de4d0a9123b69875b35a4ed009a8f6
test "$(shasum -a 256 tests/layout/browser_parity/scripts/gentest/test_helper.js | awk '{print $1}')" = c684c7f167d95997a4a9f0250467bbaf72c1b73e69e0f707a2ef32f4d25f7f36
test "$(shasum -a 256 tests/layout/browser_parity/xml/generation-reports/all.json | awk '{print $1}')" = c10dc550d260a239c8bf9dd553f5272ca3bcc2826099bc182f800986b8b94c0e
test "$(find tests/layout/browser_parity/html -type f -name '*.html' | wc -l | tr -d ' ')" = 1448; test "$(find tests/layout/browser_parity/xml -type f -name '*.xml' | wc -l | tr -d ' ')" = 5776; test -z "$(git status --porcelain=v1)"
```

After publication/readback: prove no stale layout Cargo/Rust/generator process; run `cargo clean`; prove `target/` absent and Git clean. R04 handoff records published SHA, reviewed revisions, five accepted spans/reviews, final/API/dependency/safety/artifact evidence, readback, and cleanup. Blocker disposition: none known; absent/wrong existing Taffy cache or any required behavior/API/artifact/generator expansion is a stop, never acquisition or scope widening.
