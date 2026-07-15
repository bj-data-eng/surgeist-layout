# FRI-03-C02 Bounded Fixture Schema And Corpus Baseline

Status: reviewed
Cycle ID: `FRI-03-C02`
Owning repository: `surgeist-layout`
Cycle base: `b2af2a464f4c8ad868e3b490ae16aabec2a30394`

Reviewed specification: `plans/specs/2026-07-15-surgeist-layout-fri-03-box-participation-contracts.md`
at `56efbca03febc725bee2d829da9bfdcf45f6194b24555eb22c1aa1082d9b12f2`,
commit `ad342c4526802460f89d6d02125f16e419b6f81b`, fixture/parser scope in
`FRI-03.2`, `E-PARITY`, `FRI-03.7`, `FRI-03.8`, fixture paths in `FRI-03.9`,
generator constraints in `FRI-03.11`, and artifact portions of acceptance items
7 and 8.
Reviewed sequence: `plans/sequences/2026-07-15-surgeist-layout-fri-03-box-participation-contracts.md`
at `4ad6f9ffca47d7119c487da8be09f03bdebae269debd2a20dd502323ee43bdd2`,
commit `05401beb53853a5eaf1c622050cfa0d7cebc0c4c`, entry `C02`.

Bounded outcome: the existing constrained-HTML producer and Rust fixture parser
carry exact item order and actual flex-item viewport parent axes; three active
order sources and their generated outputs establish the final FRI-03 corpus and
report baseline before any algorithm consumes that data.

## Boundary

This cycle owns only the embedded helper, its existing Rust serializer/tests,
the browser fixture parser/tests, three named constrained HTML sources,
`corpus.toml`, corpus inventory/report tests, and mechanically generated XML and
report JSON. The single already-cached ExistingPinned Chrome
`149.0.7827.115` is used through `generate-existing` with offline Cargo. No
download, managed generation, import, standalone generator, new command, module,
dependency, feature, script, schema version, launch policy, browser pin, or
hand-edited XML is allowed.

Production `src/`, public API, algorithms, caches, root/siblings, README,
`Cargo.toml`, `Cargo.lock`, `Justfile`, and task scripts do not change. C03 owns
public containing-context consumption. C05 through C07 own order-sensitive
geometry. Replaced fixtures, tag inference, natural dimensions, and object-size
modeling are explicitly out of scope; later replaced evidence remains focused
real-`LayoutTree` coverage. A confirmed genuine generator bug requires a revised
and freshly reviewed plan before any change beyond the helper/serializer/parser
work named here.

Current evidence at the cycle base: 1,403 HTML, 5,256 XML, 356 unsupported
tuples, six reports, no serialized order, and 16 flex-item roots without parent
axes. All three `just` gates are green; one cached executable is exact Chrome 149.

## Impacts
Public API, production behavior, dependencies, features, lockfile, MSRV, docs,
and examples: unchanged. Artifacts add three HTML/12 XML and mechanically refresh
provenance, 16 viewport bodies, the manifest, and ten reports. Root: C08. Unsafe: none.

## Tasks

### C02-T1 - Exact Producer And Serializer Metadata

Files: `tests/layout/browser_parity/scripts/gentest/test_helper.js`,
`tests/bin/surgeist-layout-generate/generator.rs`, existing generated XML, and
the existing six report JSON files only.
Outcome: every described element records `getComputedStyle(e).order` as its
exact string, including initial `"0"`; a flex-item viewport records the actual
computed writing-mode and direction of `e.parentElement`. XML omits zero order,
emits every nonzero signed value exactly, and emits both parent attributes on
every flex-item viewport while root viewports emit neither. Full ExistingPinned
generation refreshes helper provenance on all 5,256 outputs; only the 16 known
flex-item viewport bodies gain metadata, and the six existing reports remain the
exact manifest inventory.
RED: add
`generator::tests::bundled_helper_captures_exact_order_and_flex_parent_axes` and
`generator::tests::xml_generation_serializes_exact_order_and_parent_axes`; they
fail before helper/serializer changes. After source GREEN, `just corpus-check`
fails stale-helper freshness until authoritative generation refreshes artifacts.
Acceptance: tests cover zero, `i32::MIN`, `i32::MAX`, root omission, flex parent
horizontal/orthogonal axes, and no inference from the item root. Generator
lifecycle, launch profile, output topology, unsupported tuples, XML bodies other
than the 16 parent-attribute additions, and six-report inventory stay unchanged.
Commands: first two exact list/run gates below; Intermediate Artifact Gate;
`CARGO_NET_OFFLINE=true just verify-generator`; `CARGO_NET_OFFLINE=true just
corpus-check`; `git diff --check`.
Intended commit: `test(generator): capture FRI-03 fixture metadata`.

### C02-T2 - Strict Scalar-Independent Fixture Parsing

Files: `tests/layout/browser_parity/support.rs` only.
Depends on: C02-T1 task-review CLEAN.
Outcome: omitted order defaults to `ItemOrder::ZERO`; a canonical signed base-10
integer fitting `i32` parses directly to `ItemOrder` without a layout scalar.
`+1`, leading zeros, `-0`, fractions, exponents, text, whitespace, and overflow
fail. Root viewports reject stray parent attributes. Flex-item viewports require
both attributes, parse them through the existing strict direction/writing-mode
domains into one `FlowAxes`, and retain that value for C03; they never substitute
the root item's axes. The current one-argument public root context remains the
temporary consumer boundary and is not changed here.
RED: add
`layout::browser_parity::support::tests::item_order_parser_is_canonical_and_scalar_independent`
and
`layout::browser_parity::support::tests::viewport_parent_axes_schema_is_strict`;
they fail because order and parent axes are ignored. Update existing flex-root
parser/front-door tests to provide and assert the now-required metadata.
Acceptance: both exact tests execute once; min/zero/max and every invalid class
above are covered; all 5,256 checked-in fixtures parse; invalid metadata returns
the existing fixture `Error`; no fallback, primitive conversion, scalar order,
production error, panic, or production source change is introduced.
Commands: third and fourth exact list/run gates below; `CARGO_NET_OFFLINE=true
just verify`; `CARGO_NET_OFFLINE=true just verify-generator`;
`CARGO_NET_OFFLINE=true just corpus-check`; `git diff --check`.
Intended commit: `test(parity): parse FRI-03 fixture metadata`.

### C02-T3 - Final FRI-03 Corpus And Report Baseline

Files: the three exact HTML sources below, `corpus.toml`, final report-inventory
tests in `generator.rs` and `browser_parity.rs`, 12 new generated XML files, and
the final ten report JSON files.
Depends on: C02-T2 task-review CLEAN.
Outcome: add exactly `flex/fri03_order_modified_flex`,
`grid/fri03_order_modified_grid`, and
`grid-lanes/fri03_order_modified_lanes`. Each uses four visible fixed-size
children with source-order values `2, -1, 2, 0`, so later algorithm cycles can
prove negative/zero/positive ordering and a stable equal-order tie while source
identity remains observable. Add four scoped
reports for the three sources and
`block/block_align_baseline_child_margin_percent`; full ExistingPinned
generation creates 12 outputs and full/scoped generation establishes the final
10-report state.
RED: update/add
`generator::tests::generation_report_manifest_requires_the_exact_fri_03_inventory`,
`layout::browser_parity::browser_parity_html_corpus_inventory_is_documented`,
`layout::browser_parity::browser_parity_generation_report_counts_full_scope`,
and
`layout::browser_parity::browser_parity_generation_report_inventory_matches_fri_03_scopes`.
They fail at 1,403/5,256/six reports and missing source/output paths before the
manifest, sources, and generated artifacts are complete.
Acceptance: HTML is 1,406 (1,161 ordinary, 26 grid-lanes, 219 subgrid); XML is
5,268; full report is 5,268 generated and 356 unsupported with every failure
class zero; ten reports and nine disjoint scopes contain 224 unique outputs
within full; unsupported normalized tuple hash remains
`c44aaae7f939ebc07341cb984ca3f040512ec4dd5462d75454b178a713492030`;
all report/helper/launch provenance is current; repeated derivation is byte
idempotent. No order parity test is claimed before C05-C07.
Commands: last four exact list/run gates below; Final Artifact Gate;
`CARGO_NET_OFFLINE=true just verify`; `CARGO_NET_OFFLINE=true just
verify-generator`; `CARGO_NET_OFFLINE=true just corpus-check`;
`git diff --check`.
Intended commit: `test(parity): derive FRI-03 corpus baseline`.

## Exact Focused-Test Gates

```sh
bash -lc 'set -euo pipefail; test=generator::tests::bundled_helper_captures_exact_order_and_flex_parent_axes; CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- --list | rg -x "$test: test"; CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate "$test" -- --exact'
bash -lc 'set -euo pipefail; test=generator::tests::xml_generation_serializes_exact_order_and_parent_axes; CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- --list | rg -x "$test: test"; CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate "$test" -- --exact'
bash -lc 'set -euo pipefail; test=layout::browser_parity::support::tests::item_order_parser_is_canonical_and_scalar_independent; CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout -- --list | rg -x "$test: test"; CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout "$test" -- --exact'
bash -lc 'set -euo pipefail; test=layout::browser_parity::support::tests::viewport_parent_axes_schema_is_strict; CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout -- --list | rg -x "$test: test"; CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout "$test" -- --exact'
bash -lc 'set -euo pipefail; test=generator::tests::generation_report_manifest_requires_the_exact_fri_03_inventory; CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- --list | rg -x "$test: test"; CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate "$test" -- --exact'
bash -lc 'set -euo pipefail; test=layout::browser_parity::browser_parity_html_corpus_inventory_is_documented; CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout -- --list | rg -x "$test: test"; CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout "$test" -- --exact'
bash -lc 'set -euo pipefail; test=layout::browser_parity::browser_parity_generation_report_counts_full_scope; CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout -- --list | rg -x "$test: test"; CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout "$test" -- --exact'
bash -lc 'set -euo pipefail; test=layout::browser_parity::browser_parity_generation_report_inventory_matches_fri_03_scopes; CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout -- --list | rg -x "$test: test"; CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout "$test" -- --exact'
```

## ExistingPinned Artifact Gates

Use one cached executable only. Never run `generate`, `import-taffy`, or any
acquisition-capable command. Intermediate uses the five existing scopes; final
uses all nine. Final reruns full plus every scope and compares a deterministic
artifact-tree hash.

### Intermediate Artifact Gate
```sh
/bin/bash -lc 'set -euo pipefail; base=b2af2a464f4c8ad868e3b490ae16aabec2a30394; unset SURGEIST_BROWSER_PATH SURGEIST_BROWSER_CACHE SURGEIST_BROWSER_VERSION SURGEIST_LAYOUT_GENERATE_FILTER SURGEIST_LAYOUT_BROWSER_PARITY_ROOT; browser=$(find target/surgeist-browser -type f -path "*/mac_arm-149.0.7827.115/*/Contents/MacOS/Google Chrome for Testing" -perm -111 -print); test "$(printf "%s\n" "$browser" | sed "/^$/d" | wc -l | tr -d " ")" -eq 1; version=$("$browser" --version | sed -E "s/[[:space:]]+$//"); test "$version" = "Google Chrome for Testing 149.0.7827.115"; run_generation() { env -u SURGEIST_BROWSER_CACHE -u SURGEIST_BROWSER_VERSION -u SURGEIST_LAYOUT_BROWSER_PARITY_ROOT CARGO_NET_OFFLINE=true SURGEIST_BROWSER_PATH="$browser" SURGEIST_LAYOUT_GENERATE_FILTER="$1" cargo run --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- generate-existing; }; run_generation ""; filters=(block/block_axes flex/flex_axes grid/grid_axes grid-lanes/grid_lanes_axes subgrid/subgrid_axes); for filter in "${filters[@]}"; do run_generation "$filter"; done; dir=tests/layout/browser_parity/xml/generation-reports; expected=$(printf "%s\n" all.json block_block_axes.json flex_flex_axes.json grid_grid_axes.json grid-lanes_grid_lanes_axes.json subgrid_subgrid_axes.json | sort); actual=$(find "$dir" -maxdepth 1 -type f -name "*.json" -exec basename {} \; | sort); test "$expected" = "$actual"; full="$dir/all.json"; jq -e ".summary.generated == 5256 and .summary.unsupported == 356 and .summary.expected_fail == 0 and .summary.quarantined == 0 and .summary.failed_to_generate == 0" "$full" >/dev/null; test "$(jq -S ".unsupported | map({name, source, variant, reason}) | sort_by(.name, .source, .variant, .reason)" "$full" | shasum -a 256 | awk "{print \$1}")" = c44aaae7f939ebc07341cb984ca3f040512ec4dd5462d75454b178a713492030; helper=$(shasum -a 256 tests/layout/browser_parity/scripts/gentest/test_helper.js | awk "{print \$1}"); launch=9e2b5a4850e8d5ae31cf133c30f7129f1e214705f7a848697ca42c7c1b7551cb; for report in "$dir"/*.json; do jq -e --arg helper "$helper" --arg launch "$launch" ".metadata.helper_sha256 == \$helper and .metadata.launch_profile_sha256 == \$launch" "$report" >/dev/null; done; test "$(find tests/layout/browser_parity/xml -type f -name "*.xml" | wc -l | tr -d " ")" -eq 5256; ids=(grid_available_space_greater_than_max_content grid_available_space_smaller_than_max_content grid_available_space_smaller_than_min_content chrome_issue_325928327); variants=(border_box_ltr border_box_rtl content_box_ltr content_box_rtl); expected_parent=$(for id in "${ids[@]}"; do for variant in "${variants[@]}"; do printf "tests/layout/browser_parity/xml/grid/%s__%s.xml\n" "$id" "$variant"; done; done | sort); actual_parent=$(rg -l "parent-writing-mode=" tests/layout/browser_parity/xml --glob "*.xml" | sort); test "$expected_parent" = "$actual_parent"; while IFS= read -r file; do rg -q "parent-direction=" "$file"; rg -q "root-context=\"flex-item\"" "$file"; done <<<"$expected_parent"; tmp=$(mktemp -d); trap "rm -rf \"$tmp\"" EXIT; while IFS= read -r path; do mkdir -p "$tmp/old/$(dirname "$path")" "$tmp/new/$(dirname "$path")"; git show "$base:$path" | sed "1{/^<!-- generated-by:/d;}" >"$tmp/old/$path"; sed -E "1{/^<!-- generated-by:/d;}; s/ parent-writing-mode=\"[^\"]*\"//; s/ parent-direction=\"[^\"]*\"//" "$path" >"$tmp/new/$path"; diff -u "$tmp/old/$path" "$tmp/new/$path"; done < <(git ls-tree -r --name-only "$base" -- tests/layout/browser_parity/xml | rg "[.]xml$"); git diff --exit-code "$base" -- tests/layout/browser_parity/corpus.toml tests/layout/browser_parity/html tests/layout/browser_parity.rs tests/layout/browser_parity/support.rs; CARGO_NET_OFFLINE=true just corpus-check'
```

### Final Artifact Gate
```sh
/bin/bash -lc 'set -euo pipefail; base=b2af2a464f4c8ad868e3b490ae16aabec2a30394; unset SURGEIST_BROWSER_PATH SURGEIST_BROWSER_CACHE SURGEIST_BROWSER_VERSION SURGEIST_LAYOUT_GENERATE_FILTER SURGEIST_LAYOUT_BROWSER_PARITY_ROOT; browser=$(find target/surgeist-browser -type f -path "*/mac_arm-149.0.7827.115/*/Contents/MacOS/Google Chrome for Testing" -perm -111 -print); test "$(printf "%s\n" "$browser" | sed "/^$/d" | wc -l | tr -d " ")" -eq 1; version=$("$browser" --version | sed -E "s/[[:space:]]+$//"); test "$version" = "Google Chrome for Testing 149.0.7827.115"; run_generation() { env -u SURGEIST_BROWSER_CACHE -u SURGEIST_BROWSER_VERSION -u SURGEIST_LAYOUT_BROWSER_PARITY_ROOT CARGO_NET_OFFLINE=true SURGEIST_BROWSER_PATH="$browser" SURGEIST_LAYOUT_GENERATE_FILTER="$1" cargo run --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- generate-existing; }; filters=(block/block_axes flex/flex_axes grid/grid_axes grid-lanes/grid_lanes_axes subgrid/subgrid_axes block/block_align_baseline_child_margin_percent flex/fri03_order_modified_flex grid/fri03_order_modified_grid grid-lanes/fri03_order_modified_lanes); run_all() { run_generation ""; for filter in "${filters[@]}"; do run_generation "$filter"; done; }; artifact_hash() { find tests/layout/browser_parity/xml -type f \( -name "*.xml" -o -path "*/generation-reports/*.json" \) -print0 | sort -z | xargs -0 shasum -a 256 | shasum -a 256 | awk "{print \$1}"; }; run_all; before=$(artifact_hash); run_all; test "$before" = "$(artifact_hash)"; dir=tests/layout/browser_parity/xml/generation-reports; expected=$(printf "%s\n" all.json block_block_axes.json flex_flex_axes.json grid_grid_axes.json grid-lanes_grid_lanes_axes.json subgrid_subgrid_axes.json block_block_align_baseline_child_margin_percent.json flex_fri03_order_modified_flex.json grid_fri03_order_modified_grid.json grid-lanes_fri03_order_modified_lanes.json | sort); actual=$(find "$dir" -maxdepth 1 -type f -name "*.json" -exec basename {} \; | sort); test "$expected" = "$actual"; full="$dir/all.json"; jq -e ".summary.generated == 5268 and .summary.unsupported == 356 and .summary.expected_fail == 0 and .summary.quarantined == 0 and .summary.failed_to_generate == 0 and (.generated|length) == 5268 and (.unsupported|length) == 356" "$full" >/dev/null; tmp_outputs=$(mktemp); trap "rm -f \"$tmp_outputs\"" EXIT; scopes=(block_block_axes.json:20 flex_flex_axes.json:80 grid_grid_axes.json:36 grid-lanes_grid_lanes_axes.json:36 subgrid_subgrid_axes.json:36 block_block_align_baseline_child_margin_percent.json:4 flex_fri03_order_modified_flex.json:4 grid_fri03_order_modified_grid.json:4 grid-lanes_fri03_order_modified_lanes.json:4); for spec in "${scopes[@]}"; do file=${spec%%:*}; count=${spec##*:}; jq -e --argjson count "$count" ".summary.generated == \$count and .summary.unsupported == 0 and .summary.expected_fail == 0 and .summary.quarantined == 0 and .summary.failed_to_generate == 0 and (.generated|length) == \$count" "$dir/$file" >/dev/null; jq -r ".generated[].output" "$dir/$file" >>"$tmp_outputs"; done; test "$(wc -l <"$tmp_outputs" | tr -d " ")" -eq 224; test "$(sort -u "$tmp_outputs" | wc -l | tr -d " ")" -eq 224; test -z "$(comm -23 <(sort -u "$tmp_outputs") <(jq -r ".generated[].output" "$full" | sort -u))"; test "$(jq -S ".unsupported | map({name, source, variant, reason}) | sort_by(.name, .source, .variant, .reason)" "$full" | shasum -a 256 | awk "{print \$1}")" = c44aaae7f939ebc07341cb984ca3f040512ec4dd5462d75454b178a713492030; helper=$(shasum -a 256 tests/layout/browser_parity/scripts/gentest/test_helper.js | awk "{print \$1}"); launch=9e2b5a4850e8d5ae31cf133c30f7129f1e214705f7a848697ca42c7c1b7551cb; for report in "$dir"/*.json; do jq -e --arg helper "$helper" --arg launch "$launch" ".metadata.helper_sha256 == \$helper and .metadata.launch_profile_sha256 == \$launch" "$report" >/dev/null; done; html=tests/layout/browser_parity/html; test "$(find "$html" -type f -name "*.html" | wc -l | tr -d " ")" -eq 1406; test "$(find "$html" -type f -name "*.html" ! -path "*/grid-lanes/*" ! -path "*/subgrid/*" | wc -l | tr -d " ")" -eq 1161; test "$(find "$html/grid-lanes" -type f -name "*.html" | wc -l | tr -d " ")" -eq 26; test "$(find "$html/subgrid" -type f -name "*.html" | wc -l | tr -d " ")" -eq 219; test "$(find tests/layout/browser_parity/xml -type f -name "*.xml" | wc -l | tr -d " ")" -eq 5268; ids=(grid_available_space_greater_than_max_content grid_available_space_smaller_than_max_content grid_available_space_smaller_than_min_content chrome_issue_325928327); variants=(border_box_ltr border_box_rtl content_box_ltr content_box_rtl); expected_parent=$(for id in "${ids[@]}"; do for variant in "${variants[@]}"; do printf "tests/layout/browser_parity/xml/grid/%s__%s.xml\n" "$id" "$variant"; done; done | sort); test "$expected_parent" = "$(rg -l "parent-writing-mode=" tests/layout/browser_parity/xml --glob "*.xml" | sort)"; while IFS= read -r file; do rg -q "parent-direction=" "$file"; done <<<"$expected_parent"; sources=(flex/fri03_order_modified_flex grid/fri03_order_modified_grid grid-lanes/fri03_order_modified_lanes); expected_new=$(for source in "${sources[@]}"; do for variant in "${variants[@]}"; do printf "tests/layout/browser_parity/xml/%s__%s.xml\n" "$source" "$variant"; done; done | sort); current=$(find tests/layout/browser_parity/xml -type f -name "*.xml" | sort); previous=$(git ls-tree -r --name-only "$base" -- tests/layout/browser_parity/xml | rg "[.]xml$" | sort); test "$expected_new" = "$(comm -13 <(printf "%s\n" "$previous") <(printf "%s\n" "$current"))"; while IFS= read -r file; do test "$(rg -o " order=\"[^\"]+\"" "$file" | wc -l | tr -d " ")" -eq 3; done <<<"$expected_new"; tmp=$(mktemp -d); trap "rm -rf \"$tmp\" \"$tmp_outputs\"" EXIT; while IFS= read -r path; do mkdir -p "$tmp/old/$(dirname "$path")" "$tmp/new/$(dirname "$path")"; git show "$base:$path" | sed "1{/^<!-- generated-by:/d;}" >"$tmp/old/$path"; sed -E "1{/^<!-- generated-by:/d;}; s/ parent-writing-mode=\"[^\"]*\"//; s/ parent-direction=\"[^\"]*\"//" "$path" >"$tmp/new/$path"; diff -u "$tmp/old/$path" "$tmp/new/$path"; done < <(git ls-tree -r --name-only "$base" -- tests/layout/browser_parity/xml | rg "[.]xml$"); CARGO_NET_OFFLINE=true just corpus-check'
```

Worker evidence names every offline `generate-existing` invocation, unique browser
path, unset override variables, and manifest filter. The final audit proves report
and corpus counts, hashes, 16 parent-axis/12 new order XML, no junk, and base-body equality.

## Completion

```sh
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
git diff --check
bash -lc 'set -euo pipefail; tests=(generator::tests::bundled_helper_captures_exact_order_and_flex_parent_axes generator::tests::xml_generation_serializes_exact_order_and_parent_axes generator::tests::generation_report_manifest_requires_the_exact_fri_03_inventory); for test in "${tests[@]}"; do CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- --list | rg -x "$test: test"; CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate "$test" -- --exact; done; tests=(layout::browser_parity::support::tests::item_order_parser_is_canonical_and_scalar_independent layout::browser_parity::support::tests::viewport_parent_axes_schema_is_strict layout::browser_parity::browser_parity_html_corpus_inventory_is_documented layout::browser_parity::browser_parity_generation_report_counts_full_scope layout::browser_parity::browser_parity_generation_report_inventory_matches_fri_03_scopes); for test in "${tests[@]}"; do CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout -- --list | rg -x "$test: test"; CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout "$test" -- --exact; done'
git diff --exit-code b2af2a464f4c8ad868e3b490ae16aabec2a30394 -- src README.md Cargo.toml Cargo.lock Justfile scripts tests/bin/surgeist-layout-generate.rs tests/layout/browser_parity/README.md
/bin/bash -lc 'set -euo pipefail; files=(); while IFS= read -r -d "" file; do files+=("$file"); done < <(git ls-files -z --cached --others --exclude-standard -- "*.rs"); test "${#files[@]}" -gt 0; if rg -n --pcre2 '\''#\s*!?\s*\[[^]]*(?:unsafe\s*\(|\b(?:no_mangle|export_name|link_section|naked)\b|\b(?:allow|expect)\s*\([^]]*\b(?:unsafe_code|unsafe_op_in_unsafe_fn)\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{'\'' "${files[@]}"; then exit 1; else test "$?" -eq 1; fi'
```

Cycle acceptance: exact order and parent-axis fixture data is captured, emitted,
strictly parsed, and derivably current; all final inventory/provenance/count/hash
claims hold; default generator architecture and production layout remain
unchanged. C03 may now make parent axes mandatory at the public consumer boundary;
C05-C07 may later make the generated order expectations pass. No root handoff is
emitted from this cycle alone.

Genuine blockers: cached-browser/version drift, any fetch, tuple or body drift,
non-idempotence, algorithm/public-source change, hand-edited XML, or generator expansion.
Stop and replan; do not weaken, quarantine, or infer.
