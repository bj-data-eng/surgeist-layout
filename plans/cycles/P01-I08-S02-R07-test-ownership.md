# P01/I08/S02/R07 Test Ownership

Cycle ID: `P01/I08/S02/R07`

Owning repository: `surgeist-layout`

Status: `in_progress`

Cycle base: `496baae07f7a1216cab51267848231da82970941`

Specification: `plans/specs/P01-I08-grid-subgrid-and-grid-lanes-completeness.md`,
reviewed semantic SHA-256
`65050fe9723a62ef832badd02426c3fc2cb461f7931a4549a4c48c2ea39614e7`,
commit `98d67b05b7570e84490c6bf0121ba4a0cc2ec224`, sections `FRI-08.20`
row `AR-006`, `FRI-08.21`, `FRI-08.26`, and the partition-preservation
and ordinary-verification portions of `FRI-08.27`.

Sequence: `plans/sequences/P01-I08-S02-architectural-remediation.md`,
reviewed semantic SHA-256
`bb3642deea547129932693820df949b3db365ba4e4f134814ab9611dbb2aa171`,
commit `5ec0eae900daac01d56dba8ea919080ea13be26e`, entry
`P01/I08/S02/R07`.

Bounded outcome: the four large companion suites become private test-module
directories partitioned by the production responsibilities they verify, with
one explicit test-only fixture owner per suite and no test, assertion, ignored
state, public API, production behavior, or artifact drift.

## 1 Boundary And Baseline

R01 through R06A are published and remotely read back. The immutable entry is
clean `main` at the cycle base above, with no repository `target/`. Production
owners, the compatible public facade, and the opt-in Dylint catalog are stable.

This cycle changes only test organization. It does not change production
algorithms, public names or paths, errors, caches, dependencies, features,
MSRV, README, fixtures, generated XML, provenance, the generator, ordinary
commands, CI, or the Dylint catalog. Browser execution, generation, and new
artifact writes remain prohibited. Generator architecture expansion and FRI-09
implementation remain excluded.

Every production source, including `src/grid/mod.rs` and `src/lib.rs`, remains
byte-identical to the cycle base. Each existing `*_tests.rs` file becomes a slim
test-only composition facade and retains its existing loader path. Every child
file also ends in `_tests.rs`, preserving the legacy production-source
classifier unchanged until R08 removes that proxy. No production visibility or
test hook is added.

The existing source-text proxy tests are temporary R08 debt. R07 neither adds
one nor changes their assertion semantics. A moved proxy may receive only the
relative `include_str!` path correction required by its new directory depth;
R08 owns its final removal or replacement. Test partition evidence comes from
Rust test-harness discovery, focused execution, exact diff review, and full
behavior checks rather than a Rust test that parses source.

At the cycle base, locked/offline Rust test discovery reports:

| Inventory | Count | Sorted leaf-name multiset SHA-256 |
| --- | ---: | --- |
| library target | 2,087 | `e0da19f5f8ff509122a3b1f846f0257cbd1e2c0ff850bf3cbbdeaa84f59eb87b` |
| `grid::tests` | 1,017 | `7d183fadd0668543df877f58247158c02a36541b3c327230b92eced892541adf` |
| `root_tests` | 235 | `2e616af2e7c7b7a0f480cba90b63b7807831bc72da4d42d5464a9e49e04610dc` |
| `flex_tests` | 169 | `5a1cd29203c4ee0eea91169e18512ffdd79bea40fe6237120d261490f8a6c0d5` |
| `block_tests` | 212 | `a73897032730d7b3b0d3c10746a17f16403baab9ffb1de4fedcdca01e2b3822f` |
| complete default package, including integration and doctests | 2,403 | not required |

The exact ignored leaf names are
`layout_oracle_grid_baseline_offset_matches_oracle` and
`runs_all_checked_in_browser_parity_xml`; their sorted multiset SHA-256 is
`235b22672841a0d7889b49ce4e7241b5abde0ba18311c8a23774d76597dfaf99`.
R07 preserves both. R08 owns whether either remains justified.

Partition every test by the primary production responsibility whose observable
contract it asserts. A test spanning settled phases goes to that suite's named
composition module. Keep a macro and every test it generates together. Keep a
helper local when one partition consumes it; move a helper used by multiple
partitions to that suite's `fixtures_tests.rs`, its sole shared-fixture owner.
Every partition imports shared fixtures explicitly through its test parent. The
existing `*_tests.rs` facade composes modules and exposes only test-private
imports or fixtures; it does not remain a second monolith.

Body and assertion preservation is literal except for module wrappers, imports,
test-private visibility, and relative fixture/source paths forced by the move.
The worker and reviewer inspect Git's moved-line diff for every task. The stable
leaf-name multisets above prove that nesting did not add, remove, rename, or
duplicate a discovered test, including macro-generated cases.

On 2026-08-14 the user confirmed that restoring the manifest-pinned Taffy
checkout is preapproved. After the predecessor's required `cargo clean`, the
coordinator may run the existing browser-free importer for exactly
`https://github.com/DioxusLabs/taffy.git` commit
`d1ff7e339b9ee35b33858779f8d7653197e93d92` into the documented target cache.
Workers do not acquire it. No other acquisition is authorized.

## 2 Impacts

- **Public API:** internal-only; exact crate-root names, paths, signatures,
  fields, constructors, defaults, errors, and facade remain unchanged.
- **Dependencies/features/MSRV:** unchanged; product and catalog lockfiles are
  unchanged.
- **Artifacts:** HTML, XML, `corpus.toml`, helper source, and `all.json` remain
  byte-identical; no generator or browser run occurs.
- **Docs/examples/root:** unchanged; root integration is not in scope.
- **Safety:** all repository-authored Rust remains free of `unsafe`; no new
  `allow`, `expect`, or `cfg_attr` suppression is added.

## 3 Tasks

### 3.1 `P01/I08/S02/R07/T01` Partition Grid Verification

**Area:** reduce `src/grid_tests.rs` to the composition facade and add
`src/grid_tests/{fixtures,topology_placement,tracks_intrinsic,lanes_subgrid,child_baseline,scroll_composition,oracle_comparison,browser_controls}_tests.rs`.

**Outcome:** grid tests follow topology/placement, tracks/intrinsic sizing,
lanes/subgrid, settled child/baseline, scroll/composition, oracle comparison,
browser-control, and shared-fixture owners. `grid::tests` remains the enclosing
test namespace and no production grid API changes.

**Characterization/migration evidence:** before mutation, run the full
`grid::tests::` prefix and record 1,017 passing-or-ignored discovered tests plus
the exact leaf multiset above. The structural migration probe is RED because
`src/grid_tests.rs` is the monolith and the directory does not exist. After the
move, the facade plus all eight child files exist, the facade contains only
composition/imports/shared-fixture exposure, the same prefix is executable, and
the same count/digest/ignored leaf is discovered.

**Acceptance:** every existing test body and assertion is present exactly once;
the four existing production-source proxy bodies retain their semantics and
receive only required `../grid/...` path corrections; shared fixtures have one
owner; the focused suite, full package, strict Clippy, formatting, moved-line
diff inspection, exact task scope, unsafe scan, and suppression scan pass.

**Commands:** locked/offline `cargo test -p surgeist-layout --lib
grid::tests::`; locked/offline library discovery and digest command from section
4; locked/offline full package test; strict locked/offline Clippy with
`-F unsafe-code -D warnings`; `cargo fmt --check`; `git diff --check`.

Dependency: published R06A. Commit: `refactor(test): partition grid verification`.

### 3.2 `P01/I08/S02/R07/T02` Partition Root Verification

**Area:** reduce `src/root_tests.rs` to the composition facade and add
`src/root_tests/{fixtures,requests,containing_contexts,transaction_cache,measurement,rounding}_tests.rs`.

**Outcome:** root tests follow root requests, containing contexts,
transaction/cache behavior, measurement, rounding, and shared-fixture owners.
Root composition, inline/float controls, and failure-atomicity cases go to the
owner whose published outcome they principally verify.

**Characterization/migration evidence:** before mutation, run `root_tests::`
and record 235 discovered tests and the exact leaf multiset above. The
structural probe is RED because only the monolith exists. After the move, the
facade plus all six child files exist and the count/digest remain exact.

**Acceptance:** bodies/assertions are preserved exactly once; the three existing
source-proxy tests retain their semantics and receive only required
`../block/...` or `../cache.rs` relative-path corrections; fixtures have one
owner; focused/full tests, strict Clippy, formatting, moved-line diff, exact
scope, unsafe, and suppression gates pass.

**Commands:** locked/offline `cargo test -p surgeist-layout --lib root_tests::`;
section 4 discovery/digest; locked/offline full package test; strict
locked/offline Clippy; `cargo fmt --check`; `git diff --check`.

Dependency: T01. Commit: `refactor(test): partition root verification`.

### 3.3 `P01/I08/S02/R07/T03` Partition Flex Verification

**Area:** reduce `src/flex_tests.rs` to the composition facade and add
`src/flex_tests/{fixtures,items,lines_distribution,alignment_baselines,intrinsic_absolute_scroll}_tests.rs`.

**Outcome:** flex tests follow item collection/sizing, line collection and
distribution, alignment/baselines, intrinsic/absolute/scroll composition, and
shared-fixture owners.

**Characterization/migration evidence:** before mutation, run `flex_tests::`
and record 169 discovered tests and the exact leaf multiset above. The
structural probe is RED because only the monolith exists. After the move, the
facade plus all five child files exist and the count/digest remain exact.

**Acceptance:** bodies/assertions and property cases are preserved exactly once;
all cross-partition fixtures have one test-only owner; no production helper or
visibility is added; focused/full tests, strict Clippy, formatting, moved-line
diff, exact scope, unsafe, and suppression gates pass.

**Commands:** locked/offline `cargo test -p surgeist-layout --lib flex_tests::`;
section 4 discovery/digest; locked/offline full package test; strict
locked/offline Clippy; `cargo fmt --check`; `git diff --check`.

Dependency: T02. Commit: `refactor(test): partition flex verification`.

### 3.4 `P01/I08/S02/R07/T04` Partition Block Verification

**Area:** reduce `src/block_tests.rs` to the composition facade and add
`src/block_tests/{fixtures,in_flow_margins,inline_runs,floats_bfcs,absolute,sizing_scroll}_tests.rs`.

**Outcome:** block tests follow in-flow/margins, inline runs, floats/BFCs,
absolute layout, sizing/scroll composition, and shared-fixture owners.

**Characterization/migration evidence:** before mutation, run `block_tests::`
and record 212 discovered tests and the exact leaf multiset above. The
structural probe is RED because only the monolith exists. After the move, the
facade plus all six child files exist and the count/digest remain exact.

**Acceptance:** bodies/assertions remain exact and unique; fixtures have one
test-only owner; no production helper or visibility is added; focused/full
tests, strict Clippy, formatting, moved-line diff, exact scope, unsafe, and
suppression gates pass.

**Commands:** locked/offline `cargo test -p surgeist-layout --lib block_tests::`;
section 4 discovery/digest; locked/offline full package test; strict
locked/offline Clippy; `cargo fmt --check`; `git diff --check`.

Dependency: T03. Commit: `refactor(test): partition block verification`.

## 4 Completion

Every worker and reviewer uses the following test-harness discovery command
after its move. It parses only `libtest` listing output, never Rust source:

```sh
set -e
lib_list="$(CARGO_NET_OFFLINE=true cargo test --locked --offline \
  -p surgeist-layout --lib -- --list 2>&1)"
count_prefix() {
  printf '%s\n' "$lib_list" | awk -v p="$1" \
    'index($0,p)==1 && /: test$/ {n++} END {print n+0}'
}
digest_prefix() {
  printf '%s\n' "$lib_list" | awk -F': test$' -v p="$1" \
    'index($0,p)==1 && /: test$/ {n=$1; sub(/^.*::/, "", n); print n}' \
    | LC_ALL=C sort | shasum -a 256 | awk '{print $1}'
}
test "$(count_prefix '')" = 2087
test "$(digest_prefix '')" = e0da19f5f8ff509122a3b1f846f0257cbd1e2c0ff850bf3cbbdeaa84f59eb87b
test "$(count_prefix 'grid::tests::')" = 1017
test "$(digest_prefix 'grid::tests::')" = 7d183fadd0668543df877f58247158c02a36541b3c327230b92eced892541adf
test "$(count_prefix 'root_tests::')" = 235
test "$(digest_prefix 'root_tests::')" = 2e616af2e7c7b7a0f480cba90b63b7807831bc72da4d42d5464a9e49e04610dc
test "$(count_prefix 'flex_tests::')" = 169
test "$(digest_prefix 'flex_tests::')" = 5a1cd29203c4ee0eea91169e18512ffdd79bea40fe6237120d261490f8a6c0d5
test "$(count_prefix 'block_tests::')" = 212
test "$(digest_prefix 'block_tests::')" = a73897032730d7b3b0d3c10746a17f16403baab9ffb1de4fedcdca01e2b3822f
package_list="$(CARGO_NET_OFFLINE=true cargo test --locked --offline \
  -p surgeist-layout -- --list 2>&1)"
test "$(printf '%s\n' "$package_list" | awk '/: test$/ {n++} END {print n+0}')" = 2403
ignored_list="$(CARGO_NET_OFFLINE=true cargo test --locked --offline \
  -p surgeist-layout -- --ignored --list 2>&1)"
ignored_leaves="$(printf '%s\n' "$ignored_list" | awk -F': test$' \
  '/: test$/ {n=$1; sub(/^.*::/, "", n); print n}' | LC_ALL=C sort)"
test "$ignored_leaves" = $'layout_oracle_grid_baseline_offset_matches_oracle\nruns_all_checked_in_browser_parity_xml'
test "$(printf '%s\n' "$ignored_leaves" | shasum -a 256 | awk '{print $1}')" = \
  235b22672841a0d7889b49ce4e7241b5abde0ba18311c8a23774d76597dfaf99
```

For each task, set its exact `task_base`, `task_head`, and `task_id`, then run:

```sh
case "$task_id" in
  T01) allowed='^(src/grid_tests\.rs|src/grid_tests/[^/]+_tests\.rs)$' ;;
  T02) allowed='^(src/root_tests\.rs|src/root_tests/[^/]+_tests\.rs)$' ;;
  T03) allowed='^(src/flex_tests\.rs|src/flex_tests/[^/]+_tests\.rs)$' ;;
  T04) allowed='^(src/block_tests\.rs|src/block_tests/[^/]+_tests\.rs)$' ;;
  *) exit 1 ;;
esac
task_paths="$(git diff --name-only "$task_base..$task_head" | LC_ALL=C sort -u)"
test -n "$task_paths"
test -z "$(printf '%s\n' "$task_paths" | rg -v "$allowed")"
git diff --check "$task_base..$task_head"
git diff --find-renames=1% --histogram --color-moved=blocks \
  --color-moved-ws=ignore-all-space "$task_base..$task_head" -- $task_paths
```

The final diff command is an explicit worker and reviewer inspection gate. Its
predicate is that every test/helper body and assertion is a moved line; the only
ordinary additions/deletions are module declarations/wrappers, imports,
test-private visibility, and the recorded relative-path corrections. The task
result enumerates every such non-moved exception. Any other changed token is a
task defect, even when the suite remains green.

After four independently clean task ranges, set this plan to `complete`. If the
pinned Taffy checkout is absent, the coordinator runs exactly:

```sh
CARGO_NET_OFFLINE=true cargo run --locked --offline -p surgeist-layout \
  --features layout-golden-generate --bin surgeist-layout-generate -- import-taffy
```

and proves the checkout Git HEAD is
`d1ff7e339b9ee35b33858779f8d7653197e93d92` with no repository delta.

Final discovery reruns the exact command above. No cargo test reads Rust source
to establish this inventory.

The exact final file inventory is the four slim facades and directory layouts in
section 3. Outside this plan and those test trees, the cycle diff is empty.
`src/lib.rs`, `src/grid/mod.rs`, all production behavior, Cargo manifests and
locks, README, scripts, catalog, fixtures, and generated artifacts match the
cycle base.

Final commands are:

```sh
set -e
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib grid::tests::
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib root_tests::
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib flex_tests::
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib block_tests::
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
audit_repo_root="$PWD"; (set -e; cd tools/surgeist-layout-audits; \
  CARGO_NET_OFFLINE=true CARGO_TARGET_DIR="$audit_repo_root/target/dylint-audits" \
  cargo +nightly-2026-05-28 test --locked --offline)
audit_repo_root="$PWD"; (set -e; cd tools/surgeist-layout-audits; \
  CARGO_NET_OFFLINE=true CARGO_TARGET_DIR="$audit_repo_root/target/dylint-audits" \
  RUSTFLAGS='-F unsafe-code -D warnings' \
  cargo +nightly-2026-05-28 check --locked --offline --all-targets)
(cd tools/surgeist-layout-audits && cargo +stable fmt --check)
cargo fmt --check
git diff --check
```

The final matrix also runs the following exact structure, scope, suppression,
and safety predicates:

```sh
set -e
expected_test_files="$(printf '%s\n' \
  src/{block,flex,grid,root}_tests.rs \
  src/block_tests/{absolute,fixtures,floats_bfcs,in_flow_margins,inline_runs,sizing_scroll}_tests.rs \
  src/flex_tests/{alignment_baselines,fixtures,intrinsic_absolute_scroll,items,lines_distribution}_tests.rs \
  src/grid_tests/{browser_controls,child_baseline,fixtures,lanes_subgrid,oracle_comparison,scroll_composition,topology_placement,tracks_intrinsic}_tests.rs \
  src/root_tests/{containing_contexts,fixtures,measurement,requests,rounding,transaction_cache}_tests.rs \
  | LC_ALL=C sort)"
actual_test_files="$({ printf '%s\n' src/{block,flex,grid,root}_tests.rs; \
  find src/block_tests src/flex_tests src/grid_tests src/root_tests \
    -type f -name '*.rs' -print; } | LC_ALL=C sort)"
test "$actual_test_files" = "$expected_test_files"
cycle_paths="$(git diff --name-only \
  496baae07f7a1216cab51267848231da82970941..HEAD | LC_ALL=C sort -u)"
test -z "$(printf '%s\n' "$cycle_paths" | rg -v \
  '^(plans/cycles/P01-I08-S02-R07-test-ownership\.md|src/(block_tests|flex_tests|grid_tests|root_tests)(\.rs|/[^/]+_tests\.rs))$')"
test "$(git diff 496baae07f7a1216cab51267848231da82970941..HEAD \
  -- Cargo.toml Cargo.lock README.md Justfile src/lib.rs src/grid/mod.rs tools tests scripts | wc -l | tr -d ' ')" = 0
base_suppressions="$(while IFS= read -r p; do
  git show "496baae07f7a1216cab51267848231da82970941:$p" \
    | perl -0777 -ne 'while (/^[ \t]*#\s*\[\s*(?:allow|expect|cfg_attr)\b[^\]]*\]/gms) {$m=$&;$m=~s/\s+/ /g;print "$m\n"}'
done < <(git ls-tree -r --name-only 496baae07f7a1216cab51267848231da82970941 \
  | rg '\.rs$') | LC_ALL=C sort)"
current_suppressions="$({ git ls-files -z -- '*.rs'; \
  git ls-files -z --others --exclude-standard -- '*.rs'; } | sort -zu \
  | xargs -0 perl -0777 -ne 'while (/^[ \t]*#\s*\[\s*(?:allow|expect|cfg_attr)\b[^\]]*\]/gms) {$m=$&;$m=~s/\s+/ /g;print "$m\n"}' \
  | LC_ALL=C sort)"
test -z "$(comm -13 <(printf '%s\n' "$base_suppressions") \
  <(printf '%s\n' "$current_suppressions"))"
if { git ls-files -z -- '*.rs'; git ls-files -z --others \
  --exclude-standard -- '*.rs'; } | sort -zu | xargs -0 rg -n --pcre2 \
  '#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{'; then
  exit 1
fi
```

It also proves public API inventory, unchanged dependencies/features/MSRV/locks,
frozen hashes
`c6e6f1422e14a5e4aa474c143998063ce0de4d0a9123b69875b35a4ed009a8f6`
(`corpus.toml`),
`c684c7f167d95997a4a9f0250467bbaf72c1b73e69e0f707a2ef32f4d25f7f36`
(helper), and
`c10dc550d260a239c8bf9dd553f5272ca3bcc2826099bc182f800986b8b94c0e`
(`all.json`), 1,448 HTML, 5,776 XML, exact path scope, and clean Git state.
The selected node-projection Dylint audit does not recur.

After clean holistic review, publish immutable `main`, read back the authority
remote, prove no cycle-owned process remains, run repository-root `cargo clean`,
prove both target paths absent and Git clean, and hand the published partitioned
candidate to R08. R08 owns whole-crate evidence classification, source-proxy
removal/replacement, current-output re-oracling, ignored-test justification, and
final FRI-08 acceptance.
