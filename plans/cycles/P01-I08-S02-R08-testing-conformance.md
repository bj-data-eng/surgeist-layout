# P01/I08/S02/R08 Whole-Crate Testing-Reference Conformance

Cycle ID: `P01/I08/S02/R08`

Owning repository: `surgeist-layout`

Status: `draft`

Cycle base: `97a93e935a42a62d931192697abbafe382054806`

Specification: `plans/specs/P01-I08-grid-subgrid-and-grid-lanes-completeness.md`,
reviewed semantic SHA-256
`65050fe9723a62ef832badd02426c3fc2cb461f7931a4549a4c48c2ea39614e7`,
commit `98d67b05b7570e84490c6bf0121ba4a0cc2ec224`, sections `FRI-08.20`
row `AR-007`, `FRI-08.21`, `FRI-08.27.1`, and all of `FRI-08.28`.

Sequence: `plans/sequences/P01-I08-S02-architectural-remediation.md`,
reviewed SHA-256
`bb3642deea547129932693820df949b3db365ba4e4f134814ab9611dbb2aa171`,
commit `5ec0eae900daac01d56dba8ea919080ea13be26e`, entry
`P01/I08/S02/R08`.

Testing authority: installed `surgeist-agent/references/testing.md`.

Reviewed entry-disposition attachment:
`plans/cycles/P01-I08-S02-R08-test-disposition-ledger.md`, SHA-256
`f2f9092e9303a0e4319472ec09f33c1a42fb93b967b9eb3269f29a4d1a7483f7`.

## 1 Outcome And Boundaries

Every tracked Rust test asserts a product contract rather than source, symbol,
file placement, initiative census, workflow state, or current implementation
output. The exact prohibited entry tests have one disposition in the reviewed
ledger. Behavioral, compile-contract, oracle, declared-artifact consumer,
scalar, cache, transaction, browser-parity, and generator coverage remains.
Only the explicitly runnable complete browser-parity tier stays ignored.

No production behavior, public API, dependency, feature, MSRV, ordinary command,
fixture, or generated artifact changes. No source-parsing Rust test, lexical
audit script, standing Dylint selection, or permanent architecture rule is
added. R06A's useful node-projection audit already exists and remains opt-in and
unselected. A newly discovered semantic audit question requires a reviewed plan
revision; workers do not improvise a lint or script.

## 2 Entry And Final Inventories

R07's locked/offline entry inventory is authoritative:

| Target/prefix | Entry | Entry leaf digest | Final |
| --- | ---: | --- | ---: |
| library | 2,087 | `e0da19f5f8ff509122a3b1f846f0257cbd1e2c0ff850bf3cbbdeaa84f59eb87b` | 2,042 |
| `grid::tests::` | 1,017 | `7d183fadd0668543df877f58247158c02a36541b3c327230b92eced892541adf` | 1,011 |
| `root_tests::` | 235 | `2e616af2e7c7b7a0f480cba90b63b7807831bc72da4d42d5464a9e49e04610dc` | 233 |
| `flex_tests::` | 169 | `5a1cd29203c4ee0eea91169e18512ffdd79bea40fe6237120d261490f8a6c0d5` | 169 |
| `block_tests::` | 212 | `a73897032730d7b3b0d3c10746a17f16403baab9ffb1de4fedcdca01e2b3822f` | 212 |
| integration | 244 | recorded R07 list | 215 |
| default package incl. 72 doctests | 2,403 | recorded R07 list | 2,329 |
| generator binary | 386 (384 pass, 2 ignored) | recorded R07 list | 350 pass |

Entry ignored leaves are
`layout_oracle_grid_baseline_offset_matches_oracle` and
`runs_all_checked_in_browser_parity_xml`, digest
`235b22672841a0d7889b49ce4e7241b5abde0ba18311c8a23774d76597dfaf99`.
Final ignored state is only `runs_all_checked_in_browser_parity_xml`, with its
existing reason and browser-executing gate `CARGO_NET_OFFLINE=true just parity-all`
(recorded, not run here).

Survivors are classified by owning family: model/value contracts; public or
crate-boundary layout/cache/invalidation/transaction/measurement/rounding and
algorithm behavior; independently derived oracle comparisons; external compile
contracts/doctests; declared parser/serializer/manifest/schema/report/provenance/
corpus/import/locking formats; and actual browser-parity comparisons. Reviewers
inspect every survivor and the exact ledger; lexical scans are supplemental.

## 3 Ordered Tasks

### T01 `P01/I08/S02/R08/T01` Core Source Proxies

Paths: `src/lib_tests.rs`, `src/contract_tests.rs`, `src/cache_tests.rs`, and
the test module in `src/scroll/construction.rs`. Apply ledger T01: remove 37
library tests and dead lexical helpers. Existing public type/compile, cache, and
canonical-scroll behavior is the replacement authority. No production item or
visibility changes. Dependency: cycle base. Commit:
`test(conformance): remove source proxy contracts`.

Commands: T01 library entry-list command, then locked/offline library filters
`fri05_c01_ fri05_c02_ fri05_c03_ fri05_c04_ fri05_c05_ fri05_c07_ fri06_c01_ fri06_c02_ fri06_c05_ canonical_geometry_`, then full library/package and strict gates.

### T02 `P01/I08/S02/R08/T02` Internal Architecture And Oracle Proxies

Paths: `src/grid_tests/{lanes_subgrid,oracle_comparison}_tests.rs`,
`src/root_tests/transaction_cache_tests.rs`, and
`src/block_tests/inline_runs_tests.rs`. Apply ledger T02: remove seven library
tests; rename/re-oracle the line-break test exactly to
`block_line_break_metadata_positions_rtl_lines_and_preserves_two_line_extent`.
No production change. Dependency: T01. Commit:
`test(conformance): replace internal architecture proxies`.

Commands: T02 library entry-list command, then locked/offline library filters
`fri08_c06r_inherited_placement_ fri08_c02r_lanes_track_phase_ fri06_c04_float_ fri06_c03_lifecycle_ block_line_break_ grid::tests::oracle_comparison::`, then full library/package and strict gates.

### T03 `P01/I08/S02/R08/T03` Generator Source/Workflow Proxies

Path: generator test module only. Apply ledger T03: remove 13 generator-target
tests and dead source/census/entry helpers. Retain temporary-Git import behavior
and `track_definition_serializes_subgrid_line_names`. Dependency: T02. Commit:
`test(conformance): remove generator workflow proxies`.

Commands: T03 generator entry-list command, then locked/offline generator filters
`fri06_c08_new_ fri06_c08_existing_ fri06_c08_range_ink_ fri06_c08_recovery_ fri06_c08r_ track_definition_`, then the full generator binary target,
`just verify-generator`, and strict generator-feature Clippy.

### T04 `P01/I08/S02/R08/T04` Browser Source/Workflow Proxies

Paths: `tests/layout/browser_parity.rs` and support test module only. Apply ledger
T04: remove four integration tests and one support test compiled in both library
and integration; remove dead history/census helpers. Dependency: T03. Commit:
`test(conformance): remove browser workflow proxies`.

Commands: T04 integration and library entry-list commands, then locked/offline integration filters
`fri06_c08r_ fri06_c12_t07_ support::tests::`, matching library browser-control
filter `grid::tests::browser_controls::fri06_c12_t08_browser_front_door::tests::`,
then full integration/library/package and strict gates.

### T05 `P01/I08/S02/R08/T05` Behavior-Owned Parity Evidence

Path: `tests/layout/browser_parity.rs` only. Apply ledger T05: remove 24 raw
inventory tests/dead path helpers; retain every actual comparison and declared
artifact consumer. Rename/refactor, without count change:

- `fri08_c06_exact_seventy_two_owned_rows_match_production` to
  `manifest_active_outputs_match_layout`;
- the ordinary finite-collapse comparison to
  `normalized_flex_collapse_ordinary_variants_match_layout`; and
- the Chrome finite-collapse comparison to
  `normalized_flex_collapse_chrome_variants_match_reviewed_geometry`.

Dependency: T04. Commit: `test(conformance): make parity evidence behavior owned`.

Commands: T05 integration entry-list command, then locked/offline integration filters
`runs_fri_02_ fixture_against_surgeist_layout outputs_match adapter_template_areas normalized_flex_collapse_ manifest_active_outputs_`, full integration/package,
`just corpus-check`, and strict gates.

### T06 `P01/I08/S02/R08/T06` Declared Generator Artifacts

Path: generator test module only. Apply ledger T06: remove 23 generator-target
tests/dead lineage/inventory helpers. Rename/refactor
`centralized_provenance_accepts_current_exact_inventory` to
`centralized_provenance_accepts_manifest_described_inventory` using synthetic
manifest-described membership. Retain generic manifest/report/hash/referential
integrity, serialization, XML, check-corpus, import, locking, helper runtime,
parser, and serializer contracts. Dependency: T05. Commit:
`test(conformance): consolidate declared artifacts`.

Commands: T06 generator entry-list command, then locked/offline generator filters
`centralized_provenance_ corpus_manifest_ generation_report_ xml_generation_ import_taffy_ track_definition_`, full generator binary target,
`just verify-generator`, and strict generator-feature Clippy.

For each space-separated filter above, the worker runs exactly:

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout "$filter"
```

For generator filters add
`--features layout-golden-generate --bin surgeist-layout-generate` before the
filter; for integration filters add `--test layout`. Entry proof uses the real
target's `-- --list` and the ledger names, never Rust source parsing. Every
retained focused family must be nonzero before and after. A deletion task's RED
is the external conformance predicate finding the listed executed entry tests;
do not invent a behavioral failure.

The exact ordered entry-list commands are:

```sh
# T01 and T02
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib -- --list
# T03 and T06
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- --list
# T04
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --test layout -- --list
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib -- --list
# T05
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --test layout -- --list
```

After the listed focused commands, every task runs this exact ordered GREEN gate:

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --test layout
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout
CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
cargo fmt --check
git diff --check
```

T03 and T06 first run
`CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate`, then
`CARGO_NET_OFFLINE=true just verify-generator`, then exact generator strict
Clippy
`CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --features layout-golden-generate --all-targets -- -F unsafe-code -D warnings`,
before the common GREEN gate. T05 runs `CARGO_NET_OFFLINE=true just corpus-check`
before the common gate. This ordering is mandatory.

## 4 Scope And Preservation Gates

Each task records exact base/head and runs:

```sh
case "$task_id" in
T01) allowed='^(src/(lib_tests|contract_tests|cache_tests)\.rs|src/scroll/construction\.rs)$';;
T02) allowed='^(src/grid_tests/(lanes_subgrid|oracle_comparison)_tests\.rs|src/root_tests/transaction_cache_tests\.rs|src/block_tests/inline_runs_tests\.rs)$';;
T03|T06) allowed='^tests/bin/surgeist-layout-generate/generator\.rs$';;
T04) allowed='^tests/layout/browser_parity(\.rs|/support\.rs)$';;
T05) allowed='^tests/layout/browser_parity\.rs$';; *) exit 1;; esac
task_paths="$(git diff --name-only "$task_base..$task_head" | LC_ALL=C sort -u)"
test -n "$task_paths"
test -z "$(printf '%s\n' "$task_paths" | rg -v "$allowed")"
git diff --check "$task_base..$task_head"
base_suppressions="$(while IFS= read -r p; do git show "$task_base:$p" | perl -0777 -ne 'while (/^[ \t]*#\s*!?\s*\[\s*(?:allow|expect|cfg_attr)\b[^\]]*\]/gms) {$m=$&;$m=~s/\s+/ /g;print "$m\n"}'; done < <(git ls-tree -r --name-only "$task_base" | rg '\.rs$') | LC_ALL=C sort)"
current_suppressions="$({ git ls-files -z -- '*.rs'; git ls-files -z --others --exclude-standard -- '*.rs'; } | sort -zu | xargs -0 perl -0777 -ne 'while (/^[ \t]*#\s*!?\s*\[\s*(?:allow|expect|cfg_attr)\b[^\]]*\]/gms) {$m=$&;$m=~s/\s+/ /g;print "$m\n"}' | LC_ALL=C sort)"
test -z "$(comm -13 <(printf '%s\n' "$base_suppressions") <(printf '%s\n' "$current_suppressions"))"
```

Production-prefix hashes remain exact for construction
`a0ff41e7b5ec666961fedd90e3b66c158497c6c1206c2098a3e8437f97a5de23`,
generator
`cc8054c393b7ac8307c033b17bf4aea5eb6cca9a7a75200f58a6cce0de35b526`,
and support
`39403ce2bb7c1e20024d0a90a4e5f19c115a273c9eb7a5a3dbd3e6ded4331518`.
The exact gates are:

```sh
test "$(perl -0ne '($p)=split(/#\[cfg\(test\)\]\npub\(super\) mod fri05_c02_factory_tests \{/); print $p' src/scroll/construction.rs | shasum -a 256 | awk '{print $1}')" = a0ff41e7b5ec666961fedd90e3b66c158497c6c1206c2098a3e8437f97a5de23
test "$(perl -0ne '($p)=split(/#\[cfg\(test\)\]\nmod tests \{/); print $p' tests/bin/surgeist-layout-generate/generator.rs | shasum -a 256 | awk '{print $1}')" = cc8054c393b7ac8307c033b17bf4aea5eb6cca9a7a75200f58a6cce0de35b526
test "$(perl -0ne '($p)=split(/#\[cfg\(test\)\]\nmod tests \{/); print $p' tests/layout/browser_parity/support.rs | shasum -a 256 | awk '{print $1}')" = 39403ce2bb7c1e20024d0a90a4e5f19c115a273c9eb7a5a3dbd3e6ded4331518
```

Marker movement is a defect.

All tasks run affected full targets, default package, strict locked/offline
Clippy `--all-targets -- -F unsafe-code -D warnings`, format, diff, exact scope,
no-new-suppression, and complete owned-Rust unsafe gates. No worker runs a
browser, generation, import/acquisition, Dylint/catalog command, or `cargo clean`.

Cycle paths are limited to this plan, its ledger, and the exact 11 unique task
paths matched by the task allowlists above.
Cargo files, README, Justfile, `src/lib.rs`, all other production sources,
`tools/`, `scripts/`, corpus manifest, HTML/XML, helper, reports, and generated
artifacts match the cycle base.

## 5 Completion

After six independently CLEAN task reviews, set status `complete`, verify ledger
SHA, exact final counts and one ignored leaf, and run workflow-only conformance:

```sh
set -e
test -z "$(rg -n --glob '*.rs' 'include_str!\([^)]*\.rs|read_to_string\([^\n]*\.rs|file!\(\)' src tests || true)"
test -z "$(rg -n --glob '*.rs' 'plans/|CYCLE_BASE|byte_frozen|current_output|architecture_has|has_one_owner|worktree_is_clean|stale_artifact_inventory' src tests || true)"
test -z "$(rg -n --glob '*.rs' 'git show|entry-only|final-lineage evidence|reviewed .* census|published digest' src tests || true)"
test "$(rg -n --glob '*.rs' '#\[ignore\b' src tests | wc -l | tr -d ' ')" = 1
rg -n --glob '*.rs' '#\[ignore\b' src tests | rg 'runs_all_checked_in_browser_parity_xml|browser_parity.rs'
task_base=97a93e935a42a62d931192697abbafe382054806
base_suppressions="$(while IFS= read -r p; do git show "$task_base:$p" | perl -0777 -ne 'while (/^[ \t]*#\s*!?\s*\[\s*(?:allow|expect|cfg_attr)\b[^\]]*\]/gms) {$m=$&;$m=~s/\s+/ /g;print "$m\n"}'; done < <(git ls-tree -r --name-only "$task_base" | rg '\.rs$') | LC_ALL=C sort)"
current_suppressions="$({ git ls-files -z -- '*.rs'; git ls-files -z --others --exclude-standard -- '*.rs'; } | sort -zu | xargs -0 perl -0777 -ne 'while (/^[ \t]*#\s*!?\s*\[\s*(?:allow|expect|cfg_attr)\b[^\]]*\]/gms) {$m=$&;$m=~s/\s+/ /g;print "$m\n"}' | LC_ALL=C sort)"
test -z "$(comm -13 <(printf '%s\n' "$base_suppressions") <(printf '%s\n' "$current_suppressions"))"
```

Review adjudicates product-import Git operations and prose; scans are not the
oracle. No catalog command runs. Catalog isolation is proven by unchanged
`tools/` scope.

If R07 cleanup removed pinned Taffy, the coordinator may reuse the user's exact
preapproval for only the documented `import-taffy` command and prove clean HEAD
`d1ff7e339b9ee35b33858779f8d7653197e93d92` plus no repository delta.

Final commands:

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
cargo fmt --check
git diff --check
```

Also prove exact cycle scope, public compile/API compatibility, unchanged
dependencies/features/MSRV/locks/docs/catalog/artifacts, no new suppression, no
owned unsafe, frozen hashes `c6e6f1422e14a5e4aa474c143998063ce0de4d0a9123b69875b35a4ed009a8f6`,
`c684c7f167d95997a4a9f0250467bbaf72c1b73e69e0f707a2ef32f4d25f7f36`,
`c10dc550d260a239c8bf9dd553f5272ca3bcc2826099bc182f800986b8b94c0e`,
1,448 HTML, 5,776 comment-free XML, and clean Git.

After holistic CLEAN, rerun all completion evidence at the exact reviewed head.
Use the canonical publication gate: fetch authority `main`, require remote tip is
an ancestor, perform a fast-forward-only push with stale-tip protection (never a
force/history rewrite), and read back local/tracking/remote equality. Prove no
cycle-owned process, run repository-root `cargo clean`, prove both target paths
absent and Git clean, record final FRI-08 handoff and the still-paused reviewed
FRI-09 sequence, then stop. Do not begin FRI-09 or create a shared skill reference.
