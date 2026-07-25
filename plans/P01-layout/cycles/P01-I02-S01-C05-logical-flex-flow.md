# P01-I02-S01-C05 Logical Flex Flow
Status: complete
Cycle ID: `P01/I02/S01/C05`
Owning repository: surgeist-layout
Cycle base: c806dac4c55a1f83fc93fad4d5d234ceb37543337d27891b7901b87ff736e15b
Reviewed specification: plans/P01-layout/initiatives/P01-I02-logical-geometry-writing-modes.md at 0a666f8f698703cd7979194a7f75f834e4c9b522, commit ddb23fed47297bcdd1df67f67f0ee1ac20de7876.
Sections: D-15, D-18, FRI-02.9, flex portions of FRI-02.12-FRI-02.14, FRI-02.17, and acceptance item 6 in FRI-02.20.
Reviewed sequence: plans/P01-layout/sequences/P01-I02-S01-logical-geometry-writing-modes.md at fbadf235adc6e38e4be2c93477a4002865c20f09e081ee5403ab56c9fac2de6a, commit 21e21305718fbf3273ca90044091e87d7d0c821e, entry C05.
Bounded outcome: all current flex main/cross computation and physical output
projection derive from one crate-private FlexAxes, while established flex
equations retain their current non-axis behavior and the corpus gains its exact
80-output non-leaf matrix.
## 1 Boundary
C05 owns src/flex.rs and its direct/public tests, removal of the listed obsolete
flex/geometry APIs, and flex-axis browser fixtures, manifest, reports, and
generated XML. FlexAxes derives only from FlowAxes, FlexDirection, and FlexWrap:
row selects inline, column selects block, *-reverse flips main once, and
wrap-reverse flips cross once. It selects all flex physical axes, sides,
point/size/edge access, progression, margins, insets, and baseline axes.
Direction reaches row only through FlowAxes; it never changes column block
progression. No second WritingMode table is valid.
Do not alter FRI-07 flex equations, FRI-09 missing alignment/distribution,
FRI-10 positioned equations, FRI-05 overflow geometry, or FRI-06 inline/clear
behavior. Preserve typed collapse/scroll, physical cache/output/rounding, root
contexts, and existing characterizations. No root/sibling/API-artifact edit,
dependency/feature/MSRV change, unsafe, compatibility alias, acquisition,
managed browser command, or ignored full-corpus aggregate is in scope. Use only
the cached ExistingPinned Chrome for Testing 149.0.7827.115; a missing or
mismatched executable is a blocker.
The Rust gate used by every Rust task is:
~~~sh
CARGO_NET_OFFLINE=true cargo check --locked -p surgeist-layout --all-targets
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout
CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
cargo fmt --check
git diff --check
~~~
After a separate clean cycle-plan review, a fresh worker creates each task's
RED-GREEN-REFACTOR changes; a distinct fresh task reviewer reviews the complete
task range and worktree. The coordinator reruns acceptance and makes the logical
commit only after CLEAN; findings use a fresh fix worker and full-task re-review.
## 2 Impacts
Public API: intentionally breaking. Remove FlexDirection main_axis/cross_axis,
context-free Point/Size/Edges main/cross construction and sums, and all flex-local
edge selector traits; add no replacement public API. Dependencies, features,
docs/examples, and Rust 1.97 MSRV: unchanged. Generated artifacts: add 20 HTML
and 80 XML, flex_flex_axes.json, and refresh the full plus all scoped reports to
1,376 HTML, 5,148 XML, 356 unsupported, zero expected-fail/quarantined/failed,
and 12 reports. C06 is the next layout leaf cycle; root archival/integration
follow-up remains C08-owned. Surgeist-owned Rust remains unsafe-free.
## 3 Tasks
### 3.1 `P01/I02/S01/C05/T01` - Canonical Flex Axes
Files/area: src/flex.rs, src/flex_tests.rs; retain src/geometry.rs and
src/node_input.rs compatibility removal for T04.
Depends on: published C04 and the recorded base.
Outcome: introduce crate-private FlexAxes at the top of the flex algorithm and
make Constants carry it. Its sole derivation uses FlowAxes logical sides and
physical projection; it exposes internal main/cross size, point, edge, side
mutation, progression, requested-axis, and aspect-ratio selection needed by
later consumers. There is no WritingMode match in flex and no duplicate mapping.
RED: flex_axes_matrix_covers_all_flows_directions_and_flex_directions names all
five modes x LTR/RTL x row/row-reverse/column/column-reverse, each with normal
(NoWrap or Wrap) and WrapReverse for 80 exact rows. It asserts exact logical
axes, physical axes/sides, and main reversal; cross reversal changes only cross
start/end/progression, leaving main mapping identical.
Acceptance: the direct matrix has 40 explicit normal/WrapReverse pairs and 80
exact rows; row RTL and vertical/sideways mappings come from FlowAxes; column
LTR/RTL has the same block progression; no represented row reaches fallback or panic.
Commands: CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout flex_axes_matrix -- --nocapture; then the Rust gate.
Coordinator commit after CLEAN: layout: derive flex axes from flow axes.
### 3.2 `P01/I02/S01/C05/T02` - Logical Flex Sizing And Line Formation
Files/area: src/flex.rs collection through intrinsic/container sizing,
src/flex_tests.rs, and public matrix support in src/root_tests.rs only when
needed for compute_layout.
Depends on: T01 task-clean.
Outcome: migrate container known/available/min/max sizes, percentage bases, flex
basis, automatic minima, intrinsic contributions, child requested axes, line
collection/wrapping, gaps, grow/shrink target dimensions, and content-main
calculation to constants.axes. Preserve each existing equation and its
FRI-07-negative-free-space behavior; migrate no alignment/output code here.
RED: public compute_layout f32/f64 tests first fail on vertical-lr sizing,
unequal basis/intrinsic items, percentage margins/gaps, four-direction wrapping,
and orthogonal fake remeasurement from a definite mapped-main known dimension.
Acceptance: real non-leaf roots retain horizontal results and prove parallel,
opposing, and orthogonal children retain their flow while selection is container-owned.
Evidence covers sizes, intrinsic/content-main, wrap/line membership, and a separate
refreshed target-size assertion; coordinates, baselines, and reverse/wrap order are
T03. No Size/Edges context-free main/cross remains in this sizing/collection range.
Commands: CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout logical_flex_sizing -- --nocapture; CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout logical_flex_intrinsic -- --nocapture; then the Rust gate.
Coordinator commit after CLEAN: layout: size flex lines through logical axes.
### 3.3 `P01/I02/S01/C05/T03` - Logical Flex Placement And Projection
Files/area: remaining line resolution/alignment/baseline/final-layout/absolute
paths in src/flex.rs, with focused src/flex_tests.rs and src/root_tests.rs.
Depends on: T02 task-clean.
Outcome: route line and item placement, auto margins, justify/align/content/self,
baseline selection/synthesis, relative offsets, final physical point projection,
visible content extent, scroll/output contribution, and existing absolute/static
projection through FlexAxes. Preserve the current FRI-09 and FRI-10 equations
while replacing their axis/edge lookup only.
RED: public f32/f64 tests expose the normative vertical-lr row, wrong reverse/
wrap-reverse order, mapped margin/inset/relative/absolute/rounded sides, and the
corrected orthogonal refresh's size-dependent physical-x baseline/placement.
Acceptance: alignment and baselines select axes through FlexAxes; reverse and
wrap-reverse compose once; output and rounding stay physical; all tested five
modes, directions, and parallel/opposing/orthogonal children complete without
placeholder, fallback, todo, unreachable, or panic.
Commands: CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout logical_flex_placement -- --nocapture; CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout logical_flex_boundaries -- --nocapture; then the Rust gate.
Coordinator commit after CLEAN: layout: project flex placement through logical axes.
### 3.4 `P01/I02/S01/C05/T04` - Public Context Evidence And Axis API Removal
Files/area: src/geometry.rs, src/node_input.rs, obsolete tail helpers in
src/flex.rs, src/contract_tests.rs, src/flex_tests.rs, src/root_tests.rs, and
src/cache_tests.rs only for direct cache identity evidence.
Depends on: T03 task-clean.
Outcome: delete, without aliases, FlexDirection main_axis/cross_axis, Point/Size/
Edges main/cross/from-cross/sum APIs, PointExt, SizeExt, EdgeAxisExt,
BoolEdgeAxisExt, OptionEdgeAxisExt, and any obsolete physical row/column
selector. Complete public compute_layout f32/f64 root, flex-item-root,
hidden-descendant, cache-key, unrounded/final rounding, and physical-output
evidence for the migrated flex path.
RED: compile/API contract tests fail while old public calls exist; public
all-flow tests fail before root/flex-root cache-context and hidden-child
propagation retain FlowAxes through a non-leaf flex container.
Acceptance: no old public symbol, local trait, or compatibility bridge remains;
cache identity distinguishes containing flow; viewport and flex-item roots,
hidden children, and rounded/unrounded output remain physical and correct in both
scalar lanes. Review every FlexDirection hit in flex: only FlexAxes derivation
may interpret its four variants.
Commands: CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout logical_flex_public_contexts -- --nocapture; CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout contract -- --nocapture; then the Rust gate.
Coordinator commit after CLEAN: layout: remove physical flex axis helpers.
### 3.5 `P01/I02/S01/C05/T05` - Exact Non-Leaf Flex Browser Matrix
Files/area: tests/layout/browser_parity.rs, tests/layout/browser_parity/corpus.toml,
tests/bin/surgeist-layout-generate/generator.rs for manifest/report-inventory test assertions and count updates,
20 html/flex/flex_axes_<mode>_<flex_direction>.html files, their 80 generator-produced XML files,
xml/generation-reports/all.json, and flex_flex_axes.json. Production parser/generator/helper/browser resolution/launch logic remains unchanged unless a later review-proven requirement says otherwise; this plan identifies no such requirement.
Depends on: T04 task-clean.
Outcome: register exactly the 20 names from the reviewed spec, each with a named
mode/direction flex root, at least two element children, unequal physical sizes,
and non-leaf topology that reaches flex. Add the named non-ignored
runs_fri_02_flex_axis_families_against_surgeist_layout inventory/comparison test
for exactly four variants each and topology rejection tests for missing,
duplicate, misplaced, leaf-lowered, and bypassed paths.
RED: the named inventory test and generator report-manifest assertions for
flex/flex_axes fail before source fixtures, count updates, and generated outputs
exist; topology fixtures using a text leaf or fewer than two element children
fail before comparison.
Acceptance: all 80 parse and compare through public compute_layout; generator
report-manifest assertions prove the scoped report/count update; report scope is
exactly flex/flex_axes -> flex_flex_axes.json -> 80; the unchanged 5,068 XML
bodies hash to c5cc90a358c481517457c1b166ca996008d21fe89fb4d9462daee553541674d3,
unsupported tuples hash to c44aaae7f939ebc07341cb984ca3f040512ec4dd5462d75454b178a713492030,
helper hash remains 298fb04ffd4811de3871977c350ecfd3e66a60a2eb7cdf6da9810503fed7853c,
and launch hash remains 9e2b5a4850e8d5ae31cf133c30f7129f1e214705f7a848697ca42c7c1b7551cb.
Commands: CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout runs_fri_02_flex_axis_families_against_surgeist_layout -- --nocapture; CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate flex_axes -- --nocapture; then the completion browser block, feature Rust gate, and Rust gate.
Coordinator commit after CLEAN: tests: add logical flex browser parity matrix.

## 4 Completion

Run this exact no-fetch browser block from the repository root. It must use the
one cached executable and does not run the ignored aggregate parity test:
~~~sh
/bin/bash -lc 'set -euo pipefail
unset SURGEIST_BROWSER_PATH SURGEIST_BROWSER_CACHE SURGEIST_BROWSER_VERSION SURGEIST_LAYOUT_GENERATE_FILTER SURGEIST_LAYOUT_BROWSER_PARITY_ROOT
matches=$(find target/surgeist-browser -type f -path "*/mac_arm-149.0.7827.115/*/Contents/MacOS/Google Chrome for Testing" -perm -111 -print); test "$(printf "%s\n" "$matches" | sed "/^$/d" | wc -l | tr -d " ")" -eq 1; export SURGEIST_BROWSER_PATH="$matches"; test "$("$SURGEIST_BROWSER_PATH" --version | awk "{\$1=\$1; print}")" = "Google Chrome for Testing 149.0.7827.115"
run_generation() { env -u SURGEIST_BROWSER_CACHE -u SURGEIST_BROWSER_VERSION -u SURGEIST_LAYOUT_BROWSER_PARITY_ROOT CARGO_NET_OFFLINE=true SURGEIST_BROWSER_PATH="$SURGEIST_BROWSER_PATH" SURGEIST_LAYOUT_GENERATE_FILTER="$1" cargo run --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- generate-existing; }
run_generation ""; for filter in block/block_axes block/block_br_vertical block/block_calc_width_margin block/block_margin_x_percentage_intrinsic_size_self_negative block/block_margin_x_percentage_intrinsic_size_self_positive flex/flex_calc_basis_margin_gap flex/flex_axes grid/grid_calc_track_and_item_margin grid/grid_max_content_single_item_margin_percent grid/grid_min_content_flex_single_item_margin_percent grid/grid_named_template_area_generated_names; do run_generation "$filter"; done
test "$(find tests/layout/browser_parity/html -type f -name "*.html" | wc -l | tr -d " ")" -eq 1376; test "$(find tests/layout/browser_parity/xml -type f -name "*.xml" | wc -l | tr -d " ")" -eq 5148; test "$(find tests/layout/browser_parity/xml/generation-reports -type f -name "*.json" | wc -l | tr -d " ")" -eq 12
report=tests/layout/browser_parity/xml/generation-reports/all.json; test "$(jq -r ".summary.generated" "$report")" -eq 5148; test "$(jq -r ".summary.unsupported" "$report")" -eq 356; test "$(jq -r ".summary.expected_fail + .summary.quarantined + .summary.failed_to_generate" "$report")" -eq 0; test "$(jq -r ".generated|length" tests/layout/browser_parity/xml/generation-reports/flex_flex_axes.json)" -eq 80
test "$(jq -r ".metadata.helper_sha256" "$report")" = 298fb04ffd4811de3871977c350ecfd3e66a60a2eb7cdf6da9810503fed7853c; test "$(jq -r ".metadata.launch_profile_sha256" "$report")" = 9e2b5a4850e8d5ae31cf133c30f7129f1e214705f7a848697ca42c7c1b7551cb; test "$(shasum -a 256 tests/layout/browser_parity/scripts/gentest/test_helper.js | awk "{print \$1}")" = 298fb04ffd4811de3871977c350ecfd3e66a60a2eb7cdf6da9810503fed7853c
unsupported_hash=$(jq -S ".unsupported | map({name, source, variant, reason}) | sort_by(.name, .source, .variant, .reason)" "$report" | shasum -a 256 | awk "{print \$1}"); test "$unsupported_hash" = c44aaae7f939ebc07341cb984ca3f040512ec4dd5462d75454b178a713492030
base_body_hash=$(git ls-tree -r --name-only e0cf26b513711e030e0aec14715117c53eb3405b -- tests/layout/browser_parity/xml | rg "[.]xml$" | sort | while IFS= read -r file; do printf "%s\0" "$file"; tail -n +2 "$file"; done | shasum -a 256 | awk "{print \$1}"); test "$base_body_hash" = c5cc90a358c481517457c1b166ca996008d21fe89fb4d9462daee553541674d3
artifact_hash() { find tests/layout/browser_parity/xml -type f \( -name "*.xml" -o -path "*/generation-reports/*.json" \) -print0 | sort -z | while IFS= read -r -d "" file; do printf "%s\0" "$file"; shasum -a 256 "$file"; done | shasum -a 256 | awk "{print \$1}"; }; before=$(artifact_hash); run_generation flex/flex_axes; test "$before" = "$(artifact_hash)"
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
bash -lc 'set -euo pipefail; pattern="pub(?:\\([^)]*\\))?\\s+(?:const\\s+)?fn\\s+(?:main_axis|cross_axis|main|cross|from_cross|main_sum|cross_sum)\\b|(?:pub(?:\\([^)]*\\))?\\s+)?trait\\s+(?:PointExt|SizeExt|EdgeAxisExt|BoolEdgeAxisExt|OptionEdgeAxisExt)\\b|(?:pub(?:\\([^)]*\\))?\\s+)?fn\\s+flex_cross_axis\\b"; if rg -n --pcre2 "$pattern" src/geometry.rs src/node_input.rs src/flex.rs; then exit 1; else test "$?" -eq 1; fi; if rg -n "WritingMode::" src/flex.rs; then exit 1; else test "$?" -eq 1; fi'
bash -lc 'set -euo pipefail; files=(); while IFS= read -r -d "" file; do files+=("$file"); done < <(git ls-files -z --cached --others --exclude-standard -- "*.rs"); test "${#files[@]}" -gt 0; if rg -n --pcre2 '\''#\s*!?\s*\[[^]]*(?:unsafe\s*\(|\b(?:no_mangle|export_name|link_section|naked)\b|\b(?:allow|expect)\s*\([^]]*\b(?:unsafe_code|unsafe_op_in_unsafe_fn)\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{'\'' "${files[@]}"; then exit 1; else test "$?" -eq 1; fi'
~~~

After all task ranges are clean, make the separate status-only completion commit,
run the full commands on that exact head, obtain a distinct clean-context holistic
review, rerun the full commands after a CLEAN verdict, and require
test -z "$(git status --porcelain)" plus git status --short --branch.
Handoff to C06: FlexAxes is the only remaining flex axis selector, old
public/local helpers are absent, and C05 corpus state is 5,148 XML with the
12-report inventory. Genuine blockers are a missing/mismatched cached pin,
unexpected hash/count/tuple drift, non-idempotence, any represented-mode panic,
unsafe, an unreviewed public removal, or a requirement that changes a
later-owned equation.
