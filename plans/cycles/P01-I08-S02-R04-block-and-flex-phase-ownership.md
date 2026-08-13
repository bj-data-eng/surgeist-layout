# P01-I08-S02-R04 Block And Flex Phase Ownership

Cycle ID: `P01/I08/S02/R04`

Owning repository: `surgeist-layout`

Status: draft

Cycle base: `4f5022b720d33c1946604aeb3ce2172fd5db8fc8`

Reviewed specification: `plans/specs/P01-I08-grid-subgrid-and-grid-lanes-completeness.md`, normalized semantic-content SHA-256 `e3ac1af46ff7868e12e8df01da1cd4b46edd972638f34483fedc524b9d830595`, commit `28d4016e7bf1005b8541868e8b1d251b0e03012c`: `FRI-08.20` row `AR-004`, `FRI-08.21`, `FRI-08.24.2`, the block/flex projection boundary in `FRI-08.25`, the block/flex cases of `fri08_remediation_algorithm_phase_composition_equivalence` and the public API anchor in `FRI-08.27`, and `FRI-08.28(1)`, `(5)`, and `(8)` through `(10)`.

Reviewed sequence: `plans/sequences/P01-I08-S02-architectural-remediation.md`, normalized semantic-content SHA-256 `6d08a4c1e63a2cfd5ab858757bd6e614c852749ce93bf54d31409aa5687b7c59`, commit `fcaf08b36149bc61f45d283759149ef8748401b8`, entry `P01/I08/S02/R04`.

Bounded outcome: block and flex each become a private phase-shaped module tree with one owner for every specified responsibility, narrow carriers, unchanged entry paths, and no copied sizing or scroll policy.

## 1 Boundary

The published, remotely read-back R03 candidate is immutable. Public API, geometry, errors, cache/batching/rounding, scalar lanes, root reexports, dependencies/features/MSRV, and artifacts remain exact. `block_tests.rs` and `flex_tests.rs` remain companion suites until R07; only embedded tests follow owners. Node projections belong to R06. Grid files belong to R05.

Final block owners are `mod` (entry/constants/composition), `floats`, `in_flow`, `inline_run`, `absolute`, `sizing`, and `scroll`. Final flex owners are `mod` (entry/constants/composition), `items`, `lines`, `flexible_lengths`, `alignment`, `intrinsic`, `absolute`, and `scroll`. Cross-owner visibility is `pub(super)` or narrower except unchanged crate-visible entry/test facade contracts. No generic utility layer, parallel carrier, alternate algorithm, public module path, or copied `sizing::resolve`/canonical-scroll rule is permitted.

Frozen artifacts: corpus `c6e6f1422e14a5e4aa474c143998063ce0de4d0a9123b69875b35a4ed009a8f6`, helper `c684c7f167d95997a4a9f0250467bbaf72c1b73e69e0f707a2ef32f4d25f7f36`, `all.json` `c10dc550d260a239c8bf9dd553f5272ca3bcc2826099bc182f800986b8b94c0e`, 1,448 HTML, 5,776 comment-free XML.

Each task runs nonzero existing characterizations, adds its named structural anchor after the four quoted `allow` fixtures, and proves assertion-level RED before production edits. Source-shape evidence supplements behavior. Out of scope: FRI-09; root/siblings; generator/browser/generation/acquisition; artifacts; dependencies; README/API map; projections; companion-test partitioning; cargo clean before publication.

## 2 Tasks

### 2.1 `P01/I08/S02/R04/T01` Block Float Exclusion And BFC

**Area:** `src/block/floats.rs`, `src/block.rs`, `src/lib_tests.rs`, embedded float tests.

**Outcome:** move pending-float, ledger, band/provider, `FloatExclusions`, inherited conversion, placement/query, float publication, and float intrinsic logic to one owner. Existing block-owned inherited-float recursion remains direct and algorithm-neutral engine contracts remain untouched.

**RED/acceptance:** `fri06_c04_float_`, `fri06_c05_shape_`, and `block_bfc_` pass nonzero; add `fri08_remediation_block_float_phase_has_one_owner`. Preserve query order, bands, margins, clear, provider errors, both scalars, and facade contracts.

```sh
set -e; for f in fri06_c04_float_ fri06_c05_shape_ block_bfc_ fri08_remediation_block_float_phase_has_one_owner; do CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout "$f"; done
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout; CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings; cargo fmt --check; git diff --check
```

Dependency: none. Commit: `refactor(block): extract float exclusion`.

### 2.2 `P01/I08/S02/R04/T02` Block In-Flow And Inline Run

**Area:** `src/block/in_flow.rs`, `inline_run.rs`, `src/block.rs`, `src/lib_tests.rs`, embedded owner tests.

**Outcome:** move traversal/margin/final-flow state to `in_flow`; move inline boundaries, transitions, atomic participation, run placement/layout, and baselines to `inline_run`. One owner per carrier; float access remains narrow.

**RED/acceptance:** `ordinary_block_flow_`, `block_layout_collapses_`, `block_atomic_inline_run_`, and `fri08_c07_t01_inline_transition_` pass; add `fri08_remediation_block_flow_phases_have_one_owner`. Preserve collapse, bidi/writing modes, floats, controls, baselines, errors, order, and scalar lanes.

```sh
set -e; for f in ordinary_block_flow_ block_layout_collapses_ block_atomic_inline_run_ fri08_c07_t01_inline_transition_ fri08_remediation_block_flow_phases_have_one_owner; do CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout "$f"; done
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout; CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings; cargo fmt --check; git diff --check
```

Dependency: T01. Commit: `refactor(block): extract flow phases`.

### 2.3 `P01/I08/S02/R04/T03` Block Sizing Absolute Scroll And Facade

**Area:** `src/block/sizing.rs`, `absolute.rs`, `scroll.rs`, delete `src/block.rs`, create `src/block/mod.rs`, direct imports, `src/lib_tests.rs`, `src/contract_tests.rs`, `src/root_tests.rs`, embedded tests.

**Outcome:** move block-owned sizing composition, absolute-child layout, and canonical scroll publication to their owners; finalize composition-only `block/mod.rs` with entry/constants. Preserve `FloatExclusions`, `FloatLedgerSide`, and `resolve_logical_in_flow_margin` test facade contracts.

**RED/acceptance:** `ordinary_block_logical_sizing_`, `fri06_c13_t06_block_resolution_`, `block_absolute_child_`, `fri05_c03_block_contribution_`, and `fri08_c07_t02_scroll_source_block_` pass; add `fri08_remediation_block_phase_composition_equivalence`. `src/block.rs` is absent; shared sizing/scroll owners are consumed directly and singularly.

```sh
set -e; for f in ordinary_block_logical_sizing_ fri06_c13_t06_block_resolution_ block_absolute_child_ fri05_c03_block_contribution_ fri08_c07_t02_scroll_source_block_ fri08_remediation_block_phase_composition_equivalence; do CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout "$f"; done
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout; CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings; cargo fmt --check; git diff --check
```

Dependency: T02. Commit: `refactor(block): finalize phase ownership`.

### 2.4 `P01/I08/S02/R04/T04` Flex Item Collection And Finalization

**Area:** `src/flex/items.rs`, `src/flex.rs`, `src/lib_tests.rs`, embedded item tests.

**Outcome:** move collected/resolved/final item carriers, collection, basis, automatic minimum, and final in-flow child layout to `items`; retain `FlexAxes` facade and narrow phase inputs.

**RED/acceptance:** `flex_order_modified_sequence_`, `flex_row_hidden_overflow_item_`, `flex_replaced_automatic_minimum_`, and `fri07_c02_collapsed_output_` pass; add `fri08_remediation_flex_items_have_one_owner`. Preserve order, collapse/struts, min-size, measurement/cache, and scalars.

```sh
set -e; for f in flex_order_modified_sequence_ flex_row_hidden_overflow_item_ flex_replaced_automatic_minimum_ fri07_c02_collapsed_output_ fri08_remediation_flex_items_have_one_owner; do CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout "$f"; done
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout; CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings; cargo fmt --check; git diff --check
```

Dependency: T03. Commit: `refactor(flex): extract item phases`.

### 2.5 `P01/I08/S02/R04/T05` Flex Lines And Flexible Lengths

**Area:** `src/flex/lines.rs`, `flexible_lengths.rs`, `src/flex.rs`, `src/lib_tests.rs`, embedded owner tests.

**Outcome:** move line/round/collapsed-strut carriers and collection to `lines`; move positive/negative distribution, freezing, used space, and clamps to `flexible_lengths`. No duplicate item/line state.

**RED/acceptance:** `flex_row_wraps_`, `flex_row_distributes_positive_`, `flex_row_distributes_negative_`, and `fri07_c02_collapse_round_` pass; add `fri08_remediation_flex_line_resolution_has_one_owner`. Preserve wrap/order, violations, freeze rounds, gaps, and scalar precision.

```sh
set -e; for f in flex_row_wraps_ flex_row_distributes_positive_ flex_row_distributes_negative_ fri07_c02_collapse_round_ fri08_remediation_flex_line_resolution_has_one_owner; do CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout "$f"; done
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout; CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings; cargo fmt --check; git diff --check
```

Dependency: T04. Commit: `refactor(flex): extract line resolution`.

### 2.6 `P01/I08/S02/R04/T06` Flex Alignment And Baselines

**Area:** `src/flex/alignment.rs`, `src/flex.rs`, `src/lib_tests.rs`, embedded baseline tests.

**Outcome:** move line/item cross-axis alignment, auto margins, baseline carriers/selection, alignment offsets/fallbacks, and final item positioning to one owner.

**RED/acceptance:** `flex_row_aligns_`, `logical_flex_placement_baseline_`, `flex_row_wrap_reverse_`, and `fri05_c04_flex_alignment_` pass; add `fri08_remediation_flex_alignment_has_one_owner`. Preserve safe/unsafe fallback, bidi/writing modes, first/last baselines, wrap reverse, and scalars.

```sh
set -e; for f in flex_row_aligns_ logical_flex_placement_baseline_ flex_row_wrap_reverse_ fri05_c04_flex_alignment_ fri08_remediation_flex_alignment_has_one_owner; do CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout "$f"; done
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout; CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings; cargo fmt --check; git diff --check
```

Dependency: T05. Commit: `refactor(flex): extract alignment`.

### 2.7 `P01/I08/S02/R04/T07` Flex Intrinsic And Absolute

**Area:** `src/flex/intrinsic.rs`, `absolute.rs`, `src/flex.rs`, `src/lib_tests.rs`, embedded owner tests.

**Outcome:** move intrinsic container/item calculations and cross constants to `intrinsic`; move absolute/hidden child layout, margins, position, and alignment to `absolute`. Shared sizing remains external.

**RED/acceptance:** `fri07_c01_intrinsic_`, `flex_percent_dependent_affine_`, `flex_absolute_child_`, and `fri07_c01_absolute_auto_margin_` pass; add `fri08_remediation_flex_intrinsic_absolute_have_one_owner`. Preserve intrinsic bases, affine values, hidden children, static position, margins, errors, and scalars.

```sh
set -e; for f in fri07_c01_intrinsic_ flex_percent_dependent_affine_ flex_absolute_child_ fri07_c01_absolute_auto_margin_ fri08_remediation_flex_intrinsic_absolute_have_one_owner; do CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout "$f"; done
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout; CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings; cargo fmt --check; git diff --check
```

Dependency: T06. Commit: `refactor(flex): extract intrinsic and absolute`.

### 2.8 `P01/I08/S02/R04/T08` Flex Scroll And Final Facade

**Area:** `src/flex/scroll.rs`, delete `src/flex.rs`, create `src/flex/mod.rs`, direct imports, `src/lib_tests.rs`, `src/contract_tests.rs`, `src/root_tests.rs`, `src/flex_tests.rs` import only, embedded tests.

**Outcome:** move retained geometry, contribution carrier, canonical scroll box/contributions/subjects/publication to `scroll`; finalize composition-only `flex/mod.rs` with entry/constants/axes and intentional reexports. Add aggregate `fri08_remediation_algorithm_phase_composition_equivalence` covering block and flex.

**RED/acceptance:** `fri05_c04_flex_contribution_`, `fri08_c07_t02_scroll_source_flex_`, `fri05_c04_flex_auto_`, all earlier R04 anchors, aggregate composition, and public API inventory pass. `src/flex.rs` is absent; no copied canonical scroll/sizing policy; public and private facade contracts exact.

```sh
set -e; for f in fri05_c04_flex_contribution_ fri08_c07_t02_scroll_source_flex_ fri05_c04_flex_auto_ fri08_remediation_algorithm_phase_composition_equivalence fri08_remediation_public_api_inventory_is_compatible; do CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout "$f"; done
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri08_remediation_; CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout; CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings; cargo fmt --check; git diff --check
```

Dependency: T07. Commit: `refactor(flex): finalize phase ownership`.

## 3 Completion

R04 requires eight independently CLEAN task spans, status complete, CLEAN final matrix and holistic review, publication/readback, process hygiene, successful `cargo clean`, absent `target/`, and immutable R05 handoff. Browser, generation, acquisition, and artifact writes remain prohibited. Missing/wrong pinned cache or required behavior/API/artifact/scope expansion is a stop.

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
expected_paths="$({ printf '%s\n' plans/cycles/P01-I08-S02-R04-block-and-flex-phase-ownership.md; while IFS= read -r span; do git diff --name-only "$span"; done <<< "$TASK_SPANS"; } | LC_ALL=C sort -u)"
actual_paths="$(git diff --name-only 4f5022b720d33c1946604aeb3ce2172fd5db8fc8..HEAD | LC_ALL=C sort -u)"; test "$actual_paths" = "$expected_paths"
base_suppressions="$(while IFS= read -r source_path; do git show "4f5022b720d33c1946604aeb3ce2172fd5db8fc8:$source_path" | perl -0777 -ne 'while (/^[ \t]*#\s*\[\s*(?:allow|expect|cfg_attr)\b[^\]]*\]/gms) { $m = $&; $m =~ s/\s+/ /g; print "$m\n" }'; done < <(git ls-tree -r --name-only 4f5022b720d33c1946604aeb3ce2172fd5db8fc8 | rg '\.rs$') | LC_ALL=C sort)"
current_suppressions="$({ git ls-files -z -- '*.rs'; git ls-files -z --others --exclude-standard -- '*.rs'; } | sort -zu | xargs -0 perl -0777 -ne 'while (/^[ \t]*#\s*\[\s*(?:allow|expect|cfg_attr)\b[^\]]*\]/gms) { $m = $&; $m =~ s/\s+/ /g; print "$m\n" }' | LC_ALL=C sort)"; test -z "$(comm -13 <(printf '%s\n' "$base_suppressions") <(printf '%s\n' "$current_suppressions"))"
unsafe_hits="$({ git ls-files -z -- '*.rs'; git ls-files -z --others --exclude-standard -- '*.rs'; } | sort -zu | xargs -0 rg -n '\bunsafe\b' | rg -v 'safe_fallback returns unsafe|Some\("async" \| "unsafe" \| "default" \| "extern"\)|removed phase-unsafe surface remains|starts_with\("unsafe "\)|strip_prefix\("unsafe "\)|parse_align_content\("unsafe end"\)|parse_align_items\("unsafe first baseline"\)' || true)"; test -z "$unsafe_hits"
if { git ls-files -z -- '*.rs'; git ls-files -z --others --exclude-standard -- '*.rs'; } | sort -zu | xargs -0 rg -n -U '\b(no_mangle|export_name|link_section|naked)\b|(^|[^[:alnum:]_\"])extern[[:space:]]*"'; then exit 1; fi
test "$(shasum -a 256 tests/layout/browser_parity/corpus.toml | awk '{print $1}')" = c6e6f1422e14a5e4aa474c143998063ce0de4d0a9123b69875b35a4ed009a8f6
test "$(shasum -a 256 tests/layout/browser_parity/scripts/gentest/test_helper.js | awk '{print $1}')" = c684c7f167d95997a4a9f0250467bbaf72c1b73e69e0f707a2ef32f4d25f7f36
test "$(shasum -a 256 tests/layout/browser_parity/xml/generation-reports/all.json | awk '{print $1}')" = c10dc550d260a239c8bf9dd553f5272ca3bcc2826099bc182f800986b8b94c0e
test "$(find tests/layout/browser_parity/html -type f -name '*.html' | wc -l | tr -d ' ')" = 1448; test "$(find tests/layout/browser_parity/xml -type f -name '*.xml' | wc -l | tr -d ' ')" = 5776
test -z "$(git status --porcelain=v1)"
```

After publication/readback: prove no stale layout Cargo/Rust/generator process; run `cargo clean`; prove `target/` absent and Git clean. Record published SHA, reviewed revisions, eight task ranges/reviews, final evidence, remote readback, and cleanup for R05.
