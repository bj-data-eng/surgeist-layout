# P01-I08-S01-C08 Whole-Crate Sprawl Containment II And Final Candidate

Status: complete

Cycle ID: `P01/I08/S01/C08`

Owning repository: `surgeist-layout`

Cycle base: `4f14431cb000fceb97be7f3203927b5e5f7d07cd`

Reviewed specification:
`plans/specs/P01-I08-grid-subgrid-and-grid-lanes-completeness.md`, normalized
semantic-content SHA-256
`a636dd9c9b896e2986fd13ab303f8506fba7eec6b0ba909e542eee9dc39770e6`,
commit `09bab4edc2bbff4aad42469937a328d0724989c0`: `FRI-08.14` through
`FRI-08.19`.

Reviewed implementation sequence:
`plans/sequences/P01-I08-S01-grid-subgrid-and-grid-lanes-completeness.md`,
normalized semantic-content SHA-256
`bcda6e13068a0424a9139b7d1933717c43263d23eb080ec39ef4d1e8fd331f94`,
commit `b55c4db0eb20265f5bf086a50b8cc6f08a5ecf82`, entry
`P01/I08/S01/C08`.

Bounded outcome: close the accepted whole-crate report's remaining `SP-006`
through `SP-010`, `SP-012`, `ESC-001`, and `ESC-002` rows with
characterization-first, behavior-preserving consolidation, then publish the
final FRI-08 leaf candidate and return the complete initiative audit.

## 1 Boundary

The remotely verified C07 candidate at the cycle base is immutable. C07 closed
`SP-002`, `SP-003`, `SP-004`, `SP-005`, and `SP-011`; C06R and C02R had already
closed the inherited-placement and grid-lanes track-phase findings. All eight
FRI-08 behavior findings, public unsupported-capability removal, 72 owned rows,
centralized provenance, later-owned controls, and frozen artifact lineage are
green at entry.

The accepted whole-crate review classified all 7,388 tracked files and 1,515
behavior-bearing targets. Its remaining finite rows are:

- `SP-006`: the two duplicate generator fixture collectors;
- `SP-007`: ordinary and calc-size fixture-expression parser duplication;
- `SP-008`: repeated grid-test retained-state setup;
- `SP-009`: repeated grid comparison-tree walks;
- `SP-010`: repeated browser fixture-family runners;
- `SP-012`: unreachable independent-formatting-context subgrid state;
- `ESC-001`: impossible vertical-writing-mode error and its suppression;
- `ESC-002`: fourteen bare source `allow` attributes.

The generator expansion is only the user-authorized `SP-006` correction. It
replaces `collect_relative_files_into` and `collect_html_into` with one private,
deterministic traversal owner. The semantically separate generated-XML pruning
walk remains unchanged. No reusable generator layer, new path, output operation,
browser run, generation, fixture edit, artifact edit, or unrelated cleanup is
authorized.

Every task is behavior-preserving. A characterization difference or genuine
correctness defect stops that task before consolidation and returns exact
evidence. Out of scope: new behavior or public API; dependencies, features,
lockfile, MSRV, docs, authored HTML, helper/adapter scripts, manifest, XML,
reports, root/sibling work, FRI-09/F10 behavior, and new lint suppressions or
unsafe.

## 2 Impacts

Public API, errors, types, reexports, defaults, observable layout, dependencies,
features, MSRV, docs, root facade, and generated artifacts remain unchanged.
Removed states and errors are crate-private impossibilities. New helpers remain
crate-private or test-private, name their fixture/algorithm phase, and use typed
inputs instead of booleans or positional argument clusters.

No browser or artifact-writing generator command is authorized. Browser-free
generator-feature tests and read-only corpus/Taffy checks are required. Frozen
SHA-256 values are:

- `corpus.toml`:
  `c6e6f1422e14a5e4aa474c143998063ce0de4d0a9123b69875b35a4ed009a8f6`;
- helper:
  `c684c7f167d95997a4a9f0250467bbaf72c1b73e69e0f707a2ef32f4d25f7f36`;
- sole `all.json`:
  `c10dc550d260a239c8bf9dd553f5272ca3bcc2826099bc182f800986b8b94c0e`;
- complete XML inventory:
  `a98d1ccceaeeb336ee3cb3c0151607589c0a4ae0376a46c560ba4341f95ad6ae`;
- complete XML hash lineage:
  `bad8e418caee72cc62a123dc93efe89fdb07bfb5dee4345f3df7d8fd6fe44fdf`.

Owned Rust remains free of unsafe. The cycle removes all fourteen bare `allow`
attributes, removes the obsolete `expect(dead_code)` owned by `ESC-001`, and
adds no `allow` or `expect` attribute.

## 3 Shared Task Gate

Tasks run in order. Each behavior-preserving refactor first proves its named
behavioral characterization on the exact task base. A separate read-only source
inventory records the named duplicate or impossible state before the edit and
its absence afterward; source shape is never used as behavioral test evidence.
The worker performs the mechanical change, repeats focused commands, and runs:

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout
CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --features layout-golden-generate --all-targets -- -F unsafe-code -D warnings
cargo fmt --check
git diff --check
```

Each task proves exact assigned-file scope, no added `allow` or `expect`, zero
unsafe across every tracked and non-ignored owned Rust source, and a clean
worktree after one logical commit. The coordinator supplies exact full
`TASK_BASE` and newline-delimited `TASK_FILES`; any necessary edit outside that
envelope is returned before mutation.

```sh
test -z "$(comm -23 <(git diff --name-only "$TASK_BASE"..HEAD | LC_ALL=C sort) <(printf '%s\n' "$TASK_FILES" | LC_ALL=C sort))"
if git diff --word-diff=porcelain --word-diff-regex='[[:alpha:]_][[:alnum:]_]*' "$TASK_BASE"..HEAD -- '*.rs' | rg '^\+.*\b(allow|expect)\b'; then exit 1; fi
unsafe_hits="$({ git ls-files -z -- '*.rs'; git ls-files -z --others --exclude-standard -- '*.rs'; } | sort -zu | xargs -0 rg -n '\bunsafe\b' | rg -v '^[^:]+:[0-9]+:[[:space:]]*unreachable!\("safe_fallback returns unsafe (item|content) alignment"\)$|^src/lib_tests\.rs:[0-9]+:[[:space:]]*Some\("async" \| "unsafe" \| "default" \| "extern"\) => keyword \+= 1,$|^src/lib_tests\.rs:[0-9]+:[[:space:]]*"removed phase-unsafe surface remains: \{removed\}"$|^tests/layout/browser_parity/support\.rs:[0-9]+:[[:space:]]*let has_overflow_prefix = safe \|\| raw\.starts_with\("unsafe "\);$|^tests/layout/browser_parity/support\.rs:[0-9]+:[[:space:]]*\.or_else\(\|\| raw\.strip_prefix\("unsafe "\)\)$|^tests/layout/browser_parity/support\.rs:[0-9]+:[[:space:]]*parse_align_content\("unsafe end"\)\.expect\("unsafe content alignment should parse"\),$|^tests/layout/browser_parity/support\.rs:[0-9]+:[[:space:]]*assert!\(parse_align_items\("unsafe first baseline"\)\.is_err\(\)\);$' || true)"
test -z "$unsafe_hits"
if { git ls-files -z -- '*.rs'; git ls-files -z --others --exclude-standard -- '*.rs'; } | sort -zu | xargs -0 rg -n -U '\b(no_mangle|export_name|link_section|naked)\b|(^|[^[:alnum:]_"])extern[[:space:]]*"'; then exit 1; fi
```

## 4 Tasks

### 4.1 `P01/I08/S01/C08/T01` Unify Generator Fixture Traversal

**Finding and write envelope:** `SP-006`; exactly
`tests/bin/surgeist-layout-generate/generator.rs`. Depends only on the reviewed
plan and cycle base. Stop before every other generator, helper, fixture, or
artifact file.

**Outcome:** one private recursive collector owns deterministic entry walking,
read and entry diagnostics, and sorting. Existing callers retain their relative
versus absolute projection and HTML filtering. Generated-XML pruning traversal
and every output path remain separate and unchanged.

**Characterization and acceptance:** new
`fri08_c08_t01_generator_traversal_` tests first
characterize nested ordering, relative/absolute paths, extension filtering,
non-UTF-8-safe path handling, and missing/unreadable-directory diagnostics, then
pass unchanged after consolidation. A separate source inventory records two
fixture collector owners before and one after. Exact vectors and diagnostics
match; production outside the private traversal is unchanged.

**Mechanical inventory:** run outside the test suite; base has exactly the two
named fixture collectors, while HEAD has one new owner and retains the separate
generated-XML collector.

```sh
test "$(git show "$TASK_BASE":tests/bin/surgeist-layout-generate/generator.rs | rg -c '^fn (collect_relative_files_into|collect_html_into)\(')" = 2
test -z "$(rg '^fn (collect_relative_files_into|collect_html_into)\(' tests/bin/surgeist-layout-generate/generator.rs || true)"
test "$(rg -c '^fn collect_fixture_files_into\(' tests/bin/surgeist-layout-generate/generator.rs)" = 1
test "$(rg -c '^fn collect_generated_xml_files\(' tests/bin/surgeist-layout-generate/generator.rs)" = 1
```

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --features layout-golden-generate fri08_c08_t01_generator_traversal_
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
```

**Intended commit:** `refactor(generator): unify fixture traversal`.

### 4.2 `P01/I08/S01/C08/T02` Unify Fixture Expressions And Family Runs

**Findings and write envelope:** `SP-007` and `SP-010`; exactly
`tests/layout/browser_parity/support.rs` and `tests/layout/browser_parity.rs`.
Depends on clean T01 only for ordered attribution.

**Outcome:** parse ordinary and calc-size fixture expressions through one
test-private structural expression representation with destination-specific
validated lowering. Run all five block, flex, grid, grid-lanes, and subgrid axis
families through one typed family harness; topology predicates remain local.

**Characterization and acceptance:** new `fri08_c08_t02_` tests characterize
every supported
leaf/function, nesting limit, optional clamp bound, malformed argument,
destination-specific rejection and diagnostic, plus all five family inventories,
topology checks, paths, and mismatch identity. A separate source inventory records
the duplicate parser recursion and family loops before and their single owners
after. All pre/post results are exact.

**Mechanical inventory:** run outside the test suite; base parser-owner count is
two and the generic family harness has two callers. HEAD has one structural
parser and all five family callers.

```sh
test "$(git show "$TASK_BASE":tests/layout/browser_parity/support.rs | rg -c '^fn (parse_sizing_calculation_inner|parse_calc_size_calculation_inner)\(')" = 2
test "$(git show "$TASK_BASE":tests/layout/browser_parity.rs | rg -c 'assert_axis_fixture_family_matches\(')" = 3
test -z "$(rg '^fn (parse_sizing_calculation_inner|parse_calc_size_calculation_inner)\(' tests/layout/browser_parity/support.rs || true)"
test "$(rg -c '^fn parse_fixture_sizing_expression\(' tests/layout/browser_parity/support.rs)" = 1
test "$(rg -c 'assert_axis_fixture_family_matches\(' tests/layout/browser_parity.rs)" = 6
```

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --features layout-golden-generate --test layout fri08_c08_t02_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --features layout-golden-generate --test layout fri08_c0
CARGO_NET_OFFLINE=true just verify-generator
```

**Intended commit:** `refactor(parity): unify fixture expression harnesses`.

### 4.3 `P01/I08/S01/C08/T03` Share Grid Retained-State Setup

**Finding and write envelope:** `SP-008`; exactly `src/grid_tests.rs`. Depends
on clean T02 only for ordered attribution.

**Outcome:** one test-private typed retained-state input and batch-preparation
owner replaces repeated atomic state, cache, and completion setup. Per-scenario
tree/style/input and expected behavior remain local.

**Characterization and acceptance:** new `fri08_c08_t03_retained_state_` tests
characterize
initial revisions, prepared batches, success/failure publication, cache identity,
f32/f64, and no-partial-state behavior. A separate source inventory records the
duplicate setup before and its single owner after. Existing grid behavior and
transaction tests remain exact.

**Mechanical inventory:** run outside the test suite; base contains three
scenario-specific retained stores. HEAD contains one store and one batch owner
used by all three existing sink implementations.

```sh
test "$(git show "$TASK_BASE":src/grid_tests.rs | rg -c '^struct Fri08C(06RRetained|03NestedRetained|04BaselineRetained)')" = 3
test -z "$(rg '^struct Fri08C(06RRetained|03NestedRetained|04BaselineRetained)' src/grid_tests.rs || true)"
test "$(rg -c '^struct GridTestRetainedState' src/grid_tests.rs)" = 1
test "$(rg -c 'prepare_grid_test_batch\(' src/grid_tests.rs)" = 4
```

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib fri08_c08_t03_retained_state_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib grid_tests::
```

**Intended commit:** `refactor(grid-tests): share retained state setup`.

### 4.4 `P01/I08/S01/C08/T04` Unify Grid Comparison Walks

**Finding and write envelope:** `SP-009`; exactly
`src/test_support/grid_layout_comparison.rs` and `src/grid_tests.rs`. Depends on
clean T03 because the test module overlaps.

**Outcome:** one test-private identity map and deterministic comparison walker
owns corresponding-node traversal. Case construction and algorithm-specific
geometry, baseline, scroll, and error assertions remain local.

**Characterization and acceptance:** new `fri08_c08_t04_comparison_walk_` tests
characterize
source identity, child order, missing/duplicate identities, nested mismatch
paths, both scalars, and exact diagnostic order. A separate source inventory
records repeated walks before and one identity-map/walker owner after.
Production/oracle comparison behavior remains exact.

**Mechanical inventory:** run outside the test suite; base contains the three
reconstructed-numbering functions. HEAD removes them and has exactly one private
identity-map builder and one phase-parameterized walker.

```sh
test "$(git show "$TASK_BASE":src/test_support/grid_layout_comparison.rs | rg -c '^fn (append_node|assert_nested_expected_layouts|assert_nested_expected_final_layouts)\(')" = 3
test -z "$(rg '^fn (append_node|assert_nested_expected_layouts|assert_nested_expected_final_layouts)\(' src/test_support/grid_layout_comparison.rs || true)"
test "$(rg -c '^fn build_grid_comparison_identity_map\(' src/test_support/grid_layout_comparison.rs)" = 1
test "$(rg -c '^fn walk_grid_comparison_expectations\(' src/test_support/grid_layout_comparison.rs)" = 1
```

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib fri08_c08_t04_comparison_walk_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib grid_tests::
CARGO_NET_OFFLINE=true just taffy-check
```

**Intended commit:** `refactor(grid-tests): unify comparison walks`.

### 4.5 `P01/I08/S01/C08/T05` Remove Impossible Grid States

**Findings and write envelope:** `SP-012` and `ESC-001`; exactly
`src/grid/axis.rs`, `src/grid/subgrid.rs`, `src/grid/mod.rs`,
`src/grid/child.rs`, `src/grid/tracks.rs`, `src/grid/lanes.rs`, and
`src/grid_tests.rs`. Depends on clean T04 because the tests overlap. Oracle
source remains frozen.

**Outcome:** make grid-axis mapping infallible for all supported `FlowAxes` and
remove the impossible private vertical-writing-mode error, `Result` plumbing,
and dead-code expectation. Remove the unreachable independent-formatting-context
subgrid ineligibility state and branch while preserving every reachable
standalone, nested, lanes, intrinsic, and scroll-container decision.

**Characterization and acceptance:** new `fri08_c08_t05_impossible_state_` tests
first prove
all flow axes, scalars, caller projections, subgrid eligibility combinations,
standalone termination, nested lanes, exact remaining errors, and public output,
and pass unchanged after removal. A separate source inventory is nonempty before
the edit and requires both impossible states and the obsolete expectation absent
afterward. No valid failure is erased.

**Mechanical inventory:** run outside the test suite; every named base symbol is
required to exist and every HEAD match across the exact task envelope is empty.

```sh
for symbol in GridAxisMappingError VerticalWritingModeUnsupported IndependentFormattingContext establishes_independent_formatting_context; do git grep -q "$symbol" "$TASK_BASE" -- src/grid/axis.rs src/grid/subgrid.rs src/grid/mod.rs src/grid/child.rs src/grid/tracks.rs src/grid/lanes.rs src/grid_tests.rs; done
test -z "$(rg -n 'GridAxisMappingError|VerticalWritingModeUnsupported|IndependentFormattingContext|establishes_independent_formatting_context' src/grid/{axis,subgrid,mod,child,tracks,lanes}.rs src/grid_tests.rs || true)"
```

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib fri08_c08_t05_impossible_state_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib fri08_c06r_inherited_placement_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib fri08_c02r_lanes_track_phase_
```

**Intended commit:** `refactor(grid): remove impossible states`.

### 4.6 `P01/I08/S01/C08/T06` Eliminate Bare Source Suppressions

**Finding and write envelope:** `ESC-002`; exactly `src/grid_tests.rs` and every
tracked Rust file below `src/test_support/` at the task base. Depends on clean
T05. Stop before production modules outside
`src/test_support/`, integration tests, or another source tree.

**Outcome:** remove the three module `dead_code`, ten oracle-grid
`unused_imports`, and one grid-test `too_many_arguments` allows. Delete or
privatize genuinely unused test-only exports and imports, and replace the
positional helper argument cluster with one typed test input. Do not add or
relocate suppressions, weaken tests, or make production-visible surface.

**Characterization and acceptance:** new `fri08_c08_t06_suppression_cleanup_`
tests characterize the live consumers implicated by the removed module/import
allows and the large helper before restructuring and pass unchanged afterward.
A read-only owned-source scan reports the exact fourteen bare allows before the
edit and zero after. The final suppression inventory equals the task-base
inventory minus those fourteen allows, with no added or relocated attribute; the
repository-wide unsafe scan remains empty.

**Mechanical inventory:** run outside the test suite; the exact task-base scope
has fourteen direct bare allows and HEAD has none. The shared diff inventory also
rejects every added or relocated suppression token.

```sh
test "$(git grep -n -F '#[allow(' "$TASK_BASE" -- src/grid_tests.rs src/test_support | wc -l | tr -d ' ')" = 14
test -z "$(rg -n -F '#[allow(' src/grid_tests.rs src/test_support --glob '*.rs' || true)"
allow_hits="$({ git ls-files -z -- '*.rs'; git ls-files -z --others --exclude-standard -- '*.rs'; } | sort -zu | xargs -0 rg -n -U --pcre2 '#\s*\[\s*allow\b' | rg -v '^src/contract_tests\.rs:65:[[:space:]]*!text_source\.contains\("#\[allow\(dead_code\)\]"\),$|^src/lib_tests\.rs:984:[[:space:]]*"#\[allow\(dead_code\)\]",$|^src/lib_tests\.rs:985:[[:space:]]*"#\[allow\(clippy::too_many_arguments\)\]",$|^src/lib_tests\.rs:2180:[[:space:]]*"#\[allow\(dead_code\)\] /\* between attributes \*/ #\[cfg_attr\(not\(test\), cfg\(test\)\)\] pub\(crate\) fn hidden\(\) \{ scrollbar_size; \}",$' || true)"
test -z "$allow_hits"
```

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib fri08_c08_t06_suppression_cleanup_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib grid_tests::
```

**Intended commit:** `refactor(test-support): eliminate bare suppressions`.

## 5 Final Verification And Completion

After all six exact ordered task ranges have clean independent task reviews,
run:

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri08_c08_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri08_c02r_lanes_track_phase_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri08_c06r_inherited_placement_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --test layout fri08_c0
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --features layout-golden-generate --all-targets -- -F unsafe-code -D warnings
cargo fmt --check
git diff --check
```

Final evidence also proves the union of the six recorded task spans plus this
plan is exactly the cycle's changed-file set in both directions; all fourteen
bare allows and the obsolete dead-code expectation are absent; no new
suppression exists; owned Rust passes the lexical and raw unsafe inventories;
the browser input/output tree and frozen hashes remain exact; there are 1,448
HTML and 5,776 comment-free XML files; the schema-3 report remains 5,776
generated, 16 unsupported, three expected-fail, zero quarantined, and zero
failed-to-generate; and the worktree is clean.

```sh
expected_paths="$({ printf '%s\n' 'plans/cycles/P01-I08-S01-C08-whole-crate-sprawl-containment-and-final-candidate.md'; while IFS= read -r span; do git diff --name-only "$span"; done <<< "$TASK_SPANS"; } | LC_ALL=C sort -u)"
actual_paths="$(git diff --name-only 4f14431cb000fceb97be7f3203927b5e5f7d07cd..HEAD | LC_ALL=C sort -u)"
test "$actual_paths" = "$expected_paths"
allow_hits="$({ git ls-files -z -- '*.rs'; git ls-files -z --others --exclude-standard -- '*.rs'; } | sort -zu | xargs -0 rg -n -U --pcre2 '#\s*\[\s*allow\b' | rg -v '^src/contract_tests\.rs:65:[[:space:]]*!text_source\.contains\("#\[allow\(dead_code\)\]"\),$|^src/lib_tests\.rs:984:[[:space:]]*"#\[allow\(dead_code\)\]",$|^src/lib_tests\.rs:985:[[:space:]]*"#\[allow\(clippy::too_many_arguments\)\]",$|^src/lib_tests\.rs:2180:[[:space:]]*"#\[allow\(dead_code\)\] /\* between attributes \*/ #\[cfg_attr\(not\(test\), cfg\(test\)\)\] pub\(crate\) fn hidden\(\) \{ scrollbar_size; \}",$' || true)"; test -z "$allow_hits"
if { git ls-files -z -- '*.rs'; git ls-files -z --others --exclude-standard -- '*.rs'; } | sort -zu | xargs -0 rg -n -U '\b(no_mangle|export_name|link_section|naked)\b|(^|[^[:alnum:]_"])extern[[:space:]]*"'; then exit 1; fi
```

`TASK_SPANS` is the newline-delimited ordered list of every initial and fix span
accepted by the six task reviews. A fresh holistic reviewer must return clean
for the exact cycle range.

C08 completes only after every accepted whole-crate report row and both prior
behavior findings have one final disposition; all 59 initial findings, eight
FRI-08 closures, 72 rows, provenance, artifacts, public removal, FRI-09/F10
controls, dependencies/features/MSRV, and unsafe policy are audited; and the
candidate is published with local `main`, its tracking ref, and observed remote
`main` read back at one SHA. The final handoff records leaf revision, complete
finding/sprawl disposition, artifact lineage, root integration ownership, and
later-P01 continuation. No blocker is currently known.
