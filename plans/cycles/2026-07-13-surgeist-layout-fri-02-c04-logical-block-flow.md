# FRI-02-C04 Logical Block Flow
Status: reviewed
Cycle ID: `FRI-02-C04`
Owning repository: `surgeist-layout`
Cycle base: `584f16231bed9c3e0475a4e64056fdc9e25dc2d3`
Reviewed specification:
`plans/specs/2026-07-12-surgeist-layout-fri-02-logical-geometry-writing-modes.md`
at `9f3b3587c2feaafb02c28500034b29c6d47b58f1233b6dc8f530716ce6bf17ba`,
commit `ddb23fed47297bcdd1df67f67f0ee1ac20de7876`; `D-18`, public
collapsible-margin output in `FRI-02.6`, `FRI-02.8`, block portions of
`FRI-02.12`-`FRI-02.14`, block evidence in `FRI-02.17`, and acceptance items
4-5 in `FRI-02.20`.
Reviewed sequence:
`plans/sequences/2026-07-12-surgeist-layout-fri-02-logical-geometry-writing-modes.md`
at `fbadf235adc6e38e4be2c93477a4002865c20f09e081ee5403ab56c9fac2de6a`,
commit `21e21305718fbf3273ca90044091e87d7d0c821e`, entry `C04`.
Bounded outcome: ordinary block layout and its compute-margin state follow
containing logical axes across all supported flows, with physical public output
and an exact 20-output browser matrix.

## Boundary

C04 owns the typed physical block-margin-collapse output, measured-leaf producer,
ordinary block sizing/placement/edges/collapse/baselines, existing non-clearing
inline/control projection, relative offsets and absolute static fallback touched
by ordinary flow, root/flex-root/hidden evidence, sideways fixture support, and
the block-axis corpus/report increment. It does not change float exclusion,
vertical clear, full inline formatting (`FRI-06`), overflow/scroll geometry
correctness (`FRI-05`), positioned-layout equations (`FRI-10`), flex/grid
algorithms, root/siblings, dependencies, browser pin, or launch settings.

At the base, block flow uses `inner_width`, `cursor_y`, left/right auto margins,
top/bottom collapse, physical-height leaf emptiness, and x/y placement. Parser,
generator, and helper support omit sideways modes. The corpus has 1,351 HTML,
5,048 XML, 356 unsupported tuples, and ten reports. C04 must retain schema 2,
the exact C03 launch profile and scoped-run behavior. Only the already-cached
validated `ExistingPinned` executable may run; missing/mismatched cache is a
blocker, never permission to acquire.

One `FlowAxes` remains the sole mapping owner. Parent flow owns placement,
physical margin selection, auto margins, and collapse; child flow owns sizing
and collapse-carrier provenance. Output/cache/rounding remain physical. Both
`f32` and `f64` are required. Rust 1.97 and the absolute unsafe prohibition hold.

For T1-T3, the Rust gate is exactly:
```sh
CARGO_NET_OFFLINE=true cargo check --locked -p surgeist-layout --all-targets
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout
CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
cargo fmt --check
git diff --check
```

## Impacts

Public API: intentionally breaking. Add `PhysicalBlockMarginCollapseOf<S>` and
default-scalar alias/reexports; replace `ComputeOutputOf::{top_margin,
bottom_margin,margins_can_collapse_through}` with `block_margin_collapse`; no
alias, raw-edge constructor, loose query, or conversion remains. Dependencies,
features, and MSRV: unchanged. Artifacts: add five HTML, 20 generated XML, one
scoped report, and refreshed full/existing scoped reports; final counts are
5,068/356/0/0/0 and eleven reports. Docs: public rustdoc only; README commands
remain correct. Root: archival adaptation remains C08-owned. Unsafe: none.

## Tasks

### C04-T1 - Typed Physical Block-Margin Collapse
Files/area: `src/output.rs`, `src/lib.rs`, `src/compute.rs`, `src/block.rs`,
direct output/cache/contract/leaf/block tests.
Depends on: published C03 and the recorded cycle base.
Outcome: implement the spec's private-state carrier, `NONE`, flow constructor,
physical-side lookup, and containing-flow-aware through query. Replace all three
loose fields without compatibility surface. Block producers bind edge sets and
eligibility to their own flow; measured leaves bind zero edge sets and logical
block-axis emptiness to the leaf flow. Direct carrier queries permit
parallel/opposing axes and reject orthogonal axes.
RED: named carrier and measured-leaf producer tests fail on the old top/bottom
fields, axis-free boolean, and physical-height predicate. Cover all ten direct
flow queries plus zero-block/nonzero-inline and nonzero-block/zero-inline in
`f32`/`f64`.
Acceptance: valid state is constructor-owned; non-reporting outputs use `NONE`;
all `ComputeOutputOf` consumers migrate; no public old field/accessor/alias or
`output.<old-field>` use remains. Private block locals remain T2-owned. Commands:
`for filter in physical_block_margin_collapse measured_leaf_block_margin_collapse; do CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout "$filter" -- --nocapture; done`;
then the Rust gate.
Intended commit:
`layout: type block margin collapse by flow`.

### C04-T2 - Logical Ordinary Block Sizing And Placement
Files/area: `src/geometry.rs`, `src/block.rs`, `src/block_tests.rs`.
Depends on: task T1 is task-clean.
Outcome: add only shared generic logical-size operations required by block, then
keep ordinary non-float block known/available sizes, auto inline/block sizing,
percentage-edge bases, cursor, auto margins, collapse sets, content extent, and
placement logical until one physical projection. Parent/child flow stays distinct
for parallel, opposing, and orthogonal children; missing context remains typed.
RED: named all-mode/direction stacking, own-flow sizing, percentage/collapse,
and block/measured-leaf parallel/opposing/orthogonal tests fail through public
`compute_layout`. Include normative 100x100 vertical-rl `(80,0)/(60,0)`,
vertical-lr `(0,0)/(20,0)`, both sideways progressions, and `f32`/`f64`.
Acceptance: horizontal behavior remains green; vertical/sideways inline stretch,
auto block size, 30/60 percentage-edge basis, logical alignment, and opposing/
orthogonal requests pass through public `compute_layout`; no block-local mode
table exists. Commands: focused filters `ordinary_block_flow`,
`ordinary_block_logical_sizing`, and `ordinary_block_orthogonal` via
`for filter in ordinary_block_flow ordinary_block_logical_sizing ordinary_block_orthogonal; do CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout "$filter" -- --nocapture; done`;
then the Rust gate. Intended commit: `layout: compute ordinary block flow logically`.

### C04-T3 - Logical Block Boundaries And Root Evidence
Files/area: `src/block.rs`, `src/block_tests.rs`, `src/root_tests.rs`,
`src/cache_tests.rs`; `src/compute.rs` only if direct evidence requires it.
Depends on: task T2 is task-clean.
Outcome: project current block baselines, non-clearing inline/control reports,
relative offsets, and absolute static fallback through shared flow; prove
viewport-root, flex-item-root, hidden descendants, cache identity, and physical
output/rounding across vertical/sideways/parallel/orthogonal requests.
RED: named boundary/root tests expose physical x/y assumptions, wrong line-over
selection, or context loss. Preserve the explicit vertical-clear panic evidence
and later-owned float/positioned/overflow characterizations unchanged.
Acceptance: all ten mappings and both scalar lanes pass; represented C04 paths
have no fallback/panic. Review every remaining hit from
`rg -n 'cursor_y|inner_width|in_flow_child_x|in_flow_child_available_width|with_inner_width|WritingMode::' src/block.rs`:
hits may remain only in excluded inline/float/absolute physical boundaries, never
ordinary logical state. Commands: focused filters `ordinary_block_boundaries`
and `ordinary_block_root_contexts` via
`for filter in ordinary_block_boundaries ordinary_block_root_contexts; do CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout "$filter" -- --nocapture; done`;
then the Rust gate. Intended commit:
`layout: project block flow boundaries`.

### C04-T4 - Exact Browser Block-Axis Matrix
Files/area: browser helper/parser/generator and tests; `corpus.toml`; five
`html/block/block_axes_<mode>.html`; generated block XML and all eleven reports;
`tests/layout/browser_parity.rs`.
Depends on: task T3 is task-clean.
Outcome: parse/emit all five modes; assert the normative ten-row sideways size
and logical inline-edge mapping in helper tests; add five ordinary static block
families with two unequal children and inline-start-sensitive behavior. A named
non-ignored test rejects missing/duplicate/misplaced/topology-bypassed paths and
compares the exact four variants per family through public `compute_layout`.
RED: sideways parser/helper tests and
`runs_fri_02_block_axis_families_against_surgeist_layout` fail before sources and
generated artifacts exist. Report-manifest tests fail before the 20-case entry.
Acceptance: full plus all ten exact scoped `generate-existing` runs pass using
unchanged C03 settings; `check-corpus` is browser-free; counts are 1,356 HTML,
5,068 XML, 356 unchanged unsupported tuples, and eleven reports; existing 5,048
XML bodies retain their base hash; a scoped rerun is byte-idempotent. Commands:
`CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout runs_fri_02_block_axis_families_against_surgeist_layout -- --nocapture`;
feature tests with filters `sideways_writing_mode` and `generation_report_manifest`
via `for filter in sideways_writing_mode generation_report_manifest; do CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate "$filter" -- --nocapture; done`; then the no-fetch
block, Rust gate, and feature check/test/Clippy commands below.
Intended commit:
`tests: add logical block browser parity matrix`.

## Completion

Run this exact no-fetch block from the repository root:
```sh
/bin/bash -lc 'set -euo pipefail
unset SURGEIST_BROWSER_PATH SURGEIST_BROWSER_CACHE SURGEIST_BROWSER_VERSION SURGEIST_LAYOUT_GENERATE_FILTER SURGEIST_LAYOUT_BROWSER_PARITY_ROOT
matches=$(find target/surgeist-browser -type f -path "*/mac_arm-149.0.7827.115/*/Contents/MacOS/Google Chrome for Testing" -perm -111 -print); test "$(printf "%s\n" "$matches" | sed "/^$/d" | wc -l | tr -d " ")" -eq 1; export SURGEIST_BROWSER_PATH="$matches"; test "$("$SURGEIST_BROWSER_PATH" --version | awk "{\$1=\$1; print}")" = "Google Chrome for Testing 149.0.7827.115"
run_generation() { env -u SURGEIST_BROWSER_CACHE -u SURGEIST_BROWSER_VERSION -u SURGEIST_LAYOUT_BROWSER_PARITY_ROOT CARGO_NET_OFFLINE=true SURGEIST_BROWSER_PATH="$SURGEIST_BROWSER_PATH" SURGEIST_LAYOUT_GENERATE_FILTER="$1" cargo run --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- generate-existing; }
run_generation ""; for filter in block/block_axes block/block_br_vertical block/block_calc_width_margin block/block_margin_x_percentage_intrinsic_size_self_negative block/block_margin_x_percentage_intrinsic_size_self_positive flex/flex_calc_basis_margin_gap grid/grid_calc_track_and_item_margin grid/grid_max_content_single_item_margin_percent grid/grid_min_content_flex_single_item_margin_percent grid/grid_named_template_area_generated_names; do run_generation "$filter"; done
test "$(find tests/layout/browser_parity/html -type f -name "*.html" | wc -l | tr -d " ")" -eq 1356; test "$(find tests/layout/browser_parity/xml -type f -name "*.xml" | wc -l | tr -d " ")" -eq 5068; test "$(find tests/layout/browser_parity/xml/generation-reports -type f -name "*.json" | wc -l | tr -d " ")" -eq 11
report=tests/layout/browser_parity/xml/generation-reports/all.json; test "$(jq -r ".summary.generated" "$report")" -eq 5068; test "$(jq -r ".summary.unsupported" "$report")" -eq 356; test "$(jq -r ".summary.expected_fail + .summary.quarantined + .summary.failed_to_generate" "$report")" -eq 0; test "$(jq -r ".generated|length" tests/layout/browser_parity/xml/generation-reports/block_block_axes.json)" -eq 20
unsupported_hash=$(jq -S ".unsupported | map({name, source, variant, reason}) | sort_by(.name, .source, .variant, .reason)" "$report" | shasum -a 256 | awk "{print \$1}"); test "$unsupported_hash" = c44aaae7f939ebc07341cb984ca3f040512ec4dd5462d75454b178a713492030
base_body_hash=$(git ls-tree -r --name-only 584f16231bed9c3e0475a4e64056fdc9e25dc2d3 -- tests/layout/browser_parity/xml | rg "[.]xml$" | sort | while IFS= read -r file; do printf "%s\0" "$file"; tail -n +2 "$file"; done | shasum -a 256 | awk "{print \$1}"); test "$base_body_hash" = 1f79b729937f0e239619ff8e18e6fab080b8573bcfacf04e67f6ad195f39486b
artifact_hash() { find tests/layout/browser_parity/xml -type f \( -name "*.xml" -o -path "*/generation-reports/*.json" \) -print0 | sort -z | while IFS= read -r -d "" file; do printf "%s\0" "$file"; shasum -a 256 "$file"; done | shasum -a 256 | awk "{print \$1}"; }; before=$(artifact_hash); run_generation block/block_axes; test "$before" = "$(artifact_hash)"
env -u SURGEIST_LAYOUT_BROWSER_PARITY_ROOT SURGEIST_BROWSER_PATH=/not/consulted SURGEIST_BROWSER_CACHE=/not/consulted SURGEIST_BROWSER_VERSION=wrong SURGEIST_LAYOUT_GENERATE_FILTER=wrong CARGO_NET_OFFLINE=true cargo run --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- check-corpus'
```

Then run:
```sh
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
bash -lc 'set -euo pipefail; pattern="pub (const )?(fn )?(top_margin|bottom_margin|margins_can_collapse_through)\\b|output\\.(top_margin|bottom_margin|margins_can_collapse_through)\\b"; if rg -n "$pattern" src/output.rs src/lib.rs src/compute.rs src/block.rs; then exit 1; else test "$?" -eq 1; fi; if rg -n "WritingMode::" src/block.rs; then exit 1; else test "$?" -eq 1; fi'
bash -lc 'set -euo pipefail; files=(); while IFS= read -r -d "" file; do files+=("$file"); done < <(git ls-files -z --cached --others --exclude-standard -- "*.rs"); test "${#files[@]}" -gt 0; if rg -n --pcre2 '\''#\s*!?\s*\[[^]]*(?:unsafe\s*\(|\b(?:no_mangle|export_name|link_section|naked)\b|\b(?:allow|expect)\s*\([^]]*\b(?:unsafe_code|unsafe_op_in_unsafe_fn)\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{'\'' "${files[@]}"; then exit 1; else test "$?" -eq 1; fi'
```

Do not run or claim the ignored full-corpus aggregate; `FRI-13` owns it.
After cycle commits, require `test -z "$(git status --porcelain)"` and run
`git status --short --branch`.
Required handoff: C05 receives typed
physical collapse state, completed logical block flow, 5,068-output full report,
and eleven-report inventory. Genuine blockers: missing/mismatched cached pin,
unexpected existing XML-body or unsupported-tuple drift, any later-finding
expectation rewrite, non-idempotence, unsafe, or an unreviewed public surface.
