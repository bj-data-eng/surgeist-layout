# FRI-02-C08 Corpus, Public Surface, And Initiative Closure

Status: in_progress
Cycle ID: `FRI-02-C08`
Owning repository: `surgeist-layout`
Cycle base: `cf93e080593a5742d957db3c908eac7262f44f87`

Reviewed specification: `plans/specs/2026-07-12-surgeist-layout-fri-02-logical-geometry-writing-modes.md`
at `9f3b3587c2feaafb02c28500034b29c6d47b58f1233b6dc8f530716ce6bf17ba`,
commit `ddb23fed47297bcdd1df67f67f0ee1ac20de7876`, sections `FRI-02.13`-`FRI-02.20`.
Reviewed sequence: `plans/sequences/2026-07-12-surgeist-layout-fri-02-logical-geometry-writing-modes.md`
at `fbadf235adc6e38e4be2c93477a4002865c20f09e081ee5403ab56c9fac2de6a`,
commit `21e21305718fbf3273ca90044091e87d7d0c821e`, entry `C08`.

Bounded outcome: remove nine temporary pre-FRI-02 report entries/files, retain
the exact five scoped FRI-02 reports and 208-output union, and make the public
front door and layout-owned docs state the completed geometry, scroll, generator,
and ownership contracts.
Exit evidence: exactly six manifest-owned reports; full summary 5,256 generated,
356 unsupported, and zero failure buckets with unsupported tuples unchanged;
scoped counts 20/80/36/36/36 and an exact unique subset union; old helpers and
duplicate mappings absent; Rust 1.97, both feature states, docs, five named
browser families, and the five FRI-02 finding closures agree.

## Boundary

C01-C07 completed every FRI-02 algorithm and fixture family. The base still has
15 reports: full, five final, and nine temporary. The final reports already form
a duplicate-free 208-output subset of `all.json`; the corpus has 1,403 HTML and
5,256 XML; all required counts/hashes match. Production source has no old lanes
route, axis fallback, inherited RTL adjustment, loose collapse-through carrier,
or second writing-mode owner. Required public reexports exist. README and crate
rustdoc do not yet state the complete model.

This cycle may change only report inventory in `corpus.toml`, the three generator
inventory/pruning tests, generator-owned report files, README, and crate rustdoc.
It must not expand or refactor generator architecture. Generator implementation
changes require a focused genuine-bug RED; no parser update or new fixture is
needed or authorized. Do not change HTML/XML bodies, layout code, public symbols,
behavior, dependencies, features, MSRV, launch/helper/resolver/browser/batch/retry
contracts, imports, root, siblings, or later-FRI expectations. Never invoke
managed browser acquisition. Owned Rust remains unsafe-free.

## Impacts

Public API/behavior and dependencies/features/MSRV are unchanged. Default and
`layout-golden-generate` remain verified on the already-installed Rust 1.97.
Delete exactly nine obsolete reports; retain/regenerate only six; HTML/XML counts
remain 1,403/5,256. After publication preserve the archival `FRI-02.15` root
handoff, but do not edit or message root.

## Tasks

### C08-T1 - Final Six-Report Corpus Inventory

Files: `tests/layout/browser_parity/corpus.toml`; tests
`generation_report_manifest_requires_the_exact_temporary_inventory`,
`generation_report_manifest_pruning_keeps_manifest_reports_and_scoped_runs_isolated`,
and `generation_report_metadata_validation_accepts_current_manifests` in
`tests/bin/surgeist-layout-generate/generator.rs`; report JSON only.
Depends on: published C01-C07 and exact cycle base; T2 waits for T1 task-clean.
Outcome: manifest/report directory own `all.json`, `block_block_axes.json`,
`flex_flex_axes.json`, `grid_grid_axes.json`, `grid-lanes_grid_lanes_axes.json`,
and `subgrid_subgrid_axes.json` only; existing-pinned full generation prunes the
nine obsolete files and retained scopes regenerate independently.
RED: rename/update both 15-count inventory tests to require the final six while
the manifest still has 15; both focused tests fail. Adapt pruning from temporary
`block/block_br_vertical` to retained `block/block_axes`.
Acceptance: scoped summaries are 20/80/36/36/36 with other buckets zero; their
208 outputs are unique and a subset of full; full summary and unsupported/helper/
launch hashes match below; HTML categories are 1,159/25/219 and total 1,403; XML
is 5,256; exact filenames, browser-free freshness, and artifact idempotence pass.
Any fixture/XML-body/parser/runtime/launch/helper/resolver/lifecycle/dependency/
feature diff is a blocker.
Commands: both focused RED/GREEN tests; Artifact Gate; `CARGO_NET_OFFLINE=true
just corpus-check`; `CARGO_NET_OFFLINE=true just verify-generator`; `git diff --check`.
Coordinator commit after task-review CLEAN: `tests: finalize FRI-02 report inventory`.

### C08-T2 - Public Geometry Documentation And Closure

Files: `README.md` and crate-level rustdoc in `src/lib.rs` only.
Depends on: C08-T1 task-clean; tasks are intentionally serial so final docs name
the already-final report contract.
Outcome: both docs name public physical geometry; crate-private logical algorithm
geometry; `FlowAxes` and all five modes; used inline-direction ownership; signed
physical/flow-relative scroll ranges; managed-pinned, existing-pinned, and
browser-free modes; root ownership of lowering/adapters/live scroll/API artifacts;
and later inline, overflow, flex, grid, alignment, and positioned initiatives.
RED: the exact documentation predicate below fails on the base. This is a docs
contract RED; no artificial behavior test is required.
Acceptance: the documentation and source gates below pass; generator instructions
stay accurate; the reexport region is byte-identical to cycle base; behavior,
MSRV, and public symbols do not change; rustdoc, both feature states, five named
FRI-02 families, and the unsafe scan pass.
Coordinator commit after task-review CLEAN: `docs: close FRI-02 geometry contract`.

### C08-T2 Exact Documentation And Source Gates

```sh
/bin/bash -lc 'set -euo pipefail; crate_docs=$(mktemp); trap "rm -f \"$crate_docs\"" EXIT; rg "^//!" src/lib.rs >"$crate_docs"; for file in README.md "$crate_docs"; do for term in "public physical geometry" "crate-private logical algorithm geometry" FlowAxes HorizontalTb VerticalRl VerticalLr SidewaysRl SidewaysLr "used inline direction" "signed physical scroll" "flow-relative scroll" "managed-pinned" "existing-pinned" "browser-free" "live scroll state" "API artifacts" "later inline, overflow, flex, grid, alignment, and positioned initiatives"; do rg -Fqi "$term" "$file"; done; done'
diff -u <(git show cf93e080593a5742d957db3c908eac7262f44f87:src/lib.rs | awk '/^pub type DefaultScalar/{keep=1} /^mod block_tests;/{keep=0} keep') <(awk '/^pub type DefaultScalar/{keep=1} /^mod block_tests;/{keep=0} keep' src/lib.rs)
/bin/bash -lc 'set -euo pipefail; reject() { if rg -n --pcre2 "$1" "${@:2}"; then exit 1; else test "$?" -eq 1; fi; }; reject "\\bAxis\\b" src/geometry.rs src/node_input.rs src/output.rs src/compute.rs src/lib.rs; reject "\\b(?:ScrollOffset|ScrollOffsetOf|ScrollRange|ScrollRangeOf|InvalidScrollRange)\\b" src/scroll.rs src/lib.rs; reject "Edges::zip_inline_size|pub (?:const )?fn (?:main_axis|cross_axis)\\b" src; reject "\\b(?:LegacyPhysicalGridLanes(?:Context|Axis|ContextInput)?|legacy_grid_lanes|inherited_rtl_column_line_adjustment|grid_sizing_flow_axes|column_line_offset_adjustment)\\b" src/grid; reject "pub (?:const )?(?:fn )?(?:top_margin|bottom_margin|margins_can_collapse_through)\\b|output\\.(?:top_margin|bottom_margin|margins_can_collapse_through)\\b" src/output.rs src/lib.rs src/compute.rs src/block.rs; reject "match\\s*(?:\\([^\\n]*(?:\\bwriting_mode\\b(?!\\.is_vertical)|\\.writing_mode(?!\\.is_vertical))[^\\n]*,|(?:[A-Za-z_][A-Za-z0-9_]*\\.)?writing_mode\\s*\\{)|_\\s*=>\\s*Direction::Ltr" src --glob "!geometry.rs" --glob "!*_tests.rs"'
rustc --version | rg -q '^rustc 1\.97\.'
cargo --version | rg -q '^cargo 1\.97\.'
CARGO_NET_OFFLINE=true cargo metadata --locked --no-deps --format-version 1 | rg -q '"rust_version":"1\.97"'
```

## Artifact Gate

Use the single cached ExistingPinned Chrome 149.0.7827.115. Run full first to
prune, then the five scopes serially; rerun scopes for idempotence. Never `generate`.

```sh
/bin/bash -lc 'set -euo pipefail
unset SURGEIST_BROWSER_PATH SURGEIST_BROWSER_CACHE SURGEIST_BROWSER_VERSION SURGEIST_LAYOUT_GENERATE_FILTER SURGEIST_LAYOUT_BROWSER_PARITY_ROOT
matches=$(find target/surgeist-browser -type f -path "*/mac_arm-149.0.7827.115/*/Contents/MacOS/Google Chrome for Testing" -perm -111 -print); test "$(printf "%s\n" "$matches" | sed "/^$/d" | wc -l | tr -d " ")" -eq 1; export SURGEIST_BROWSER_PATH="$matches"
run_generation() { env -u SURGEIST_BROWSER_CACHE -u SURGEIST_BROWSER_VERSION -u SURGEIST_LAYOUT_BROWSER_PARITY_ROOT CARGO_NET_OFFLINE=true SURGEIST_BROWSER_PATH="$SURGEIST_BROWSER_PATH" SURGEIST_LAYOUT_GENERATE_FILTER="$1" cargo run --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- generate-existing; }
run_generation ""; filters=(block/block_axes flex/flex_axes grid/grid_axes grid-lanes/grid_lanes_axes subgrid/subgrid_axes); for filter in "${filters[@]}"; do run_generation "$filter"; done
artifact_hash() { find tests/layout/browser_parity/xml -type f \( -name "*.xml" -o -path "*/generation-reports/*.json" \) -print0 | sort -z | xargs -0 shasum -a 256 | shasum -a 256 | awk "{print \$1}"; }; before=$(artifact_hash); for filter in "${filters[@]}"; do run_generation "$filter"; done; test "$before" = "$(artifact_hash)"'
git diff --exit-code cf93e080593a5742d957db3c908eac7262f44f87 -- tests/layout/browser_parity/xml ':(exclude)tests/layout/browser_parity/xml/generation-reports/*.json'
test -z "$(git ls-files --others --exclude-standard -- tests/layout/browser_parity/xml | rg -v '^tests/layout/browser_parity/xml/generation-reports/[^/]+[.]json$' || true)"
/bin/bash -lc 'set -euo pipefail
dir=tests/layout/browser_parity/xml/generation-reports; expected=$(printf "%s\n" all.json block_block_axes.json flex_flex_axes.json grid_grid_axes.json grid-lanes_grid_lanes_axes.json subgrid_subgrid_axes.json | sort); actual=$(find "$dir" -maxdepth 1 -type f -name "*.json" -exec basename {} \; | sort); test "$expected" = "$actual"
full="$dir/all.json"; jq -e ".summary.generated == 5256 and .summary.unsupported == 356 and .summary.expected_fail == 0 and .summary.quarantined == 0 and .summary.failed_to_generate == 0 and (.generated|length) == 5256 and (.unsupported|length) == 356" "$full" >/dev/null
tmp=$(mktemp); trap "rm -f \"$tmp\"" EXIT; scopes=(block_block_axes.json:20 flex_flex_axes.json:80 grid_grid_axes.json:36 grid-lanes_grid_lanes_axes.json:36 subgrid_subgrid_axes.json:36); for spec in "${scopes[@]}"; do file=${spec%%:*}; count=${spec##*:}; jq -e --argjson n "$count" ".summary.generated == \$n and .summary.unsupported == 0 and .summary.expected_fail == 0 and .summary.quarantined == 0 and .summary.failed_to_generate == 0 and (.generated|length) == \$n" "$dir/$file" >/dev/null; jq -r ".generated[].output" "$dir/$file" >>"$tmp"; done
test "$(wc -l <"$tmp" | tr -d " ")" -eq 208; test "$(sort -u "$tmp" | wc -l | tr -d " ")" -eq 208; test -z "$(comm -23 <(sort -u "$tmp") <(jq -r ".generated[].output" "$full" | sort -u))"
unsupported_hash=$(jq -S ".unsupported | map({name, source, variant, reason}) | sort_by(.name, .source, .variant, .reason)" "$full" | shasum -a 256 | awk "{print \$1}"); test "$unsupported_hash" = c44aaae7f939ebc07341cb984ca3f040512ec4dd5462d75454b178a713492030
test "$(jq -r ".metadata.helper_sha256" "$full")" = 298fb04ffd4811de3871977c350ecfd3e66a60a2eb7cdf6da9810503fed7853c; test "$(shasum -a 256 tests/layout/browser_parity/scripts/gentest/test_helper.js | awk "{print \$1}")" = 298fb04ffd4811de3871977c350ecfd3e66a60a2eb7cdf6da9810503fed7853c; test "$(jq -r ".metadata.launch_profile_sha256" "$full")" = 9e2b5a4850e8d5ae31cf133c30f7129f1e214705f7a848697ca42c7c1b7551cb
html=tests/layout/browser_parity/html; test "$(find "$html" -type f -name "*.html" | wc -l | tr -d " ")" -eq 1403; test "$(find "$html" -type f -name "*.html" ! -path "*/grid-lanes/*" ! -path "*/subgrid/*" | wc -l | tr -d " ")" -eq 1159; test "$(find "$html/grid-lanes" -type f -name "*.html" | wc -l | tr -d " ")" -eq 25; test "$(find "$html/subgrid" -type f -name "*.html" | wc -l | tr -d " ")" -eq 219; test "$(find tests/layout/browser_parity/xml -type f -name "*.xml" | wc -l | tr -d " ")" -eq 5256'
CARGO_NET_OFFLINE=true just corpus-check
```

## Completion

```sh
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --doc
RUSTDOCFLAGS="-D warnings" CARGO_NET_OFFLINE=true cargo doc --locked -p surgeist-layout --no-deps
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout runs_fri_02_ -- --nocapture
cargo fmt --check
git diff --check
/bin/bash -lc 'set -euo pipefail; files=(); while IFS= read -r -d "" file; do files+=("$file"); done < <(git ls-files -z --cached --others --exclude-standard -- "*.rs"); test "${#files[@]}" -gt 0; if rg -n --pcre2 '\''#\s*!?\s*\[[^]]*(?:unsafe\s*\(|\b(?:no_mangle|export_name|link_section|naked)\b|\b(?:allow|expect)\s*\([^]]*\b(?:unsafe_code|unsafe_op_in_unsafe_fn)\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{'\'' "${files[@]}"; then exit 1; else test "$?" -eq 1; fi'
```

The ignored aggregate corpus is not claimed; its later-owned `BLOCK-014` panic
remains visible, while all five named FRI-02 families must be nonignored/green.
Completion closes `BLOCK-003`, `FLEX-001`, `GRID-004`, `OVERFLOW-004`, and
`TEST-005` through the immutable specification matrix and this completed record.
After task-clean ranges, transition to `complete`, run canonical final checks and
a fresh holistic review, publish with explicit lease SHA, read back remote main,
and record the leaf SHA and nine deferred root obligations from `FRI-02.15`.
Genuine blockers are count/hash/tuple drift, non-idempotence, named-family
failure, unsafe, generator architecture work, out-of-bound artifacts, or a later-
owned change. Preserve evidence; do not weaken, quarantine, or broaden the cycle.
