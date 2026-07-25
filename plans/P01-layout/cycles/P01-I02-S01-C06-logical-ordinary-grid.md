# P01-I02-S01-C06 Logical Ordinary Grid
Status: complete
Cycle ID: `P01/I02/S01/C06`
Owning repository: surgeist-layout
Cycle base: c806dac4c55a1f83fc93fad4d5d234ceb37543337d27891b7901b87ff736e15b
Reviewed specification: plans/P01-layout/initiatives/P01-I02-logical-geometry-writing-modes.md at 0a666f8f698703cd7979194a7f75f834e4c9b522, commit ddb23fed47297bcdd1df67f67f0ee1ac20de7876.
Sections: FRI-02.10 excluding lanes/subgrid-specific behavior, ordinary-grid
portions of FRI-02.12-FRI-02.14, grid evidence in FRI-02.17, and the
ordinary-grid portion of acceptance item 7 in FRI-02.20.
Reviewed sequence: plans/P01-layout/sequences/P01-I02-S01-logical-geometry-writing-modes.md at fbadf235adc6e38e4be2c93477a4002865c20f09e081ee5403ab56c9fac2de6a, commit 21e21305718fbf3273ca90044091e87d7d0c821e, entry C06.
Bounded outcome: ordinary grid keeps columns as logical inline tracks and rows
as logical block tracks through sizing, areas, reruns, baselines, and content
extents, projects only at physical boundaries, and gains its exact 36-output
browser matrix.
## 1 Boundary
C06 owns ordinary-grid axis carriers and behavior in src/grid/axis.rs,
placement.rs, mod.rs, tracks.rs, and child.rs; direct/public tests; and the
ordinary-grid axis fixtures, manifest entries, reports, and generated XML.
GridAxisKind maps once to LogicalAxis: Column is Inline and Row is Block.
Grid-area extents, track requirements, gaps, percentage bases, intrinsic totals,
and ordinary-grid offsets use LogicalSizeOf/LogicalPointOf rather than physical
Size/Point. FlowAxes alone maps writing mode and direction and projects child
compute inputs, final output, absolute/static areas, baselines, and content
extents.
Shared carriers consumed by grid-lanes or subgrid may receive only the mechanical
field/type adaptations required by the logical model; their inheritance,
placement, sizing, and projection behavior remains C07-owned. Do not introduce a
parallel carrier, compatibility alias, conversion/lowering layer, or second
writing-mode table. Preserve existing C07 characterizations.
Do not alter GRID-001-GRID-003, GRID-005-GRID-008, GRID-010, FRI-09 alignment,
FRI-10 positioned equations, FRI-05 overflow semantics, or unrelated grid track
algorithms. Axis-correct evidence uses explicit fixed tracks and definite,
non-overlapping placement so it does not codify those later defects.
No root/sibling/API-artifact edit, public API change, dependency/feature/MSRV
change, unsafe, acquisition, managed browser command, browser setting/batch
change, or ignored aggregate parity run is in scope. Use only cached
ExistingPinned Chrome for Testing 149.0.7827.115; absence or mismatch blocks
artifact work.
The Rust gate used by every Rust task is:
~~~sh
CARGO_NET_OFFLINE=true cargo check --locked -p surgeist-layout --all-targets
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout
CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
cargo fmt --check
git diff --check
~~~
## 2 Impacts
Public API, dependencies, features, docs/examples, and Rust 1.97 MSRV: unchanged.
The crate-private model intentionally replaces physical-shaped column/row
carriers without aliases. Generated artifacts add nine HTML and 36 XML files,
grid_grid_axes.json, and refresh the full plus all scoped reports to 1,385 HTML,
5,184 XML, 356 unsupported, zero expected-fail/quarantined/failed, and 13 reports.
C07 receives logical shared carriers and ordinary-grid projection semantics; C08
owns temporary-report cleanup and root handoff. Owned Rust remains unsafe-free.
## 3 Tasks
### 3.1 `P01/I02/S01/C06/T01` - Typed Logical Grid Carriers
Files/area: src/grid/axis.rs, placement.rs, carrier declarations and direct
callers in mod.rs/tracks.rs/child.rs, mechanical shared consumers, grid_tests.rs.
Depends on: published C05 and the recorded base.
Outcome: expose one crate-private GridAxisKind-to-LogicalAxis operation; make
GridArea size, track requirements, container gaps, and intrinsic percentage
bases explicitly inline/block typed; project a fixed-track ordinary-grid
container through FlowAxes. Mechanical C07 consumer edits preserve output.
RED: corrected vertical_rl_grid_places_distinct_rows_on_physical_x_axis expects
logical 70x110 to produce physical 110x70, and
logical_ordinary_grid_carriers_project_fixed_tracks covers horizontal and every
vertical/sideways mode in f32/f64 before the raw carriers can satisfy it.
Acceptance: no raw Size silently carries ordinary column/row totals; Column/Row
interpretation has one owner; all five modes and both directions are total; the
existing grid-lanes/subgrid suites retain their baseline.
Commands: CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout logical_ordinary_grid_carriers -- --nocapture; CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout vertical_rl_grid_places_distinct_rows -- --nocapture; then the Rust gate.
Coordinator commit after CLEAN: layout: type ordinary grid axes logically.
### 3.2 `P01/I02/S01/C06/T02` - Logical Track And Container Sizing
Files/area: ordinary-grid initialization/intrinsic/container sizing in
src/grid/mod.rs and tracks.rs, with focused grid_tests.rs and root_tests.rs.
Depends on: T01 task-clean.
Outcome: keep known/available/min/max sizes, explicit/implicit expansion bases,
gaps, intrinsic min/max inputs and totals, track sums, and final container size
logical until one FlowAxes projection. Preserve existing track equations.
RED: logical_ordinary_grid_container_sizing_f32/f64 use unequal fixed/implicit
tracks, percentage gaps, constraints, all modes, both directions, and public
compute_layout; inline=70 and block=110 always project to 70x110 horizontally
and 110x70 vertically/sideways without changing horizontal results.
Acceptance: direction does not change block progression; no width/height branch
selects column/row semantics in this range; output and rounding remain physical.
Commands: CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout logical_ordinary_grid_container_sizing -- --nocapture; then the Rust gate.
Coordinator commit after CLEAN: layout: size ordinary grid tracks logically.
### 3.3 `P01/I02/S01/C06/T03` - Logical Child Contributions And Reruns
Files/area: src/grid/tracks.rs, ordinary pre-layout sizing in child.rs, percentage
and flexible reruns in mod.rs, and focused grid_tests.rs/root_tests.rs.
Depends on: T02 task-clean.
Outcome: derive intrinsic child contributions, known/available area inputs,
percentage/flexible reruns, constrained-axis remeasurement, and baseline sizing
from logical areas; project to each child's physical compute boundary while
retaining the child's own flow and existing equations.
RED: logical_ordinary_grid_intrinsic_reruns_f32/f64 cover parallel, opposing,
and both orthogonal parent/child relationships with unequal intrinsic sizes,
definite mapped-axis reruns, percentage dependencies, and real public leaves.
Acceptance: fake and real measurement select the container's logical track role,
child inputs stay physical in the child's own flow, and f64 never narrows to f32.
Commands: CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout logical_ordinary_grid_intrinsic_reruns -- --nocapture; then the Rust gate.
Coordinator commit after CLEAN: layout: measure ordinary grid children logically.
### 3.4 `P01/I02/S01/C06/T04` - Logical In-Flow Placement And Baselines
Files/area: ordinary in-flow area/origin/alignment/baseline/final-layout/content
paths in src/grid/child.rs and placement.rs, with grid_tests.rs/root_tests.rs.
Depends on: T03 task-clean.
Outcome: retain item areas and offsets as logical points/sizes, apply existing
alignment and baseline groups on the mapped axes, project final physical
locations once, and accumulate current visible content extents physically from
the projected output.
RED: logical_ordinary_grid_in_flow_placement_f32/f64 covers all modes and
directions, parallel/opposing/orthogonal items, unequal areas, mapped margins,
baselines on both physical axes, content extents, and rounded/unrounded output.
Acceptance: represented flows cannot panic/fallback; baselines follow the
container block axis and line-over side; no later alignment/grid defect is
reclassified or weakened.
Commands: CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout logical_ordinary_grid_in_flow_placement -- --nocapture; then the Rust gate.
Coordinator commit after CLEAN: layout: project ordinary grid flow placement.
### 3.5 `P01/I02/S01/C06/T05` - Absolute Projection And Legacy Removal
Files/area: ordinary absolute/static paths in src/grid/child.rs and placement.rs,
obsolete ordinary helpers, grid_tests.rs/root_tests.rs/cache_tests.rs as needed.
Depends on: T04 task-clean.
Outcome: project absolute grid areas, static positions, insets, margins, and
final points through the same logical boundary; remove grid_area_logical_size,
horizontal_grid_axis_offsets, grid_item_block_axis_offset,
grid_item_physical_offset, and grid_area_physical_origin without aliases.
RED: logical_ordinary_grid_absolute_static_f32/f64 and
logical_ordinary_grid_public_contexts expose reversed block progression,
mapped insets/margins, viewport/flex-item roots, hidden descendants, cache
identity, and physical rounded output through public compute_layout.
Acceptance: each remaining WritingMode/is_vertical hit is justified only by a
C07 or physical-boundary consumer; ordinary grid has no local mapping table or
listed legacy helper; output/cache geometry remains physical.
Commands: CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout logical_ordinary_grid_absolute_static -- --nocapture; CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout logical_ordinary_grid_public_contexts -- --nocapture; then the Rust gate.
Coordinator commit after CLEAN: layout: project ordinary grid absolute placement.
### 3.6 `P01/I02/S01/C06/T06` - Exact Ordinary-Grid Browser Matrix
Files/area: tests/layout/browser_parity.rs and corpus.toml; generator.rs only for
report-inventory assertions/counts; nine html/grid/grid_axes_*.html; their 36
generated XML; all.json, every existing scoped report's manifest hash, and new
grid_grid_axes.json. Parser/helper/resolver/launch/batching stay unchanged.
Depends on: T05 task-clean.
Outcome: register the exact nine spec families. Each uses explicit tracks
30px+40px inline and 50px+60px block, two in-flow element children with definite
non-overlapping cells, and the named parallel/opposing/orthogonal flow relation.
Add nonignored runs_fri_02_grid_axis_families_against_surgeist_layout for the
exact four variants per family and all 36 public compute_layout comparisons.
RED: the named matrix/report assertions fail before fixtures/outputs exist;
rejection tests cover missing/duplicate/misplaced/extra paths, non-grid or
grid-lanes/subgrid roots, text/absolute/hidden-only topology, equal totals,
indefinite/overlapping placement, and wrong named flow relationship.
Acceptance: scope is exactly grid/grid_axes -> grid_grid_axes.json -> 36; totals
are 1,385/5,184/13 with unchanged 356 unsupported and zero other buckets; the
5,148 prior XML bodies hash to 68735dcdbe45273d4adda2fc818527a71d9ac0a4a8cd9b295d8944ba79b3c3b4;
unsupported/helper/launch hashes and scoped regeneration remain unchanged.
Commands: CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout runs_fri_02_grid_axis_families_against_surgeist_layout -- --nocapture; CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate grid_axes -- --nocapture; then Completion.
Coordinator commit after CLEAN: tests: add logical ordinary grid browser matrix.
## 4 Completion
Run from the repository root with the sole cached ExistingPinned executable:
~~~sh
/bin/bash -lc 'set -euo pipefail
unset SURGEIST_BROWSER_PATH SURGEIST_BROWSER_CACHE SURGEIST_BROWSER_VERSION SURGEIST_LAYOUT_GENERATE_FILTER SURGEIST_LAYOUT_BROWSER_PARITY_ROOT
matches=$(find target/surgeist-browser -type f -path "*/mac_arm-149.0.7827.115/*/Contents/MacOS/Google Chrome for Testing" -perm -111 -print); test "$(printf "%s\n" "$matches" | sed "/^$/d" | wc -l | tr -d " ")" -eq 1; export SURGEIST_BROWSER_PATH="$matches"
run_generation() { env -u SURGEIST_BROWSER_CACHE -u SURGEIST_BROWSER_VERSION -u SURGEIST_LAYOUT_BROWSER_PARITY_ROOT CARGO_NET_OFFLINE=true SURGEIST_BROWSER_PATH="$SURGEIST_BROWSER_PATH" SURGEIST_LAYOUT_GENERATE_FILTER="$1" cargo run --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- generate-existing; }
run_generation ""; for filter in block/block_axes block/block_br_vertical block/block_calc_width_margin block/block_margin_x_percentage_intrinsic_size_self_negative block/block_margin_x_percentage_intrinsic_size_self_positive flex/flex_calc_basis_margin_gap flex/flex_axes grid/grid_calc_track_and_item_margin grid/grid_max_content_single_item_margin_percent grid/grid_min_content_flex_single_item_margin_percent grid/grid_named_template_area_generated_names grid/grid_axes; do run_generation "$filter"; done
test "$(find tests/layout/browser_parity/html -type f -name "*.html" | wc -l | tr -d " ")" -eq 1385; test "$(find tests/layout/browser_parity/xml -type f -name "*.xml" | wc -l | tr -d " ")" -eq 5184; test "$(find tests/layout/browser_parity/xml/generation-reports -type f -name "*.json" | wc -l | tr -d " ")" -eq 13
report=tests/layout/browser_parity/xml/generation-reports/all.json; test "$(jq -r ".summary.generated" "$report")" -eq 5184; test "$(jq -r ".summary.unsupported" "$report")" -eq 356; test "$(jq -r ".summary.expected_fail + .summary.quarantined + .summary.failed_to_generate" "$report")" -eq 0; test "$(jq -r ".generated|length" tests/layout/browser_parity/xml/generation-reports/grid_grid_axes.json)" -eq 36
test "$(jq -r ".metadata.helper_sha256" "$report")" = 298fb04ffd4811de3871977c350ecfd3e66a60a2eb7cdf6da9810503fed7853c; test "$(jq -r ".metadata.launch_profile_sha256" "$report")" = 9e2b5a4850e8d5ae31cf133c30f7129f1e214705f7a848697ca42c7c1b7551cb
unsupported_hash=$(jq -S ".unsupported | map({name, source, variant, reason}) | sort_by(.name, .source, .variant, .reason)" "$report" | shasum -a 256 | awk "{print \$1}"); test "$unsupported_hash" = c44aaae7f939ebc07341cb984ca3f040512ec4dd5462d75454b178a713492030
base_body_hash=$(git ls-tree -r --name-only d0a8c30deb701ee31397f73b868de974c4810d31 -- tests/layout/browser_parity/xml | rg "[.]xml$" | sort | while IFS= read -r file; do printf "%s\0" "$file"; tail -n +2 "$file"; done | shasum -a 256 | awk "{print \$1}"); test "$base_body_hash" = 68735dcdbe45273d4adda2fc818527a71d9ac0a4a8cd9b295d8944ba79b3c3b4
artifact_hash() { find tests/layout/browser_parity/xml -type f \( -name "*.xml" -o -path "*/generation-reports/*.json" \) -print0 | sort -z | while IFS= read -r -d "" file; do printf "%s\0" "$file"; shasum -a 256 "$file"; done | shasum -a 256 | awk "{print \$1}"; }; before=$(artifact_hash); run_generation grid/grid_axes; test "$before" = "$(artifact_hash)"
env -u SURGEIST_LAYOUT_BROWSER_PARITY_ROOT SURGEIST_BROWSER_PATH=/not/consulted SURGEIST_BROWSER_CACHE=/not/consulted SURGEIST_BROWSER_VERSION=wrong SURGEIST_LAYOUT_GENERATE_FILTER=wrong CARGO_NET_OFFLINE=true cargo run --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- check-corpus'
~~~
Then run:
~~~sh
CARGO_NET_OFFLINE=true cargo check --locked -p surgeist-layout --all-targets
CARGO_NET_OFFLINE=true cargo check --locked -p surgeist-layout --all-targets --features layout-golden-generate
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --doc
RUSTDOCFLAGS="-D warnings" CARGO_NET_OFFLINE=true cargo doc --locked -p surgeist-layout --no-deps
CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets --features layout-golden-generate -- -F unsafe-code -D warnings
cargo fmt --check
git diff --check
bash -lc 'set -euo pipefail; if rg -n --pcre2 "\\b(?:grid_area_logical_size|horizontal_grid_axis_offsets|grid_item_block_axis_offset|grid_item_physical_offset|grid_area_physical_origin)\\b" src/grid; then exit 1; else test "$?" -eq 1; fi'
bash -lc 'set -euo pipefail; files=(); while IFS= read -r -d "" file; do files+=("$file"); done < <(git ls-files -z --cached --others --exclude-standard -- "*.rs"); test "${#files[@]}" -gt 0; if rg -n --pcre2 '\''#\s*!?\s*\[[^]]*(?:unsafe\s*\(|\b(?:no_mangle|export_name|link_section|naked)\b|\b(?:allow|expect)\s*\([^]]*\b(?:unsafe_code|unsafe_op_in_unsafe_fn)\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{'\'' "${files[@]}"; then exit 1; else test "$?" -eq 1; fi'
test -z "$(git status --porcelain)"
git status --short --branch
~~~
After task-clean ranges, make the status-only completion commit, run Completion,
obtain a distinct holistic review of the exact cycle range against both code
quality and the fallible plan, rerun Completion after CLEAN, publish main, and
verify local/tracking/remote equality.
Handoff to C07: shared carriers are logical, ordinary grid has one projection
boundary and no listed helper, and corpus state is 5,184 XML with 13 reports.
Blockers: cached-pin mismatch, count/hash/tuple drift, non-idempotence, a
represented-flow panic, unsafe, or a required change to C07/FRI-08 behavior.
