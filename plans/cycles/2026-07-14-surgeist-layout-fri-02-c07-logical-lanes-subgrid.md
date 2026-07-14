# FRI-02-C07 Logical Lanes, Subgrid, And Sealed Browser Evidence
Status: in_progress
Cycle ID: FRI-02-C07
Owning repository: surgeist-layout
Cycle base: 78ed8be9cb16cf415aa45be7b40263969976c61a
Reviewed specification: plans/specs/2026-07-12-surgeist-layout-fri-02-logical-geometry-writing-modes.md at ed20972484206e22c3b28ab27671390a218d3083adf4b2480c8a4f78a702a177, commit 0fd7f3f67a825a2176e76c83ead44f76039498e7.
Sections: lanes/subgrid behavior in FRI-02.10; fixture, report, snapshot, and
oracle contracts in FRI-02.13; corresponding rows of FRI-02.14; generator
lifecycle/artifact impact in FRI-02.16; grid/generator evidence in FRI-02.17;
verification in FRI-02.18; GRID-004 closure in FRI-02.19; acceptance items 7 and
9 plus applicable artifact/safety items in FRI-02.20.
Reviewed sequence: plans/sequences/2026-07-12-surgeist-layout-fri-02-logical-geometry-writing-modes.md at 4f101019b4d253f7e9203a71c7f8246cae3fd7f014de412590f2e601f1916b66, commit bcfbbc026fe61b02ffa2b4795a21270b7b603d51, entry C07.
Bounded outcome: grid-lanes and subgrid preserve logical column/row identity;
their exact browser matrices run through finite owned processes and publish one
sealed, generator-bound artifact snapshot.

## Boundary
C06 left logical ordinary-grid tracks and one `FlowAxes` projection boundary.
`GridAxisKind::Column` remains Inline and Row remains Block. C07 removes
`grid_sizing_flow_axes`, inherited RTL adjustment, and
`LegacyPhysicalGridLanes`; output/cache geometry stays physical and `f64` never
narrows. CSS Grid Level 2, WPT `grid-gap-008/009`, and pinned Blink 149 agree
that an orthogonal subgrid with inherited `7px` and own mapped `11px` gaps yields
`48px` and `58px` tracks in `117px`.

T1, T2, and T2A are task-clean at `36f87ac1`, `3f2da986`, and `e3f9d04b`.
The fixture candidate then exposed a real batch-24 CDP hang: only DOM polling was
bounded; protocol evaluation, launch ownership, teardown, and profile removal
were not total, and reports did not content-bind XML. Its earlier task review is
invalidated by the reviewed specification/sequence amendment. T3 first lands
that already-generated candidate without another browser run. T4 runs a browser
only after its owned-lifecycle and snapshot focused tests are green.

Preserve C01-C06 and fixed definite non-overlapping axis-test tracks. Do not
absorb FRI-08 defects, authored CSS/style, identity, text, rendering, root
adapters/API artifacts, compatibility aliases, duplicate models, unsafe,
dependencies/features/MSRV changes, pin/version/batch/argument/helper retuning,
managed-browser acquisition, or hand-edited XML. A confirmed pinned-Chrome
shortcoming follows FRI-02.13 CSS/WPT/Blink adjudication before expectations move.

## Impacts
Public layout API, dependencies, features, docs/examples, and Rust 1.97 MSRV:
unchanged. Generator-only manifest names change without aliases to
`job_timeout_ms` and `browser-job-fault`. Crate-private owned process/session,
typed failure, generator identity, and sealed snapshot models are internal.
Artifacts add 18 HTML and 72 XML, retain 1,403 HTML, 5,256 XML, and the current
15 reports, and refresh provenance lines on all 5,184 prior XML plus all reports;
prior XML bodies and 356 unsupported tuples remain exact. C08 owns final report
pruning and root owns later integration/API artifacts. Owned Rust stays unsafe-free.

## Tasks
### C07-T1 - Logical Grid-Lanes And Inherited-Axis Projection
Files/area: `src/grid/{axis,mod,lanes,child,tracks,placement}.rs` and focused
grid/root/cache tests. Depends on: published C06 and cycle base.
Outcome: carry inherited tracks, gaps, offsets, areas, lane sizing, child inputs,
content, baselines, and absolute/static placement logically until `FlowAxes`;
remove every named bridge without replacement.
RED: `logical_inherited_grid_axis_contexts_f32/f64` produced `121x77` instead of
`77x121`; `logical_grid_lanes_axes_f32/f64` covered all modes/directions, axes,
unequal totals, flow relations, measurement, areas, baselines, and positioning.
Acceptance: Column=Inline/Row=Block; `70x110` maps to `110x70` vertically;
physical output/cache and scalar genericity remain; C06 ordinary-grid remains.
Commands: `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout logical_inherited_grid_axis_contexts -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout logical_grid_lanes_axes -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout logical_ordinary_grid -- --nocapture`; then the Rust gate.
Coordinator commit after CLEAN: `layout: project inherited grid lanes through logical axes`.

### C07-T2 - Logical Subgrid Inheritance And Projection
Files/area: `src/grid/{subgrid,tracks,child}.rs` and focused grid/root/cache tests.
Depends on: T1 task-clean.
Outcome: inherited/traversal tracks, spans, gaps, edge MBP, available bases,
offsets, and baselines remain logical until the owning parent/child projection.
RED: `logical_subgrid_axes_f32/f64` covered columns/rows subgrid, unequal tracks,
swap/reversal, parallel, opposing, and both orthogonal directions.
Acceptance: child inputs use child flow; inheritance uses parent logical roles;
mapped gaps and cross-flow demand are correct; no C07 flow becomes horizontal.
Commands: `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout logical_subgrid_axes -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout resolved_subgrid_axis_gap_uses_node_logical_axes -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout logical_grid_lanes_axes -- --nocapture`; then the Rust gate.
Coordinator commit after CLEAN: `layout: inherit subgrid axes logically`.

### C07-T2A - Preserve Indefinite Orthogonal Auto Child Sizing
Files/area: `src/block.rs` and focused block/root tests. Depends on: T2 task-clean.
Outcome: a parent auto-derived physical extent stays provisional when it maps to
an orthogonal child's auto inline axis; only definite context becomes known input.
RED: an f32/f64 four-mode/two-direction matrix sized siblings `117x162` each.
Acceptance: root `117x162`, siblings `117x81` at y `0/81`, explicit heights unchanged.
Commands: `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout orthogonal_auto_child_inline_size_remains_indefinite -- --nocapture`; then the Rust gate.
Coordinator commit after CLEAN: `layout: preserve orthogonal auto child sizing`.

### C07-T3 - Exact Lanes And Subgrid Fixture Candidate
Files/area: `tests/layout/browser_parity.rs`, `support.rs`, current generator
inventory assertions, `corpus.toml`, 18 HTML, 72 generated XML, and 15 reports.
Depends on: T2A task-clean. This task does not execute a browser.
Outcome: register the exact nine four-variant grid-lanes and nine four-variant
subgrid families from FRI-02.13, flow-project parsed gaps, enforce topology, and
land the already-generated candidate as the explicit input to T4's provenance
migration. T4 removes the old launch fields and replaces all artifact metadata.
RED: path/topology matrices reject missing, duplicate, misplaced, extra,
non-grid/wrong-root, text, absolute, hidden, equal-total, indefinite, overlap,
and wrong-flow cases; orthogonal subgrid is `48px` versus old `50px` before gap
projection. Named nonignored parity tests fail before valid generated artifacts.
Acceptance: both 36-output families compare through `compute_layout`; logical CSS
gaps project correctly; exact 1,403 HTML/5,256 XML/15 reports and zero owned
failure buckets validate under the entering generator; no old XML body or 356
unsupported tuple changes and no generator process starts.
Commands: `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout fixture_gaps_project_logical_css_axes -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout grid_lanes_axis_fixture_matrix -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout subgrid_axis_fixture_matrix -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout runs_fri_02_grid_lanes_axis_families_against_surgeist_layout -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout runs_fri_02_subgrid_axis_families_against_surgeist_layout -- --nocapture`; `env -u SURGEIST_LAYOUT_BROWSER_PARITY_ROOT SURGEIST_BROWSER_PATH=/not/consulted SURGEIST_BROWSER_CACHE=/not/consulted SURGEIST_BROWSER_VERSION=wrong SURGEIST_LAYOUT_GENERATE_FILTER=wrong CARGO_NET_OFFLINE=true cargo run --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- check-corpus`; then the Rust gate.
Coordinator commit after CLEAN: `tests: add logical lanes and subgrid fixture candidate`.

### C07-T4 - Total Owned Browser Lifecycle And Sealed Matrices
Files/area: generator entry/implementation/tests, `corpus.toml` launch fields,
all 5,256 XML provenance lines, all 15 reports, and fixture corrections if needed.
Depends on: T3 task-clean and cached ExistingPinned Chrome 149.0.7827.115.
Outcome: implement FRI-02.13/.16/.17 exactly: compiled generator identity;
owned process-to-session phases; typed failures/retry; bounded child/helper cleanup;
staged full/scoped candidates; deterministic commitment/projections; `all.json`
seal last; reject old fields without dependency, alias, or profile retuning.
RED: `owned_browser_lifecycle_`, `private_temp_cleanup_`,
`generator_source_digest_`, and `artifact_snapshot_` tests fail against the
unbounded, directly-writing generator for their named missing behavior.
Acceptance: every transition/remainder is finite and exact; failures do not seal;
partial publication is inadmissible; scoped generation requires a clean baseline;
no process/removal overlaps; full plus every scope runs serially; both 36-output
families, counts, tuples, prior bodies, snapshot equality, and idempotence pass.
Commands: `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate owned_browser_lifecycle_ -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate private_temp_cleanup_ -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate generator_source_digest_ -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate artifact_snapshot_ -- --nocapture`; then Completion.
Coordinator commit after CLEAN: `tests: bound generation and seal logical browser matrices`.

## Completion
Rust gate: `CARGO_NET_OFFLINE=true cargo check --locked -p surgeist-layout --all-targets`; `CARGO_NET_OFFLINE=true cargo check --locked -p surgeist-layout --all-targets --features layout-golden-generate`; `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout`; `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate`; `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --doc`; `RUSTDOCFLAGS="-D warnings" CARGO_NET_OFFLINE=true cargo doc --locked -p surgeist-layout --no-deps`; `CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets -- -F unsafe-code -D warnings`; `CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets --features layout-golden-generate -- -F unsafe-code -D warnings`; `cargo fmt --check`; `git diff --check`.

Use only the sole cached executable and run one generator at a time:
```sh
/bin/bash -lc 'set -euo pipefail
unset SURGEIST_BROWSER_PATH SURGEIST_BROWSER_CACHE SURGEIST_BROWSER_VERSION SURGEIST_LAYOUT_GENERATE_FILTER SURGEIST_LAYOUT_BROWSER_PARITY_ROOT
matches=$(find target/surgeist-browser -type f -path "*/mac_arm-149.0.7827.115/*/Contents/MacOS/Google Chrome for Testing" -perm -111 -print); test "$(printf "%s\n" "$matches" | sed "/^$/d" | wc -l | tr -d " ")" -eq 1; export SURGEIST_BROWSER_PATH="$matches"
run_generation() { env -u SURGEIST_BROWSER_CACHE -u SURGEIST_BROWSER_VERSION -u SURGEIST_LAYOUT_BROWSER_PARITY_ROOT CARGO_NET_OFFLINE=true SURGEIST_BROWSER_PATH="$SURGEIST_BROWSER_PATH" SURGEIST_LAYOUT_GENERATE_FILTER="$1" cargo run --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- generate-existing; }
run_generation ""
for filter in block/block_axes block/block_br_vertical block/block_calc_width_margin block/block_margin_x_percentage_intrinsic_size_self_negative block/block_margin_x_percentage_intrinsic_size_self_positive flex/flex_axes flex/flex_calc_basis_margin_gap grid/grid_axes grid/grid_calc_track_and_item_margin grid/grid_max_content_single_item_margin_percent grid/grid_min_content_flex_single_item_margin_percent grid/grid_named_template_area_generated_names grid-lanes/grid_lanes_axes subgrid/subgrid_axes; do run_generation "$filter"; done
test "$(find tests/layout/browser_parity/html -type f -name "*.html" | wc -l | tr -d " ")" -eq 1403; test "$(find tests/layout/browser_parity/xml -type f -name "*.xml" | wc -l | tr -d " ")" -eq 5256; test "$(find tests/layout/browser_parity/xml/generation-reports -maxdepth 1 -type f -name "*.json" | wc -l | tr -d " ")" -eq 15
report=tests/layout/browser_parity/xml/generation-reports/all.json; test "$(jq -r ".summary.generated" "$report")" -eq 5256; test "$(jq -r ".summary.unsupported" "$report")" -eq 356; test "$(jq -r ".summary.expected_fail + .summary.quarantined + .summary.failed_to_generate" "$report")" -eq 0
snapshot=$(jq -r ".metadata.artifact_snapshot_sha256" "$report"); test "${#snapshot}" -eq 64; test "$(for file in tests/layout/browser_parity/xml/generation-reports/*.json; do jq -r ".metadata.artifact_snapshot_sha256" "$file"; done | sort -u)" = "$snapshot"; test "$(rg -o --no-filename "artifact-snapshot-sha256=\"[0-9a-f]{64}\"" tests/layout/browser_parity/xml -g "*.xml" | sort -u)" = "artifact-snapshot-sha256=\"$snapshot\""
for file in tests/layout/browser_parity/xml/generation-reports/grid-lanes_grid_lanes_axes.json tests/layout/browser_parity/xml/generation-reports/subgrid_subgrid_axes.json; do test "$(jq -r ".summary.generated" "$file")" -eq 36; test "$(jq -r ".summary.unsupported + .summary.expected_fail + .summary.quarantined + .summary.failed_to_generate" "$file")" -eq 0; done
unsupported_hash=$(jq -S ".unsupported | map({name, source, variant, reason}) | sort_by(.name, .source, .variant, .reason)" "$report" | shasum -a 256 | awk "{print \$1}"); test "$unsupported_hash" = c44aaae7f939ebc07341cb984ca3f040512ec4dd5462d75454b178a713492030
base_body_hash=$(git ls-tree -r --name-only 78ed8be9cb16cf415aa45be7b40263969976c61a -- tests/layout/browser_parity/xml | rg "[.]xml$" | sort | while IFS= read -r file; do printf "%s\0" "$file"; tail -n +2 "$file"; done | shasum -a 256 | awk "{print \$1}"); test "$base_body_hash" = 327b081fc5b4215306b62b87faa263f41d7e02d929303c53484b9abdc6c1d77f
artifact_hash() { find tests/layout/browser_parity/xml -type f \( -name "*.xml" -o -path "*/generation-reports/*.json" \) -print0 | sort -z | while IFS= read -r -d "" file; do printf "%s\0" "$file"; shasum -a 256 "$file"; done | shasum -a 256 | awk "{print \$1}"; }; before=$(artifact_hash); run_generation grid-lanes/grid_lanes_axes; run_generation subgrid/subgrid_axes; test "$before" = "$(artifact_hash)"
for dir in target/surgeist-browser-profile target/surgeist-layout-generate-staging; do if test -d "$dir"; then test -z "$(find "$dir" -mindepth 1 -print -quit)"; fi; done
env -u SURGEIST_LAYOUT_BROWSER_PARITY_ROOT SURGEIST_BROWSER_PATH=/not/consulted SURGEIST_BROWSER_CACHE=/not/consulted SURGEIST_BROWSER_VERSION=wrong SURGEIST_LAYOUT_GENERATE_FILTER=wrong CARGO_NET_OFFLINE=true cargo run --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- check-corpus'
```

Then rerun the Rust gate and these exact predicates:
`bash -lc 'set -euo pipefail; if rg -n --pcre2 "\\b(?:LegacyPhysicalGridLanes(?:Context|Axis|ContextInput)?|legacy_grid_lanes|inherited_rtl_column_line_adjustment|grid_sizing_flow_axes|column_line_offset_adjustment)\\b|FlowAxes::new\\(crate::WritingMode::HorizontalTb, crate::Direction::Ltr\\)" src/grid; then exit 1; else test "$?" -eq 1; fi'`;
`bash -lc 'set -euo pipefail; files=(); while IFS= read -r -d "" file; do files+=("$file"); done < <(git ls-files -z --cached --others --exclude-standard -- "*.rs"); test "${#files[@]}" -gt 0; if rg -n --pcre2 '\''#\\s*!?\\s*\\[[^]]*(?:unsafe\\s*\\(|\\b(?:no_mangle|export_name|link_section|naked)\\b|\\b(?:allow|expect)\\s*\\([^]]*\\b(?:unsafe_code|unsafe_op_in_unsafe_fn)\\b)|\\bunsafe\\s*(?:\\{|fn\\b|trait\\b|impl\\b|extern\\b)|\\bstatic\\s+mut\\b|\\bextern\\s*(?:"[^"]*")?\\s*\\{'\'' "${files[@]}"; then exit 1; else test "$?" -eq 1; fi'`;
`test -z "$(git status --porcelain)"`.
The feature tests contain the exact production-source assertion for absent
`Browser::{launch,wait,kill}` and `spawn_blocking`. The two C07 families are nonignored and green;
the aggregate ignored corpus remains FRI-13. After task-clean ranges, make the
status-only `complete` commit, run final checks, obtain a fresh holistic review,
publish/read back main, and hand C08 only the reviewed sequence state. Genuine
blockers are a pin mismatch, non-finite lifecycle, failed seal/check, count/hash/
tuple/body drift, non-idempotence, represented-flow panic, unsafe, confirmed
Chrome defect pending adjudication, or required FRI-08/cross-repository change.
