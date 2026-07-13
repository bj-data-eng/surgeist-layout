# FRI-02-C03 Reproducible Browser Runtime
Status: in_progress
Cycle ID: `FRI-02-C03`
Owning repository: `surgeist-layout`
Cycle base: `d383f446b5bb64424806c89aa3cd296c3deb7658`
Reviewed specification:
`plans/specs/2026-07-12-surgeist-layout-fri-02-logical-geometry-writing-modes.md`
at `cc42db7e9bb5d895f10cb4d62e964bfc3cf4aef2b4e74a85e39d822a441e7d3f`,
commit `092f3b383ce87a9b72834ed444996861e3cfda2d`; generator/report
requirements in `FRI-02.13`, generator rows of `FRI-02.14`, browser/runtime/
stability/docs contracts in `FRI-02.16`, fixture evidence in `FRI-02.17`, and
verification in `FRI-02.18`.
Reviewed sequence:
`plans/sequences/2026-07-12-surgeist-layout-fri-02-logical-geometry-writing-modes.md`
at `4e11f24ba41bd6a98260155e0a5e6d2fd83eb5456a3a547f5ebeb4f3df8f5eb9`,
commit `2436454bc8aefffe7e3af55866ccdddbd6fce97a`, entry `C03`.
Bounded outcome: managed-pinned and existing-pinned generation validate one
manifest-owned browser pin/executable and use one manifest-owned launch profile,
while corpus checks and Taffy maintenance never enter browser state.

## Boundary

This cycle owns generator command dispatch, browser resolution/validation,
launch construction, schema-2 browser/launch/report metadata, temporary report
inventory, provenance, corpus freshness, generator-produced migration, and local
docs. It does not add axis fixtures, change layout/XML geometry, retune generation,
acquire browser/Taffy software, edit root/siblings, or implement the final report
state. `C04`-`C07` add five scoped entries; `C08` removes the temporary nine.
FRI-13 owns the pre-existing failing ignored full-corpus aggregate gate; C03 does
not claim or alter it.

At the base, omitting the command generates; arbitrary prefixes select reports; browser
cache/version env values override constants; explicit paths bypass containment,
executable, and runtime-version validation; primary/retry duplicate browser
builders; fixed profile values live in Rust; schema-1 `corpus.toml` owns neither
browser nor reports. The full report is `5048/356/0/0/0`. Schema 2 temporarily
owns exactly:

| Filter | File | Generated | Unsupported |
| --- | --- | ---: | ---: |
| full | `all.json` | 5048 | 356 |
| `block/block_br_vertical` | `block_block_br_vertical.json` | 16 | 0 |
| `block/block_calc_width_margin` | `block_block_calc_width_margin.json` | 4 | 0 |
| `block/block_margin_x_percentage_intrinsic_size_self_negative` | `block_block_margin_x_percentage_intrinsic_size_self_negative.json` | 4 | 0 |
| `block/block_margin_x_percentage_intrinsic_size_self_positive` | `block_block_margin_x_percentage_intrinsic_size_self_positive.json` | 4 | 0 |
| `flex/flex_calc_basis_margin_gap` | `flex_flex_calc_basis_margin_gap.json` | 4 | 0 |
| `grid/grid_calc_track_and_item_margin` | `grid_grid_calc_track_and_item_margin.json` | 4 | 0 |
| `grid/grid_max_content_single_item_margin_percent` | `grid_grid_max_content_single_item_margin_percent.json` | 4 | 0 |
| `grid/grid_min_content_flex_single_item_margin_percent` | `grid_grid_min_content_flex_single_item_margin_percent.json` | 4 | 0 |
| `grid/grid_named_template_area_generated_names` | `grid_grid_named_template_area_generated_names.json` | 4 | 0 |

Every non-generated scoped bucket and full expected-fail/quarantined/failed
bucket is zero. Duplicate, unlisted, missing, renamed, or extra reports fail.
The cached repo-relative pin `149.0.7827.115` exists and normalizes to
`Google Chrome for Testing 149.0.7827.115`; only `generate-existing` may execute
it. Managed behavior uses an injected fetch boundary; agents never acquire.

Generation retains exactly: batch 50, sorted sequential jobs, 10,000 ms timeout,
25 ms poll, one open/load/reset-timeout retry, per-batch/retry profiles, one page
per job, disabled defaults/cache, and the ordered 28 manifest arguments including
`use-mock-keychain`. Pre-migration hashes are XML path plus bytes after its first
line `1f79b729937f0e239619ff8e18e6fab080b8573bcfacf04e67f6ad195f39486b`;
sorted unsupported `{name,source,variant,reason}` JSON
`c44aaae7f939ebc07341cb984ca3f040512ec4dd5462d75454b178a713492030`;
sorted full-report output paths
`8cd0d239a9a2a9196c185abbf4e29c45b08490ea05d68c1efe93e54ea4bf8868`;
all ten `{file,filter,generated[{name,source,output,variant}]}` records serialized
as filename-sorted compact sorted-key JSON lines
`c7ca81f7052b0d8046a7700a726d0177b50e2a87432fc6c6c4ae5d52c770a520`.

## Impacts

Public API: none. Generator CLI is intentionally breaking pre-release:
`generate`, `generate-existing`, `check-corpus`, `check-taffy-corpus`, and
`import-taffy` only; no command errors; legacy cache/version overrides are
generation ambiguity errors; filters are full/empty or an exact scoped entry.
Dependencies/features/MSRV: unchanged, Rust 1.97. Artifacts: `corpus.toml`, 5,048
XML provenance lines, and ten reports advance atomically through the generator;
geometry, paths, classifications, and sources do not change. Docs: `README.md`,
crate rustdoc, and browser-parity README cover both browser modes, version checks,
shared mock-keychain launch, and browser-free checks. Root: none. Unsafe: none.

## Tasks

### C03-T1 - Atomic Pinned Runtime And Schema-Two Corpus

Files/area: `tests/bin/surgeist-layout-generate/generator.rs`, `corpus.toml`,
`README.md`, `src/lib.rs`, browser-parity README, all generator-owned XML/reports.

Outcome: strict schema-2 values own pin/cache/provenance, launch profile/digest,
full report, and the table above. A closed command request parses args/captured env
before access. `BrowserResolutionMode`/`PinnedBrowser` implement exact managed and
existing contracts; one validator enforces relative syntax, canonical cache
containment, regular executable status, normalized `--version`, and stable
repo-relative provenance before any write. Managed tests inject fetch and never
acquire. One `browser_launch_config` owner serves primary/retry. Non-browser
commands do not read browser/filter state. Old override authority, default command,
arbitrary report naming, hardcoded profile owners, duplicate builders, schema-1
reader, and compatibility aliases are deleted.

Report/provenance metadata carries schema 2 and the launch digest. `check-corpus`
derives the exact table, validates filters/counts/zero buckets, metadata, XML
provenance, scoped/full relations and extra/missing files, and is browser-free.
Full generation prunes only non-manifest reports; scoped generation touches only
its entry. Schema activation, full plus nine scoped existing-pinned generation,
all code/docs, and all generated artifacts form this one task and commit.

RED evidence: focused strict-manifest, dispatch, filter, resolution, containment,
version, launch, stability, report, provenance, browser-free, and idempotence tests
fail against schema 1, unchecked/override paths, duplicate builders, and arbitrary
report derivation.

Acceptance: every specified invalid command/env/path/version/fetch/report case
fails before access or writes; both browser modes yield identical typed provenance;
managed tests prove requested pin and zero real fetch; all launch/lifecycle values
are exact; failure accounting has no silent path. Real existing-pinned full+nine
generation has zero failures; all 5,048 XML and ten reports carry current schema,
digest, and provenance; table counts and all four hashes remain exact; no artifact
is added/removed; one scoped rerun is byte-idempotent; poisoned `check-corpus` and
all C03-owned focused/default tests pass; docs contain no legacy guidance.

Commands: focused filters `browser_command_dispatch`, `browser_resolution`,
`browser_launch_profile`, `generator_stability`, `corpus_manifest_schema`,
`generation_report_manifest`, `browser_provenance`, and `check_corpus`; then the
exact generation/evidence block below and the final package gates. Intended
commit: `generator: pin reproducible browser runtime`.

Depends on: published C02 at the cycle base; no intra-cycle predecessor.

## Completion

Before generation, run this one no-fetch shell from the repository root:
```sh
/bin/bash -lc 'set -euo pipefail
unset SURGEIST_BROWSER_PATH SURGEIST_BROWSER_CACHE SURGEIST_BROWSER_VERSION SURGEIST_LAYOUT_GENERATE_FILTER SURGEIST_LAYOUT_BROWSER_PARITY_ROOT
matches=$(find target/surgeist-browser -type f -path "*/mac_arm-149.0.7827.115/*/Contents/MacOS/Google Chrome for Testing" -perm -111 -print); test "$(printf "%s\n" "$matches" | sed "/^$/d" | wc -l | tr -d " ")" -eq 1; export SURGEIST_BROWSER_PATH="$matches"; test -x "$SURGEIST_BROWSER_PATH"; test "$("$SURGEIST_BROWSER_PATH" --version | awk "{\$1=\$1; print}")" = "Google Chrome for Testing 149.0.7827.115"
run_generation() { env -u SURGEIST_BROWSER_CACHE -u SURGEIST_BROWSER_VERSION -u SURGEIST_LAYOUT_BROWSER_PARITY_ROOT CARGO_NET_OFFLINE=true SURGEIST_BROWSER_PATH="$SURGEIST_BROWSER_PATH" SURGEIST_LAYOUT_GENERATE_FILTER="$1" cargo run --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- generate-existing; }
run_generation ""; for filter in block/block_br_vertical block/block_calc_width_margin block/block_margin_x_percentage_intrinsic_size_self_negative block/block_margin_x_percentage_intrinsic_size_self_positive flex/flex_calc_basis_margin_gap grid/grid_calc_track_and_item_margin grid/grid_max_content_single_item_margin_percent grid/grid_min_content_flex_single_item_margin_percent grid/grid_named_template_area_generated_names; do run_generation "$filter"; done
xml_hash=$(while IFS= read -r -d "" file; do printf "%s\0" "$file"; tail -n +2 "$file"; done < <(find tests/layout/browser_parity/xml -type f -name "*.xml" -print0 | sort -z) | shasum -a 256 | awk "{print \$1}"); test "$xml_hash" = 1f79b729937f0e239619ff8e18e6fab080b8573bcfacf04e67f6ad195f39486b
unsupported_hash=$(jq -S ".unsupported | map({name, source, variant, reason}) | sort_by(.name, .source, .variant, .reason)" tests/layout/browser_parity/xml/generation-reports/all.json | shasum -a 256 | awk "{print \$1}"); test "$unsupported_hash" = c44aaae7f939ebc07341cb984ca3f040512ec4dd5462d75454b178a713492030
paths_hash=$(jq -r ".generated[].output" tests/layout/browser_parity/xml/generation-reports/all.json | sort | shasum -a 256 | awk "{print \$1}"); test "$paths_hash" = 8cd0d239a9a2a9196c185abbf4e29c45b08490ea05d68c1efe93e54ea4bf8868
records_hash=$(for file in tests/layout/browser_parity/xml/generation-reports/*.json; do name=$(basename "$file"); jq -cS --arg file "$name" "{file:\$file, filter:.filter, generated:(.generated | map({name, source, output, variant}) | sort_by(.name, .source, .output, .variant))}" "$file"; done | shasum -a 256 | awk "{print \$1}"); test "$records_hash" = c7ca81f7052b0d8046a7700a726d0177b50e2a87432fc6c6c4ae5d52c770a520
artifact_hash() { while IFS= read -r -d "" file; do printf "%s\0" "$file"; shasum -a 256 "$file"; done < <(find tests/layout/browser_parity/xml -type f \( -name "*.xml" -o -path "*/generation-reports/*.json" \) -print0 | sort -z) | shasum -a 256 | awk "{print \$1}"; }; before=$(artifact_hash); run_generation block/block_calc_width_margin; test "$before" = "$(artifact_hash)"
env -u SURGEIST_LAYOUT_BROWSER_PARITY_ROOT SURGEIST_BROWSER_PATH=/not/consulted SURGEIST_BROWSER_CACHE=/not/consulted SURGEIST_BROWSER_VERSION=wrong SURGEIST_LAYOUT_GENERATE_FILTER=wrong CARGO_NET_OFFLINE=true cargo run --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- check-corpus'
```

Final commands rerun the exact no-fetch shell above unchanged, then:
```sh
for filter in browser_command_dispatch browser_resolution browser_launch_profile generator_stability corpus_manifest_schema generation_report_manifest browser_provenance check_corpus; do env -u SURGEIST_BROWSER_PATH -u SURGEIST_BROWSER_CACHE -u SURGEIST_BROWSER_VERSION -u SURGEIST_LAYOUT_GENERATE_FILTER -u SURGEIST_LAYOUT_BROWSER_PARITY_ROOT CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate "$filter" -- --nocapture; done
env -u SURGEIST_BROWSER_PATH -u SURGEIST_BROWSER_CACHE -u SURGEIST_BROWSER_VERSION -u SURGEIST_LAYOUT_GENERATE_FILTER -u SURGEIST_LAYOUT_BROWSER_PARITY_ROOT CARGO_NET_OFFLINE=true cargo check --locked -p surgeist-layout --all-targets
env -u SURGEIST_BROWSER_PATH -u SURGEIST_BROWSER_CACHE -u SURGEIST_BROWSER_VERSION -u SURGEIST_LAYOUT_GENERATE_FILTER -u SURGEIST_LAYOUT_BROWSER_PARITY_ROOT CARGO_NET_OFFLINE=true cargo check --locked -p surgeist-layout --all-targets --features layout-golden-generate
env -u SURGEIST_BROWSER_PATH -u SURGEIST_BROWSER_CACHE -u SURGEIST_BROWSER_VERSION -u SURGEIST_LAYOUT_GENERATE_FILTER -u SURGEIST_LAYOUT_BROWSER_PARITY_ROOT CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout
env -u SURGEIST_BROWSER_PATH -u SURGEIST_BROWSER_CACHE -u SURGEIST_BROWSER_VERSION -u SURGEIST_LAYOUT_GENERATE_FILTER -u SURGEIST_LAYOUT_BROWSER_PARITY_ROOT CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate
env -u SURGEIST_BROWSER_PATH -u SURGEIST_BROWSER_CACHE -u SURGEIST_BROWSER_VERSION -u SURGEIST_LAYOUT_GENERATE_FILTER -u SURGEIST_LAYOUT_BROWSER_PARITY_ROOT CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --doc
env -u SURGEIST_BROWSER_PATH -u SURGEIST_BROWSER_CACHE -u SURGEIST_BROWSER_VERSION -u SURGEIST_LAYOUT_GENERATE_FILTER -u SURGEIST_LAYOUT_BROWSER_PARITY_ROOT RUSTDOCFLAGS="-D warnings" CARGO_NET_OFFLINE=true cargo doc --locked -p surgeist-layout --no-deps
env -u SURGEIST_BROWSER_PATH -u SURGEIST_BROWSER_CACHE -u SURGEIST_BROWSER_VERSION -u SURGEIST_LAYOUT_GENERATE_FILTER -u SURGEIST_LAYOUT_BROWSER_PARITY_ROOT CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
env -u SURGEIST_BROWSER_PATH -u SURGEIST_BROWSER_CACHE -u SURGEIST_BROWSER_VERSION -u SURGEIST_LAYOUT_GENERATE_FILTER -u SURGEIST_LAYOUT_BROWSER_PARITY_ROOT CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets --features layout-golden-generate -- -F unsafe-code -D warnings
cargo fmt --check && git diff --check && git diff --cached --check && git diff --exit-code && git diff --cached --exit-code
bash -lc 'set -euo pipefail; file=tests/bin/surgeist-layout-generate/generator.rs; test "$(rg -n "BrowserConfig::builder" "$file" | wc -l | tr -d " ")" -eq 1; test "$(rg -n -o "browser_launch_config\\(" "$file" | wc -l | tr -d " ")" -eq 3; if rg -n "BROWSER_FIXTURE_BATCH_SIZE|BROWSER_NAVIGATION_TIMEOUT|BROWSER_NAVIGATION_POLL_INTERVAL|fn browser_args" "$file"; then exit 1; else rc=$?; test "$rc" -eq 1; fi'
bash -lc 'set -euo pipefail; files=(); while IFS= read -r -d "" file; do files+=("$file"); done < <(git ls-files -z --cached --others --exclude-standard -- "*.rs"); test "${#files[@]}" -gt 0; if rg -n --pcre2 '\''#\s*!?\s*\[[^]]*(?:unsafe\s*\(|\b(?:no_mangle|export_name|link_section|naked)\b|\b(?:allow|expect)\s*\([^]]*\b(?:unsafe_code|unsafe_op_in_unsafe_fn)\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{'\'' "${files[@]}"; then exit 1; else rc=$?; test "$rc" -eq 1; fi'
test -z "$(git status --porcelain)" && git status --short --branch
```
The three launch-config occurrences are one declaration plus primary and retry
callers; the sole builder is inside that declaration.

Required handoff: `C04` receives this runtime/inventory, adds only
`block/block_axes`, and refreshes the full report. No root handoff before `C08`.
Genuine blockers: missing/mismatched cached pin, acquisition need, non-idempotent
output, hash/geometry/classification drift, browser-state leakage, or profile
retuning stops the cycle.
