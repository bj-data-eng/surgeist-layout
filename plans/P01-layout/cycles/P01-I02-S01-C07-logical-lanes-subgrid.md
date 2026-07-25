# P01-I02-S01-C07 Logical Lanes And Subgrid
Status: complete
Cycle ID: `P01/I02/S01/C07`
Owning repository: surgeist-layout
Cycle base: c806dac4c55a1f83fc93fad4d5d234ceb37543337d27891b7901b87ff736e15b
Reviewed specification: plans/P01-layout/initiatives/P01-I02-logical-geometry-writing-modes.md at 0a666f8f698703cd7979194a7f75f834e4c9b522, commit ddb23fed47297bcdd1df67f67f0ee1ac20de7876.
Sections: lanes/subgrid behavior in FRI-02.10; their fixture matrices and C07
cumulative report state in FRI-02.13; corresponding rows of FRI-02.14; grid
evidence in FRI-02.17; verification in FRI-02.18; GRID-004 closure portion of
FRI-02.19; remainder of acceptance item 7 and applicable artifact/safety items
in FRI-02.20.
Reviewed sequence: plans/P01-layout/sequences/P01-I02-S01-logical-geometry-writing-modes.md at fbadf235adc6e38e4be2c93477a4002865c20f09e081ee5403ab56c9fac2de6a, commit 21e21305718fbf3273ca90044091e87d7d0c821e, entry C07.
Bounded outcome: Grid-lanes and subgrid inheritance, offsets, areas, and baseline
projection preserve logical column/row identity across parent and child flows.
Exit evidence: Parallel, opposing, orthogonal, inherited-track, area, and baseline
evidence passes with exact 36-output grid-lanes and 36-output subgrid browser
matrices; both manifest entries and the refreshed full report validate; GRID-004
is closed without absorbing FRI-08 defects.

## 1 Boundary
C06 left logical ordinary-grid tracks and a single `FlowAxes` projection boundary.
`GridAxisKind::Column` is `LogicalAxis::Inline` and `Row` is `LogicalAxis::Block`;
this existing owner remains the only grid role mapping. Current C07-owned bridges
are `grid_sizing_flow_axes` forcing horizontal axes for lanes/inherited contexts,
the inherited RTL column adjustment, and `LegacyPhysicalGridLanes` absolute-area
routing. C07 removes those bridges rather than preserving or replacing them.
Current correction evidence: removing the sizing fallback and RTL adjustment fixes
the `VerticalRl`/LTR inherited size from `121x77` to `77x121`, but exposes the
absolute/static lane mismatch `(7.75, 41.5)` versus `(7.75, 39.25)` because
`lanes.rs` and `LegacyPhysicalGridLanes` still consume physical offsets and areas.
Those routes are atomic: no red commit or compatibility bridge is permitted.
`src/grid/lanes.rs` still sizes, offsets, areas, child inputs, content extents, and
baselines physically; `subgrid.rs`, `tracks.rs`, and `child.rs` still carry
physical axis gaps, edges, available sizes, traversal data, and inherited
baselines. Child compute input is projected only at the child's own flow boundary;
shared output, cache, and geometry stay physical, and `f64` never narrows to `f32`.
For an orthogonal subgrid whose inherited parent gap is `7px` and own mapped gap
is `11px`, CSS Grid Level 2, WPT `grid-gap-008/009`, and Blink 149 agree that the
half-gutter adjustment yields `48px` and `58px` tracks inside the `117px` span.
Preserve C01-C06 behavior, especially C06 ordinary-grid evidence. Use fixed,
definite, non-overlapping tracks for axis tests. Do not absorb GRID-001/002/003/
005/006/007/008/010, any other FRI-08 defect, authored CSS/style resolution,
identity, text shaping, rendering, root adapters/API artifacts, compatibility
aliases, temporary duplicate models, unsafe, dependency/feature/MSRV changes,
generator parser/resolver/launch/batch/retry/helper changes, acquisition, or a
managed-browser invocation. The sole fixture-support exception projects the
already-parsed CSS row/column gap pair through the fixture node's `FlowAxes`;
it adds no syntax or resolution behavior. C08 alone prunes temporary reports.

## 2 Impacts
Public API, dependencies, features, docs/examples, and Rust 1.97 MSRV: unchanged.
Crate-private lane and inherited-axis carriers become logical; no compatibility
surface is retained. Generated artifacts add exactly 18 HTML, 72 XML,
`grid-lanes_grid_lanes_axes.json`, and `subgrid_subgrid_axes.json`; retain all
13 current report entries/files for 1,403 HTML, 5,256 XML, and 15 reports. Root
follow-up is the reviewed C08 sequence handoff. Owned Rust remains unsafe-free.

## 3 Tasks
### 3.1 `P01/I02/S01/C07/T01` - Logical Grid-Lanes And Inherited-Axis Projection
Files/area: `src/grid/axis.rs`, `mod.rs`, `lanes.rs`, `child.rs`, `tracks.rs`,
`placement.rs`, and focused grid/root/cache tests.
Depends on: published C06 and the recorded base.
Outcome: carry inherited tracks, gaps, offsets, bases, and parent/child axis identity,
and lane intrinsic/available sizing, reruns, gaps, tracks, offsets, areas, alignment,
child compute inputs, content extents, baselines, and absolute/static placement through
logical roles until each owning `FlowAxes` projection. Remove `grid_sizing_flow_axes`,
`inherited_rtl_column_line_adjustment`, obsolete `column_line_offset_adjustment`, and
the complete `LegacyPhysicalGridLanes` route without a replacement bridge.
RED: the recorded `logical_inherited_grid_axis_contexts_f32` and `_f64` RED is
`VerticalRl`/LTR inherited physical size `121x77` instead of `77x121`. Before
remaining lane implementation, new/adjusted `logical_grid_lanes_axes_f32` and `_f64`
run RED against the current partial worktree; they cover all five modes, both
directions, both lane axes, unequal totals, parallel/opposing/orthogonal flows,
intrinsic child measurement, definite areas/offsets, content/baselines, and
absolute/static placement through public behavior.
Acceptance: all five modes and both directions preserve Column=Inline/Row=Block;
vertical/sideways `70x110` logical totals physically yield `110x70`; only named
`FlowAxes` projections cross physical boundaries; output/cache geometry remains
physical; f64 stays scalar-generic; C06 ordinary-grid and lane behavior outside the
fixed-axis scope retain their result; no second mapping/carrier or legacy route
remains. The full Rust gate passes before this combined task is reviewed or committed.
Commands: `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout logical_inherited_grid_axis_contexts -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout logical_grid_lanes_axes -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout logical_ordinary_grid -- --nocapture`; then the Rust gate below.
Coordinator commit after CLEAN: `layout: project inherited grid lanes through logical axes`.

### 3.2 `P01/I02/S01/C07/T02` - Logical Subgrid Inheritance And Projection
Files/area: `src/grid/subgrid.rs`, inherited/traversal and rerun consumers in
`tracks.rs` and `child.rs`, plus focused grid/root/cache tests, including the
auto-sized orthogonal parent and mapped-gap traversal regressions.
Depends on: T01 task-clean.
Outcome: make mapping reports, traversal carriers, inherited contexts, parent and
child track spans, gaps, edge MBP, available-area bases, offsets, and inherited
baselines logical until each owning `FlowAxes` projection; retain existing subgrid
eligibility and error behavior.
RED: `logical_subgrid_axes_f32` and `_f64` fail for columns- and rows-subgrid
with unequal explicit inherited tracks, axis swap/progression reversal, and
parallel, opposing, horizontal-parent/vertical-child, and vertical-parent/
horizontal-child flows.
Acceptance: subgrid child compute inputs use the child's physical flow projection,
while inheritance remains parent logical-axis correct; final item area, offset, and
baseline are physical and mapped; child-local cross-flow content does not become
parent demand on a non-inherited physical axis; traversal selects `style.gap`
through the subgrid's logical axes; no C07 flow panics or silently becomes horizontal;
no FRI-08 placement, demand, track-sizing, auto-fit, named-line, or traversal
outcome becomes expected behavior.
Commands: `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout logical_subgrid_axes -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout resolved_subgrid_axis_gap_uses_node_logical_axes -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout logical_grid_lanes_axes -- --nocapture`; then the Rust gate below.
Coordinator commit after CLEAN: `layout: inherit subgrid axes logically`.

### 3.3 `P01/I02/S01/C07/T03` - Preserve Indefinite Orthogonal Auto Child Sizing
Files/area: `src/block.rs` and focused block/root tests. Depends on: T02 task-clean.
Outcome: a parent block's auto-derived final physical extent remains provisional
when it maps to an orthogonal child's auto inline axis; only genuinely definite
parent context becomes child known/definite input.
RED: a public f32/f64 matrix over four vertical/sideways modes and both directions
sizes two orthogonal auto grid/subgrid siblings as `117x162` each instead of `117x81`.
Acceptance: the root remains `117x162`, siblings are `117x81` at y `0/81`, explicit
parent heights retain current behavior, and no FRI-08 grid behavior changes.
Commands: `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout orthogonal_auto_child_inline_size_remains_indefinite -- --nocapture`; then the Rust gate below.
Coordinator commit after CLEAN: `layout: preserve orthogonal auto child sizing`.

### 3.4 `P01/I02/S01/C07/T04` - Exact Lanes And Subgrid Browser Matrices
Files/area: `tests/layout/browser_parity.rs`, `support.rs` (only logical CSS-gap
projection and focused tests), `tests/bin/surgeist-layout-generate/generator.rs`
(report inventory/count assertions only), `corpus.toml`, 18 constrained HTML
fixtures, their 72 generator-produced XML files, both new scoped reports, and all
refreshed current reports. Generator parser/runtime, helper, resolver, browser
resolution/launch, batch/retry, job lifecycle, and locking remain unchanged.
Depends on: T03 task-clean and the sole cached ExistingPinned Chrome for Testing
149.0.7827.115.
Outcome: add these four-variant (`border_box_ltr`, `border_box_rtl`,
`content_box_ltr`, `content_box_rtl`) grid-lanes families: `grid_lanes_axes_horizontal_tb_parallel`,
`grid_lanes_axes_vertical_rl_parallel`, `grid_lanes_axes_vertical_lr_parallel`,
`grid_lanes_axes_sideways_rl_parallel`, `grid_lanes_axes_sideways_lr_parallel`,
`grid_lanes_axes_vertical_opposing`, `grid_lanes_axes_sideways_opposing`,
`grid_lanes_axes_horizontal_parent_orthogonal_child`, and
`grid_lanes_axes_vertical_parent_orthogonal_child`; each HTML has one
columns-lanes and one rows-lanes container with unequal logical totals and the
named flow relation. Add these subgrid families: `subgrid_axes_horizontal_tb_parallel`,
`subgrid_axes_vertical_rl_parallel`, `subgrid_axes_vertical_lr_parallel`,
`subgrid_axes_sideways_rl_parallel`, `subgrid_axes_sideways_lr_parallel`,
`subgrid_axes_vertical_opposing`, `subgrid_axes_sideways_opposing`,
`subgrid_axes_horizontal_parent_orthogonal_child`, and
`subgrid_axes_vertical_parent_orthogonal_child`; each HTML has one columns-subgrid
and one rows-subgrid case with unequal inherited tracks and an item exposing swap
or progression reversal.
RED: exact path/report/count assertions and named nonignored
`runs_fri_02_grid_lanes_axis_families_against_surgeist_layout` and
`runs_fri_02_subgrid_axis_families_against_surgeist_layout` fail before the
manifest entries, topology guards, HTML, and XML exist; the orthogonal subgrid
comparison then fails `48px` versus `50px` until fixture gaps are flow-projected.
Acceptance: each owned family has an exact-path rejection test for missing,
duplicate, misplaced, and extra paths plus non-grid/wrong-grid-root, text,
absolute, hidden-only, equal-total, indefinite, overlapping, and wrong-flow
topology; each named parity test compares all 36 outputs through `compute_layout`.
Focused vertical and sideways fixture tests prove `column-gap` is logical inline,
`row-gap` is logical block, and the pair is physically stored through `FlowAxes`.
`generation_report_manifest_requires_the_exact_temporary_inventory` asserts 15
reports, 5,256 generated outputs, and exact 36-output scoped filters
`grid-lanes/grid_lanes_axes` and `subgrid/subgrid_axes`. The cumulative
artifact/report contract and serial generation predicates below pass without
pruning any of the 13 current reports.
Commands: `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout fixture_gaps_project_logical_css_axes -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout grid_lanes_axis_fixture_matrix -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout subgrid_axis_fixture_matrix -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout runs_fri_02_grid_lanes_axis_families_against_surgeist_layout -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout runs_fri_02_subgrid_axis_families_against_surgeist_layout -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate generation_report_manifest_requires_the_exact_temporary_inventory -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate grid_lanes_axes -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate subgrid_axes -- --nocapture`; then Completion.
Coordinator commit after CLEAN: `tests: add logical lanes and subgrid browser matrices`.

## 4 Completion
Every Rust task's gate is: `CARGO_NET_OFFLINE=true cargo check --locked -p surgeist-layout --all-targets`; `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout`; `CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets -- -F unsafe-code -D warnings`; `cargo fmt --check`; `git diff --check`.

Use only the cached ExistingPinned browser and invoke generation strictly serially:
```sh
/bin/bash -lc 'set -euo pipefail
unset SURGEIST_BROWSER_PATH SURGEIST_BROWSER_CACHE SURGEIST_BROWSER_VERSION SURGEIST_LAYOUT_GENERATE_FILTER SURGEIST_LAYOUT_BROWSER_PARITY_ROOT
matches=$(find target/surgeist-browser -type f -path "*/mac_arm-149.0.7827.115/*/Contents/MacOS/Google Chrome for Testing" -perm -111 -print); test "$(printf "%s\n" "$matches" | sed "/^$/d" | wc -l | tr -d " ")" -eq 1; export SURGEIST_BROWSER_PATH="$matches"
run_generation() { env -u SURGEIST_BROWSER_CACHE -u SURGEIST_BROWSER_VERSION -u SURGEIST_LAYOUT_BROWSER_PARITY_ROOT CARGO_NET_OFFLINE=true SURGEIST_BROWSER_PATH="$SURGEIST_BROWSER_PATH" SURGEIST_LAYOUT_GENERATE_FILTER="$1" cargo run --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- generate-existing; }
run_generation ""
for filter in block/block_axes block/block_br_vertical block/block_calc_width_margin block/block_margin_x_percentage_intrinsic_size_self_negative block/block_margin_x_percentage_intrinsic_size_self_positive flex/flex_calc_basis_margin_gap flex/flex_axes grid/grid_calc_track_and_item_margin grid/grid_max_content_single_item_margin_percent grid/grid_min_content_flex_single_item_margin_percent grid/grid_named_template_area_generated_names grid/grid_axes grid-lanes/grid_lanes_axes subgrid/subgrid_axes; do run_generation "$filter"; done
test "$(find tests/layout/browser_parity/html -type f -name "*.html" | wc -l | tr -d " ")" -eq 1403; test "$(find tests/layout/browser_parity/xml -type f -name "*.xml" | wc -l | tr -d " ")" -eq 5256
reports=(all.json block_block_axes.json block_block_br_vertical.json block_block_calc_width_margin.json block_block_margin_x_percentage_intrinsic_size_self_negative.json block_block_margin_x_percentage_intrinsic_size_self_positive.json flex_flex_calc_basis_margin_gap.json flex_flex_axes.json grid_grid_axes.json grid_grid_calc_track_and_item_margin.json grid_grid_max_content_single_item_margin_percent.json grid_grid_min_content_flex_single_item_margin_percent.json grid_grid_named_template_area_generated_names.json grid-lanes_grid_lanes_axes.json subgrid_subgrid_axes.json); test "$(find tests/layout/browser_parity/xml/generation-reports -maxdepth 1 -type f -name "*.json" | wc -l | tr -d " ")" -eq 15; test "$(printf "%s\n" "${reports[@]}" | sort)" = "$(find tests/layout/browser_parity/xml/generation-reports -maxdepth 1 -type f -name "*.json" -exec basename {} \; | sort)"
report=tests/layout/browser_parity/xml/generation-reports/all.json; test "$(jq -r ".summary.generated" "$report")" -eq 5256; test "$(jq -r ".summary.unsupported" "$report")" -eq 356; test "$(jq -r ".summary.expected_fail + .summary.quarantined + .summary.failed_to_generate" "$report")" -eq 0
for report in tests/layout/browser_parity/xml/generation-reports/grid-lanes_grid_lanes_axes.json tests/layout/browser_parity/xml/generation-reports/subgrid_subgrid_axes.json; do test "$(jq -r ".summary.generated" "$report")" -eq 36; test "$(jq -r ".summary.unsupported + .summary.expected_fail + .summary.quarantined + .summary.failed_to_generate" "$report")" -eq 0; done
test "$(jq -r ".metadata.helper_sha256" tests/layout/browser_parity/xml/generation-reports/all.json)" = 298fb04ffd4811de3871977c350ecfd3e66a60a2eb7cdf6da9810503fed7853c; test "$(jq -r ".metadata.launch_profile_sha256" tests/layout/browser_parity/xml/generation-reports/all.json)" = 9e2b5a4850e8d5ae31cf133c30f7129f1e214705f7a848697ca42c7c1b7551cb; test "$(shasum -a 256 tests/layout/browser_parity/scripts/gentest/test_helper.js | awk "{print \$1}")" = 298fb04ffd4811de3871977c350ecfd3e66a60a2eb7cdf6da9810503fed7853c
unsupported_hash=$(jq -S ".unsupported | map({name, source, variant, reason}) | sort_by(.name, .source, .variant, .reason)" tests/layout/browser_parity/xml/generation-reports/all.json | shasum -a 256 | awk "{print \$1}"); test "$unsupported_hash" = c44aaae7f939ebc07341cb984ca3f040512ec4dd5462d75454b178a713492030
base_body_hash=$(git ls-tree -r --name-only 78ed8be9cb16cf415aa45be7b40263969976c61a -- tests/layout/browser_parity/xml | rg "[.]xml$" | sort | while IFS= read -r file; do printf "%s\0" "$file"; tail -n +2 "$file"; done | shasum -a 256 | awk "{print \$1}"); test "$base_body_hash" = 327b081fc5b4215306b62b87faa263f41d7e02d929303c53484b9abdc6c1d77f
artifact_hash() { find tests/layout/browser_parity/xml -type f \( -name "*.xml" -o -path "*/generation-reports/*.json" \) -print0 | sort -z | while IFS= read -r -d "" file; do printf "%s\0" "$file"; shasum -a 256 "$file"; done | shasum -a 256 | awk "{print \$1}"; }; before=$(artifact_hash); run_generation grid-lanes/grid_lanes_axes; run_generation subgrid/subgrid_axes; test "$before" = "$(artifact_hash)"
env -u SURGEIST_LAYOUT_BROWSER_PARITY_ROOT SURGEIST_BROWSER_PATH=/not/consulted SURGEIST_BROWSER_CACHE=/not/consulted SURGEIST_BROWSER_VERSION=wrong SURGEIST_LAYOUT_GENERATE_FILTER=wrong CARGO_NET_OFFLINE=true cargo run --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- check-corpus'
```

Then run `CARGO_NET_OFFLINE=true cargo check --locked -p surgeist-layout --all-targets`; `CARGO_NET_OFFLINE=true cargo check --locked -p surgeist-layout --all-targets --features layout-golden-generate`; `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout`; `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate`; `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --doc`; `RUSTDOCFLAGS="-D warnings" CARGO_NET_OFFLINE=true cargo doc --locked -p surgeist-layout --no-deps`; `CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets -- -F unsafe-code -D warnings`; `CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets --features layout-golden-generate -- -F unsafe-code -D warnings`; `cargo fmt --check`; and `git diff --check`.

The mapping predicates are: `bash -lc 'set -euo pipefail; if rg -n --pcre2 "\\b(?:LegacyPhysicalGridLanes(?:Context|Axis|ContextInput)?|legacy_grid_lanes|inherited_rtl_column_line_adjustment|grid_sizing_flow_axes|column_line_offset_adjustment)\\b|FlowAxes::new\\(crate::WritingMode::HorizontalTb, crate::Direction::Ltr\\)" src/grid; then exit 1; else test "$?" -eq 1; fi'`; `bash -lc 'set -euo pipefail; files=(); while IFS= read -r -d "" file; do files+=("$file"); done < <(git ls-files -z --cached --others --exclude-standard -- "*.rs"); test "${#files[@]}" -gt 0; if rg -n --pcre2 '\''#\\s*!?\\s*\\[[^]]*(?:unsafe\\s*\\(|\\b(?:no_mangle|export_name|link_section|naked)\\b|\\b(?:allow|expect)\\s*\\([^]]*\\b(?:unsafe_code|unsafe_op_in_unsafe_fn)\\b)|\\bunsafe\\s*(?:\\{|fn\\b|trait\\b|impl\\b|extern\\b)|\\bstatic\\s+mut\\b|\\bextern\\s*(?:"[^"]*")?\\s*\\{'\'' "${files[@]}"; then exit 1; else test "$?" -eq 1; fi'`; and `test -z "$(git status --porcelain)"`.

The aggregate ignored corpus is not claimed. The two named C07 families must be
nonignored and green. After task-clean ranges and the status-only completion
transition, follow the canonical final checks, holistic review, landing, and
publication gate. The resulting candidate hands C08 only the reviewed sequence
handoff that all algorithm families are ready for initiative-wide surface and
corpus closure. Genuine blockers are a cached-pin mismatch, count/hash/tuple
drift, non-idempotence, a represented-flow panic, executable unsafe, or a required
change outside C07/into FRI-08 behavior.
