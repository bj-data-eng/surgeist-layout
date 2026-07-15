# FRI-03-C02 Bounded Fixture Schema And Corpus Baseline
Status: in_progress
Cycle ID: `FRI-03-C02`
Owning repository: `surgeist-layout`
Cycle base: `b2af2a464f4c8ad868e3b490ae16aabec2a30394`
Reviewed specification:
`plans/specs/2026-07-15-surgeist-layout-fri-03-box-participation-contracts.md`
at `6ca195b4ba560ae49bc6963176234f8494cfb50a91674f6dcec358d19fa9769c`,
commit `52d87a75751f9987251ec2fdf8200e75eba3e17b`, sections
`FRI-03.2`, `E-PARITY`, `FRI-03.8`, `FRI-03.9`, `FRI-03.11`, and
artifact portions of acceptance items 7 and 8.
Reviewed sequence:
`plans/sequences/2026-07-15-surgeist-layout-fri-03-box-participation-contracts.md`
at `d59317e1b80337ff4041a034c062867dc7e744048eb7047d2b2e7b412aea130a`,
commit `03e7582565fa2d4f3aa7f71973f6dfebe273c4fb`, entry `C02`.
Bounded outcome: exact item order and flex-parent axes flow through the existing
producer/parser, optional filtered generation is report-free diagnosis, and one
final full ExistingPinned run establishes the three-source corpus and sole
`all.json` report before public context signatures change.

## Boundary
This cycle owns the embedded helper, existing serializer/parser/tests, the
bounded filter/report bug fix, three named HTML sources, `corpus.toml`, the
browser-parity operational README, inventory tests, and mechanically generated
XML/report state. It uses only the already-cached Chrome `149.0.7827.115` with
offline Cargo. No new command, module, script, dependency, feature, schema
version, report kind, browser policy, import path, or acquisition is allowed.
Production `src/`, public API, algorithms, caches, root/siblings, crate README,
manifest, lockfile, Justfile, and task scripts do not change. Replaced fixture
modeling, tag inference, natural dimensions, object sizing, and hand-edited XML
remain out of scope. Optional filtered runs may aid iteration but are never
required or retained evidence. After inputs settle, exactly one full run is
allowed; all following checks are read-only.
Current evidence: T1 is committed at
`7226cc4e1b011acaa75aa7ea9cb40d953188ed9a`; its 5,256 XML and six reports are
clean and byte-stable. Do not regenerate T1. T4 replaces that interim report
inventory under the final one-run rule.

## Impacts
Public API, production behavior, dependencies, features, lockfile, MSRV, crate
docs, and examples: unchanged. Test tooling receives one confirmed generator
bug fix. Final artifacts add three HTML/12 XML, update 16 existing XML bodies and
provenance, retain only `all.json`, and prune five scoped reports. Root: C08.
Unsafe: none.

## Tasks

### C02-T1 - Exact Producer And Serializer Metadata
Files: helper, `generator.rs`, existing 5,256 XML, and existing six reports.
Outcome: computed order is captured exactly; flex-item roots capture actual
parent writing mode/direction; XML omits zero order, emits nonzero signed values,
adds both parent-axis attributes to 16 flex-item roots, and omits them on roots.
RED: the two T1 generator tests below failed before the helper/serializer change;
stale-helper corpus validation then failed until the interim derivation.
Acceptance: zero/min/max, parent/root disagreement, root omission, exact 16-file
body delta, unchanged unsupported tuples, and no generator expansion are proven.
Commands: the two T1 focused gates, `CARGO_NET_OFFLINE=true just fmt-check`,
`CARGO_NET_OFFLINE=true just verify-generator`, `CARGO_NET_OFFLINE=true just
corpus-check`, and `git diff --check`. No further T1 derivation.
Commit: `test(generator): capture FRI-03 fixture metadata`.

### C02-T2 - Strict Scalar-Independent Fixture Parsing
Files: `tests/layout/browser_parity/support.rs` only.
Depends on: T1 task-review CLEAN.
Outcome: omitted order is `ItemOrder::ZERO`; canonical signed base-10 `i32`
parses directly to `ItemOrder`. Flex-item viewports require both strict parent
axis tokens and retain one `FlowAxes`; root viewports reject them. The current
one-argument public root context remains until C03.
RED: the two T2 layout tests below fail because order and parent axes are ignored.
Acceptance: min/zero/max pass; plus, leading-zero, negative-zero, fractional,
exponent, text, whitespace, and overflow forms fail; all checked-in XML parses;
no fallback, scalar conversion, panic, production source, or artifact changes.
Commands: the two T2 focused gates, `CARGO_NET_OFFLINE=true just verify`,
`CARGO_NET_OFFLINE=true just verify-generator`,
`CARGO_NET_OFFLINE=true just corpus-check`, and `git diff --check`.
Commit: `test(parity): parse FRI-03 fixture metadata`.

### C02-T3 - Report-Free Diagnostic Filters
Files: `tests/bin/surgeist-layout-generate/generator.rs` only.
Depends on: T2 task-review CLEAN.
Outcome: a normalized matched filter may drive optional ExistingPinned diagnosis
without a manifest report; it writes matching XML but no report and performs no
report pruning. Invalid or unmatched filters fail before artifact writes. An
unfiltered run remains the sole report writer and report/XML pruner.
RED: the two T3 generator tests below fail because filters require manifest
reports and select a persisted report path.
Acceptance: valid path/prefix matching, invalid-input no-write behavior, and
full-only report ownership are covered without a browser run. Existing launch,
locking, transactional XML, retry, and full-generation behavior stay unchanged.
No new command, output kind, module, schema, dependency, or acquisition path.
Commands: the two T3 focused gates, `CARGO_NET_OFFLINE=true just fmt-check`,
`CARGO_NET_OFFLINE=true just verify-generator`, `CARGO_NET_OFFLINE=true just
corpus-check`, and `git diff --check`.
Commit: `fix(generator): decouple diagnostic filters from reports`.

### C02-T4 - Final Corpus And Full Report
Files: three named HTML sources, `corpus.toml`, browser-parity README,
inventory/report tests, 12 new XML, refreshed existing XML, `all.json`, and
removal of five scoped reports.
Depends on: T3 task-review CLEAN.
Outcome: add `flex/fri03_order_modified_flex`,
`grid/fri03_order_modified_grid`, and
`grid-lanes/fri03_order_modified_lanes`, each with four visible fixed-size
children whose source-order values are `2, -1, 2, 0`. The final manifest has an
empty scoped-report inventory. The README distinguishes optional diagnostics
from final evidence. After source/test GREEN, run the Final Derivation exactly
once; it writes all 5,268 XML and `all.json` and prunes scoped reports.
RED: the four T4 focused tests below fail at 1,403 HTML, 5,256 XML, six reports,
and missing source/output paths.
Acceptance: 1,406 HTML (1,161 ordinary, 26 grid-lanes, 219 subgrid), 5,268 XML,
one full 5,268/356 report, zero failure classes, unchanged unsupported tuple
hash, current provenance, 16 parent-axis XML, 12 new order XML, and no scoped
report remain. No second full run; all post-derivation checks are read-only.
Commands: the four T4 focused gates, Final Derivation, Read-Only Audit,
`CARGO_NET_OFFLINE=true just verify`,
`CARGO_NET_OFFLINE=true just verify-generator`,
`CARGO_NET_OFFLINE=true just corpus-check`, and `git diff --check`.
Commit: `test(parity): derive FRI-03 corpus baseline`.

## Focused Test Gates
For each name, first prove one exact listing, then run that exact test.

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- --list | rg -x 'generator::tests::bundled_helper_captures_exact_order_and_flex_parent_axes: test'
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate generator::tests::bundled_helper_captures_exact_order_and_flex_parent_axes -- --exact
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- --list | rg -x 'generator::tests::xml_generation_serializes_exact_order_and_parent_axes: test'
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate generator::tests::xml_generation_serializes_exact_order_and_parent_axes -- --exact
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout -- --list | rg -x 'layout::browser_parity::support::tests::item_order_parser_is_canonical_and_scalar_independent: test'
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout layout::browser_parity::support::tests::item_order_parser_is_canonical_and_scalar_independent -- --exact
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout -- --list | rg -x 'layout::browser_parity::support::tests::viewport_parent_axes_schema_is_strict: test'
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout layout::browser_parity::support::tests::viewport_parent_axes_schema_is_strict -- --exact
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- --list | rg -x 'generator::tests::diagnostic_filter_is_report_free_and_manifest_independent: test'
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate generator::tests::diagnostic_filter_is_report_free_and_manifest_independent -- --exact
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- --list | rg -x 'generator::tests::diagnostic_filter_rejects_invalid_or_unmatched_input_before_writes: test'
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate generator::tests::diagnostic_filter_rejects_invalid_or_unmatched_input_before_writes -- --exact
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- --list | rg -x 'generator::tests::generation_report_manifest_requires_full_only_fri_03_inventory: test'
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate generator::tests::generation_report_manifest_requires_full_only_fri_03_inventory -- --exact
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout -- --list | rg -x 'layout::browser_parity::browser_parity_html_corpus_inventory_is_documented: test'
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout layout::browser_parity::browser_parity_html_corpus_inventory_is_documented -- --exact
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout -- --list | rg -x 'layout::browser_parity::browser_parity_generation_report_counts_full_scope: test'
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout layout::browser_parity::browser_parity_generation_report_counts_full_scope -- --exact
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout -- --list | rg -x 'layout::browser_parity::browser_parity_generation_report_inventory_is_full_only: test'
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout layout::browser_parity::browser_parity_generation_report_inventory_is_full_only -- --exact
```

## Final Derivation
Optional scoped diagnostics may occur before this gate and are not evidence.
After T4 source/tests settle, execute this block once and only once.

```sh
/bin/bash <<'BASH'
set -euo pipefail
unset SURGEIST_BROWSER_PATH SURGEIST_BROWSER_CACHE SURGEIST_BROWSER_VERSION
unset SURGEIST_LAYOUT_GENERATE_FILTER SURGEIST_LAYOUT_BROWSER_PARITY_ROOT
browser=$(find target/surgeist-browser -type f -path '*/mac_arm-149.0.7827.115/*/Contents/MacOS/Google Chrome for Testing' -perm -111 -print)
test "$(printf '%s\n' "$browser" | sed '/^$/d' | wc -l | tr -d ' ')" -eq 1
test "$("$browser" --version)" = 'Google Chrome for Testing 149.0.7827.115'
env -u SURGEIST_BROWSER_CACHE -u SURGEIST_BROWSER_VERSION -u SURGEIST_LAYOUT_GENERATE_FILTER -u SURGEIST_LAYOUT_BROWSER_PARITY_ROOT \
  CARGO_NET_OFFLINE=true SURGEIST_BROWSER_PATH="$browser" \
  cargo run --locked --offline -p surgeist-layout --features layout-golden-generate \
  --bin surgeist-layout-generate -- generate-existing
BASH
```

## Read-Only Audit

```sh
test "$(find tests/layout/browser_parity/html -type f -name '*.html' | wc -l | tr -d ' ')" -eq 1406
test "$(find tests/layout/browser_parity/xml -type f -name '*.xml' | wc -l | tr -d ' ')" -eq 5268
test "$(find tests/layout/browser_parity/xml/generation-reports -type f -name '*.json' | wc -l | tr -d ' ')" -eq 1
test -f tests/layout/browser_parity/xml/generation-reports/all.json
jq -e '.summary.generated == 5268 and .summary.unsupported == 356 and .summary.expected_fail == 0 and .summary.quarantined == 0 and .summary.failed_to_generate == 0' tests/layout/browser_parity/xml/generation-reports/all.json >/dev/null
test "$(jq -S '.unsupported | map({name, source, variant, reason}) | sort_by(.name, .source, .variant, .reason)' tests/layout/browser_parity/xml/generation-reports/all.json | shasum -a 256 | awk '{print $1}')" = c44aaae7f939ebc07341cb984ca3f040512ec4dd5462d75454b178a713492030
test "$(rg -l 'parent-writing-mode=' tests/layout/browser_parity/xml --glob '*.xml' | wc -l | tr -d ' ')" -eq 16
test "$(rg -o ' order="[^"]+"' tests/layout/browser_parity/xml --glob '*.xml' | wc -l | tr -d ' ')" -eq 36
```

## Completion

```sh
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
/bin/bash -lc 'set -euo pipefail; files=(); while IFS= read -r -d "" file; do files+=("$file"); done < <(git ls-files -z --cached --others --exclude-standard -- "*.rs"); test "${#files[@]}" -gt 0; if rg -n --pcre2 '\''#\s*!?\s*\[[^]]*(?:unsafe\s*\(|\b(?:no_mangle|export_name|link_section|naked)\b|\b(?:allow|expect)\s*\([^]]*\b(?:unsafe_code|unsafe_op_in_unsafe_fn)\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{'\'' "${files[@]}"; then exit 1; else test "$?" -eq 1; fi'
git diff --check
git diff --exit-code b2af2a464f4c8ad868e3b490ae16aabec2a30394 -- src README.md Cargo.toml Cargo.lock Justfile scripts tests/bin/surgeist-layout-generate.rs
test -z "$(git status --porcelain)"
```
Cycle acceptance: exact metadata is captured and strictly parsed; diagnostics
are report-free; the three fixtures and sole full report are current after one
final full derivation; all focused, `just`, provenance, count, hash, protected
scope, and unsafe checks pass. C03 may consume parent axes; C05-C07 may consume
order. No root handoff is emitted from C02 alone.
After task reviews, commit status `complete`, run final checks and holistic review, rerun checks, publish local `main` with a lease, and read back the exact remote SHA.
The C02 leaf handoff records evidence and authorizes the JIT C03 plan; root waits for C08.
Genuine blockers: missing exact cached browser, any acquisition, full-run drift,
unexpected report/XML inventory, production-source change, hand-edited XML, or
generator work beyond the confirmed diagnostic-filter/report fix. Stop and
replan; never repeat the final full run to chase an unexplained failure.
