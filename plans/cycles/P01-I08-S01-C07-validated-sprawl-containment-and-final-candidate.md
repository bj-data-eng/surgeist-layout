# P01-I08-S01-C07 Whole-Crate Sprawl Containment I

Status: in_progress

Cycle ID: `P01/I08/S01/C07`

Owning repository: `surgeist-layout`

Cycle base: `dc71a5582ab0ef3925826dce09b93ee9fa6f49a1`

Reviewed specification:
`plans/specs/P01-I08-grid-subgrid-and-grid-lanes-completeness.md`, normalized
semantic-content SHA-256
`a636dd9c9b896e2986fd13ab303f8506fba7eec6b0ba909e542eee9dc39770e6`,
commit `09bab4edc2bbff4aad42469937a328d0724989c0`: finite structural
invariants in `FRI-08.14` and verification, responsibility, finding closure,
handoff, and acceptance in `FRI-08.15` through `FRI-08.19`.

Reviewed implementation sequence:
`plans/sequences/P01-I08-S01-grid-subgrid-and-grid-lanes-completeness.md`,
normalized semantic-content SHA-256
`bcda6e13068a0424a9139b7d1933717c43263d23eb080ec39ef4d1e8fd331f94`,
commit `b55c4db0eb20265f5bf086a50b8cc6f08a5ecf82`, entry
`P01/I08/S01/C07`.

Bounded outcome: implement the C07 partition of the accepted whole-crate sprawl
report—`SP-002`, `SP-003`, `SP-004`, `SP-005`, and `SP-011`—with
characterization-first behavior-preserving consolidation, then publish the
first sprawl-containment candidate for just-in-time C08 planning.

## 1 Boundary

The remotely verified C02R candidate at the cycle base is immutable. It closes
all eight FRI-08 behavior findings, including the C06R inherited-placement and
C02R lanes-track-phase corrections, and carries 1,448 HTML inputs, 5,776
comment-free XML outputs, 72 owned FRI-08 rows, and the schema-3 report with
5,776 generated, 16 unsupported, three unchanged FRI-07 expected-fail, zero
quarantined, and zero failed-to-generate rows.

The accepted initiative sprawl review classified every one of 7,388 tracked
files and 1,515 behavior-bearing targets at the cycle base. Both offline
warning-denied Clippy rows passed, owned Rust had zero unsafe constructs, and
the verdict was `REPORT_WITH_ESCALATIONS`: eleven mechanical opportunities and
two lint-policy escalations. Earlier commit-range “sprawl” verdicts are
superseded because they did not cover the entire crate.

The reviewed sequence provides the disjoint exhaustive partition:

- C07 owns `SP-002`, `SP-003`, `SP-004`, `SP-005`, and `SP-011`;
- future C08 owns `SP-006` through `SP-010`, `SP-012`, `ESC-001`, and
  `ESC-002`.

C07 does not pre-author C08. It preserves the remaining rows, including the
user-authorized bounded generator traversal consolidation, for the C08 plan
after this cycle is published. Existing bare source suppressions and impossible
grid scaffolding are carried unchanged rather than silently accepted; C08 owns
their correction.

Every C07 edit is behavior-preserving. A characterization failure or evidence
of a genuine behavior, correctness, public-contract, input, or artifact defect
stops the owning task before refactoring and returns the exact evidence to the
coordinator.

Out of scope: C08-owned rows; behavior or public API changes; dependencies,
features, lockfile, MSRV, docs, authored fixtures, helper/adapter/generator,
HTML/XML/report/manifest, browser execution, generation, root/sibling work,
later FRI-09/F10 behavior, lint policy, unsafe, and unrelated cleanup.

## 2 Impacts

Public API, errors, types, reexports, defaults, feature behavior, dependencies,
MSRV, artifacts, and observable layout: unchanged. Helpers are crate-private or
test-private and encode policy with named types rather than booleans or sentinel
optionality.

No browser execution or artifact-generating/writing generator command is
authorized. Browser-free generator-feature tests plus read-only corpus and Taffy
validation are required and do not authorize `generate`, `generate-existing`,
acquisition, or artifact cleanup. Frozen SHA-256 values are:

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

Owned Rust remains free of unsafe. C07 adds no `allow` or `expect` and does not
weaken or relocate the C08-owned suppression evidence.

## 3 Shared Task Gate

Tasks run in the order below. Each worker first proves its new characterization
passes on the task base, performs only the named mechanical edit, repeats the
focused commands, and then runs:

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout
CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --features layout-golden-generate --all-targets -- -F unsafe-code -D warnings
cargo fmt --check
git diff --check
```

Each task additionally proves exact assigned-file scope, no new `allow` or
`expect`, zero unsafe matches across the repository-owned Rust manifest, and a
clean worktree after its logical commit. A required edit outside the exact write
envelope is a blocker returned before that edit.

Each worker receives its exact full `TASK_BASE` SHA and newline-delimited
`TASK_FILES` from the task assignment and runs these executable proofs:

```sh
test -z "$(comm -23 <(git diff --name-only "$TASK_BASE"..HEAD | LC_ALL=C sort) <(printf '%s\n' "$TASK_FILES" | LC_ALL=C sort))"
if git diff --word-diff=porcelain --word-diff-regex='[[:alpha:]_][[:alnum:]_]*' "$TASK_BASE"..HEAD -- '*.rs' | rg '^\+.*\b(allow|expect)\b'; then exit 1; fi
unsafe_hits="$({ git ls-files -z -- '*.rs'; git ls-files -z --others --exclude-standard -- '*.rs'; } | sort -zu | xargs -0 rg -n '\bunsafe\b' | rg -v '^[^:]+:[0-9]+:[[:space:]]*unreachable!\("safe_fallback returns unsafe (item|content) alignment"\)$|^src/lib_tests\.rs:[0-9]+:[[:space:]]*Some\("async" \| "unsafe" \| "default" \| "extern"\) => keyword \+= 1,$|^src/lib_tests\.rs:[0-9]+:[[:space:]]*"removed phase-unsafe surface remains: \{removed\}"$|^tests/layout/browser_parity/support\.rs:[0-9]+:[[:space:]]*let has_overflow_prefix = safe \|\| raw\.starts_with\("unsafe "\);$|^tests/layout/browser_parity/support\.rs:[0-9]+:[[:space:]]*\.or_else\(\|\| raw\.strip_prefix\("unsafe "\)\)$|^tests/layout/browser_parity/support\.rs:[0-9]+:[[:space:]]*parse_align_content\("unsafe end"\)\.expect\("unsafe content alignment should parse"\),$|^tests/layout/browser_parity/support\.rs:[0-9]+:[[:space:]]*assert!\(parse_align_items\("unsafe first baseline"\)\.is_err\(\)\);$' || true)"
test -z "$unsafe_hits"
if { git ls-files -z -- '*.rs'; git ls-files -z --others --exclude-standard -- '*.rs'; } | sort -zu | xargs -0 rg -n -U '\b(no_mangle|export_name|link_section|naked)\b|(^|[^[:alnum:]_"])extern[[:space:]]*"'; then exit 1; fi
```

The first command exits zero only when every changed file is inside the
assignment envelope. The conservative word-diff scan rejects any newly added
`allow` or `expect` token regardless of direct, multiline, or `cfg_attr` nesting.
The raw unsafe-token scan fails on every token-separated or multiline `unsafe`
outside the exact frozen safe-string inventory; the companion scan rejects every
raw unsafe-attribute name and foreign ABI across the same complete file set.
`TASK_FILES` contains one path per line, not a shell glob.

## 4 Tasks

### 4.1 `P01/I08/S01/C07/T01` Unify Visible Inline-Run Transitions

**Finding:** `SP-002`, high-impact functional duplication with repeated
multi-responsibility transition plumbing at `src/block.rs:1206-1457`.

**Write envelope:** exactly `src/block.rs` and `src/block_tests.rs`. Depends only
on the reviewed C07 plan and cycle base. Stop before every other file.

**Outcome:** retain role-specific scan, input validation, and preprocessing,
then route text, inline boundary, explicit line break, and inline-box starts
through one private transition owner for collapsed-margin advance, child layout,
content/scroll maxima, baselines, cursor/static position, float transfer, and
collapse-state reset.

**Characterization and acceptance:** new `fri08_c07_t01_inline_transition_`
public-front-door tests cover all four start roles, hidden breaks, source
indices, margins, floats, baselines, scroll geometry, invalid-input ordering,
and f32/f64. They pass before and after the edit with exact output/error
identity. No role-specific precondition moves into the shared transition.

**Focused commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib fri08_c07_t01_inline_transition_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib block_tests::
```

**Intended commit:** `refactor(block): unify inline run transitions`.

### 4.2 `P01/I08/S01/C07/T02` Centralize Canonical Scroll Source Assembly

**Finding:** `SP-003`, high-impact functional duplication and ripple coupling in
`src/block.rs:2999-3126`, `src/flex.rs:3306-3411`,
`src/compute.rs:1640-1724`, with the lower owner in `src/scroll.rs:2466-2697`.

**Write envelope:** exactly `src/scroll.rs`, `src/block.rs`, `src/flex.rs`,
`src/compute.rs`, `src/block_tests.rs`, `src/flex_tests.rs`, and
`src/root_tests.rs`. Depends on clean T01 because `src/block.rs` and its tests
overlap. Stop before every other production or test file.

**Outcome:** add one crate-private canonical retained/source builder with a
named range-seed policy. Block, flex, and root retain contribution construction,
block-only reserved-gutter exclusion, and caller-local error mapping.

**Characterization and acceptance:** new `fri08_c07_t02_scroll_source_` tests in
the three owned test modules cover existing-geometry fast paths, retained-child
reconstruction, flow axes, settled scrollbar state, clips, padding, targets,
snapping, origins, block-only gutter exclusion, errors, and both scalars. Exact
public node output and failure identity pass before and after. No policy becomes
a boolean, zero sentinel, or ambiguous option.

**Focused commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib fri08_c07_t02_scroll_source_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib block_tests::
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib flex_tests::
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib root_tests::
```

**Intended commit:** `refactor(scroll): centralize geometry sources`.

### 4.3 `P01/I08/S01/C07/T03` Consolidate Optional Layout Arithmetic

**Finding:** `SP-004`, medium-impact exact duplicate arithmetic in
`src/block.rs:3923-3949`, `src/flex.rs:4507-4544`, and
`src/compute.rs:2952-2967`; natural owner `src/layout_math.rs`.

**Write envelope:** exactly `src/layout_math.rs`, `src/block.rs`, `src/flex.rs`,
`src/compute.rs`, `src/block_tests.rs`, `src/flex_tests.rs`, and
`src/root_tests.rs`. Depends on clean T02 because all production files overlap.
Stop before every other file.

**Outcome:** move the exact `Size<Option<S>>` componentwise operation and
optional-minimum floor to sealed layout-math extensions; delete the local
duplicate traits and retain all call-site evaluation order.

**Characterization and acceptance:** new `fri08_c07_t03_optional_math_` tests
cover `None`, componentwise maximum, absent minimum, finite/non-finite error
identity, f32/f64, and direct algorithm results. They pass before and after with
no public surface.

**Focused commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib fri08_c07_t03_optional_math_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib layout_math
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib block_tests::
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib flex_tests::
```

**Intended commit:** `refactor(math): centralize optional size floors`.

### 4.4 `P01/I08/S01/C07/T04` Unify Grid Scrollbar Settlement

**Finding:** `SP-005`, medium-impact duplicated root/context settlement loops at
`src/grid/mod.rs:126-168` and `src/grid/mod.rs:520-556`.

**Write envelope:** exactly `src/grid/mod.rs` and `src/grid_tests.rs`. Depends on
clean T03 only for ordered attribution and is otherwise independent. Stop before
every other grid or test-support file.

**Outcome:** replace both loops with one private helper accepting explicit
initial state, parent context, and measurement boundary and returning
`GridComputeResult`. Root result projection remains outside the helper.

**Characterization and acceptance:** new `fri08_c07_t04_grid_settlement_` tests
cover caller-settled root versus contextual `INITIAL`, geometry/no-geometry
termination, exact error mapping, cache, both scalars, inherited contexts, and
scrollbar convergence. Iteration, publication, and error identity pass unchanged
before and after.

**Focused commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib fri08_c07_t04_grid_settlement_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib fri08_c02_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib fri08_c06_collapsed_gutter_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib fri08_c06r_inherited_placement_
```

**Intended commit:** `refactor(grid): unify scrollbar settlement`.

### 4.5 `P01/I08/S01/C07/T05` Share Scroll Geometry Test Fixtures

**Finding:** `SP-011`, low-impact exact duplicate test helpers at
`src/block_tests.rs:18-54`, `src/flex_tests.rs:5433-5469`, and duplicate
scroll-padding cases in block, flex, and leaf tests.

**Write envelope:** exactly new `src/test_support/scroll_geometry.rs`,
`src/test_support/mod.rs`, `src/lib_tests.rs`, `src/block_tests.rs`,
`src/flex_tests.rs`, and `src/leaf_tests.rs`. Depends on clean T02 because the
behavior-sensitive scroll characterization settles first. Stop before production
code, another test-support module, or another test file.

**Outcome:** move immutable generic scroll input construction and invariant
assertions to the new test-only module. Algorithm-specific runners, expected
geometry, and behavior assertions remain local.

**Characterization and acceptance:** new `fri08_c07_t05_scroll_fixture_` tests
prove f32/f64 selection, every `ComputeInputOf` field, site/operation/invariant
identity, Auto/value edge cases, and exact scroll-padding rows before extraction;
the same tests and original cases pass afterward. Source inventory records the
new owned Rust module. No production-visible surface or weakened assertion.

**Focused commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib fri08_c07_t05_scroll_fixture_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib block_tests::
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib flex_tests::
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib leaf_tests::
```

**Intended commit:** `refactor(test-support): share scroll fixtures`.

## 5 Final Verification And Completion

After all five exact task ranges have clean independent task reviews, run:

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri08_c07_
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

Also prove exact task scopes, unchanged generator/browser tree, frozen artifact
and input hashes, 5,776 comment-free XML files, exact report buckets, no new
`allow`/`expect`, zero unsafe across owned Rust, and a clean worktree. A fresh
holistic reviewer must return clean for the exact C07 cycle range.

Final executable scope, suppression, and unsafe proofs are:

```sh
expected_paths='plans/cycles/P01-I08-S01-C07-validated-sprawl-containment-and-final-candidate.md
plans/sequences/P01-I08-S01-grid-subgrid-and-grid-lanes-completeness.md
src/block.rs
src/block_tests.rs
src/compute.rs
src/flex.rs
src/flex_tests.rs
src/grid/mod.rs
src/grid_tests.rs
src/layout_math.rs
src/leaf_tests.rs
src/lib_tests.rs
src/root_tests.rs
src/scroll.rs
src/test_support/mod.rs
src/test_support/scroll_geometry.rs'
test "$(git diff --name-only dc71a5582ab0ef3925826dce09b93ee9fa6f49a1..HEAD | LC_ALL=C sort)" = "$(printf '%s\n' "$expected_paths" | LC_ALL=C sort)"
if git diff --word-diff=porcelain --word-diff-regex='[[:alpha:]_][[:alnum:]_]*' dc71a5582ab0ef3925826dce09b93ee9fa6f49a1..HEAD -- '*.rs' | rg '^\+.*\b(allow|expect)\b'; then exit 1; fi
unsafe_hits="$({ git ls-files -z -- '*.rs'; git ls-files -z --others --exclude-standard -- '*.rs'; } | sort -zu | xargs -0 rg -n '\bunsafe\b' | rg -v '^[^:]+:[0-9]+:[[:space:]]*unreachable!\("safe_fallback returns unsafe (item|content) alignment"\)$|^src/lib_tests\.rs:[0-9]+:[[:space:]]*Some\("async" \| "unsafe" \| "default" \| "extern"\) => keyword \+= 1,$|^src/lib_tests\.rs:[0-9]+:[[:space:]]*"removed phase-unsafe surface remains: \{removed\}"$|^tests/layout/browser_parity/support\.rs:[0-9]+:[[:space:]]*let has_overflow_prefix = safe \|\| raw\.starts_with\("unsafe "\);$|^tests/layout/browser_parity/support\.rs:[0-9]+:[[:space:]]*\.or_else\(\|\| raw\.strip_prefix\("unsafe "\)\)$|^tests/layout/browser_parity/support\.rs:[0-9]+:[[:space:]]*parse_align_content\("unsafe end"\)\.expect\("unsafe content alignment should parse"\),$|^tests/layout/browser_parity/support\.rs:[0-9]+:[[:space:]]*assert!\(parse_align_items\("unsafe first baseline"\)\.is_err\(\)\);$' || true)"
test -z "$unsafe_hits"
if { git ls-files -z -- '*.rs'; git ls-files -z --others --exclude-standard -- '*.rs'; } | sort -zu | xargs -0 rg -n -U '\b(no_mangle|export_name|link_section|naked)\b|(^|[^[:alnum:]_"])extern[[:space:]]*"'; then exit 1; fi
```

The final changed-file list must equal the explicit set in both directions; the
suppression scan and both unsafe scans must find no match.

C07 completes only after `SP-002`, `SP-003`, `SP-004`, `SP-005`, and `SP-011`
each have one implemented and characterized disposition; all eight FRI-08
behavior closures, 72 rows, FRI-09/F10 controls, public API removal, provenance,
and artifacts remain exact; the candidate is published and local `main`, its
tracking ref, and observed remote `main` are read back at one SHA. The handoff
then carries the untouched C08 rows and exact reviewed evidence into the
just-in-time C08 plan. No blocker is currently known.
