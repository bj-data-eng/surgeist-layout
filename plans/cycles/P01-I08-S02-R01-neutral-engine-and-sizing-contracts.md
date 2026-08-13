# P01-I08-S02-R01 Neutral Engine And Sizing Contracts

Cycle ID: `P01/I08/S02/R01`

Owning repository: `surgeist-layout`

Status: in_progress

Cycle base: `fcaf08b36149bc61f45d283759149ef8748401b8`

Reviewed specification:
`plans/specs/P01-I08-grid-subgrid-and-grid-lanes-completeness.md`, normalized
semantic-content SHA-256
`e3ac1af46ff7868e12e8df01da1cd4b46edd972638f34483fedc524b9d830595`,
commit `28d4016e7bf1005b8541868e8b1d251b0e03012c`: `FRI-08.20` rows
`AR-001` and `AR-003`, `FRI-08.21`, `FRI-08.22`, the `error`, `tree`,
`engine::contracts`, and `sizing::resolve` portions of `FRI-08.23`, the named
R01 anchors in `FRI-08.27`, and `FRI-08.28(1)` through `FRI-08.28(4)`.

Reviewed implementation sequence:
`plans/sequences/P01-I08-S02-architectural-remediation.md`, normalized
semantic-content SHA-256
`6d08a4c1e63a2cfd5ab858757bd6e614c852749ce93bf54d31409aa5687b7c59`,
commit `fcaf08b36149bc61f45d283759149ef8748401b8`, entry
`P01/I08/S02/R01`.

Bounded outcome: public layout errors and host contracts, private recursive
engine services, and shared sizing resolution have separate algorithm-neutral
owners; block owns inherited-float recursion; public behavior and surface remain
unchanged.

## 1 Boundary

The clean published cycle base is the immutable R01 entry. Existing public
`compute_layout`, `compute_layout_invalidated`, `compute_leaf`, error types,
`Traverse`, `LayoutTree`, `LayoutBatchSink`, `NodeInputOf`, and all root
reexports remain source-compatible. Error variants, sites, operations, provider
sources, cache behavior, measurement, rounding, and batch atomicity remain
observable controls.

This cycle creates only `src/error.rs`, `src/tree.rs`,
`src/engine/contracts.rs`, `src/engine/mod.rs`, and `src/sizing/resolve.rs` as
new production source classes. It may update direct imports/callers, `lib.rs`,
the production-source inventory, and focused tests required by the three tasks.
It removes `src/traits.rs` only after all of its public and private owners have
moved. `compute.rs` remains the temporary session/dispatch/measurement owner for
R02.

Out of scope: algorithm behavior; public names, paths, fields, signatures,
variants, defaults, or reexports; session decomposition beyond imports and the
neutral trait implementation; scroll/block/flex/grid physical decomposition;
node projections; companion-test partitioning; README/API map; dependencies,
features, lockfile, MSRV, root/sibling work, FRI-09 implementation, generator,
helper, HTML, manifest, XML, report, browser execution, generation, acquisition,
suppression, and unsafe code.

Resolved ownership:

- `error` owns the public error model and error conversion helpers;
- `tree` owns the public host contracts;
- `engine::contracts` owns only neutral recursive, cache-write, and rounding
  services;
- block calls its own inherited-float compute entry through the neutral service;
- `sizing::resolve` owns all shared preferred/minimum/maximum/flex-basis
  resolution and its algorithm-neutral intermediate states; and
- `lib.rs` remains the sole unchanged public facade.

Frozen artifact state remains: corpus manifest
`c6e6f1422e14a5e4aa474c143998063ce0de4d0a9123b69875b35a4ed009a8f6`,
helper `c684c7f167d95997a4a9f0250467bbaf72c1b73e69e0f707a2ef32f4d25f7f36`,
`all.json` `c10dc550d260a239c8bf9dd553f5272ca3bcc2826099bc182f800986b8b94c0e`,
1,448 HTML, and 5,776 comment-free XML.

## 2 Impacts

Public API classification: internal-only; exact public source surface and
compile behavior unchanged. Dependencies, features, lockfile, MSRV, generated
artifacts, fixtures, root integration, and browser state are unchanged. No docs
or examples change. All new modules are private. Surgeist-owned code remains
free of unsafe and no lint suppression is added.

Behavior-preserving tasks first run their named existing characterizations on
the assignment base. New `fri08_remediation_*` structural anchors may fail on
the base because the old owner still exists; they supplement rather than replace
the passing behavioral characterization.

## 3 Tasks

### 3.1 `P01/I08/S02/R01/T01` Extract Public Error Ownership

**Files/area:** `src/error.rs`, `src/compute.rs`, `src/lib.rs`, direct production
error imports, `src/lib_tests.rs`, `src/contract_tests.rs`, and exact focused
error tests.

**Outcome:** move the complete public layout result/error model and shared
geometry/value/sizing error conversion to `error`, preserving every public name,
payload, getter, source, site, operation, and root reexport. `compute` imports
that owner and defines no public error type.

**Characterization/RED:** before mutation, run the public error/provider,
invalid-input, unsupported-sizing, measurement, and transaction-focused tests.
Add the source/API portion of
`fri08_remediation_public_api_inventory_is_compatible`; its structural owner
assertion fails while public errors remain declared in `compute.rs`.

**Acceptance:** the same behavioral tests pass after the move; the new anchor
proves one error owner and exact public reexports; source inventory includes the
new production file; no behavior, public API, artifact, or unrelated file delta
exists.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri06_c05_provider_error_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri04_c04_leaf_block_positioned_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri06_c02_transaction_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout public_leaf_invalid_numeric_affine_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout root_request_rejects_invalid_definite_availability
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout leaf_measurement_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout compute_layout_rejects_invalid_provider_output_without_batch
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri08_remediation_public_api_inventory_is_compatible
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout
CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
cargo fmt --check
git diff --check
```

**Dependency:** none. Intended commit: `refactor(engine): extract layout errors`.

### 3.2 `P01/I08/S02/R01/T02` Neutralize Recursive Engine Contracts

**Files/area:** `src/tree.rs`, `src/engine/mod.rs`,
`src/engine/contracts.rs`, `src/traits.rs`, `src/block.rs`, `src/compute.rs`,
`src/lib.rs`, `src/lib_tests.rs`, `src/contract_tests.rs`, and exact block/root
characterizations.

**Outcome:** move public host traits to `tree`, private recursive/cache/rounding
traits to `engine::contracts`, remove the block-specific method from the shared
recursive trait, and make the ordinary block child call its block-owned
inherited-float entry through neutral recursion. Remove `traits.rs` after all
owners and tests move.

**Characterization/RED:** on the task base, run shape-provider, inherited-float,
ordinary BFC, cold/warm cache, dirty replacement, and transactional publication
characterizations. Add
`fri08_remediation_engine_contract_is_algorithm_neutral`; it fails structurally
while the shared trait names `InheritedFloatExclusions` or dispatches block.

**Acceptance:** all characterizations remain byte-for-byte behavior-equivalent;
the new anchor proves the shared contract contains no algorithm-local type or
dispatch; public trait paths/signatures and test implementations compile
unchanged; no production `traits.rs` owner remains.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri06_c05_provider_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri06_c04_line_band_ordinary_block_keeps_outer_edge_and_inherits_parent_float_both_scalars
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri06_c04_bfc_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri06_c02_cache_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri06_c03_lifecycle_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri08_remediation_engine_contract_is_algorithm_neutral
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout
CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
cargo fmt --check
git diff --check
```

**Dependency:** T01. Intended commit: `refactor(engine): neutralize recursive contracts`.

### 3.3 `P01/I08/S02/R01/T03` Centralize Sizing Resolution

**Files/area:** `src/sizing.rs`, `src/sizing/resolve.rs`, `src/compute.rs`, direct
block/flex/grid sizing consumers, `src/lib_tests.rs`, and exact sizing tests.

**Outcome:** move the shared sizing-resolution error, resolved preferred/flex-
basis carriers, basis selection, and preferred/minimum/maximum/flex-basis
resolution functions to `sizing::resolve`; algorithms consume it directly and
`compute` retains no parallel resolver.

**Characterization/RED:** on the task base, run nested sizing, missing-basis,
unsupported-behavior, property-field, flex intrinsic-basis, and grid track-basis
characterizations. Add
`fri08_remediation_sizing_resolution_has_one_owner`; it fails structurally while
the resolver declarations remain in `compute.rs`.

**Acceptance:** exact resolved geometry and typed failures remain unchanged in
both scalar lanes; the new anchor proves one sizing owner and no algorithm-local
duplicate; `sizing::resolve` remains private; public sizing types/reexports are
unchanged.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri04_c03_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri04_c04_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout property_fields_preserve_layout_sizing_semantics
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout track_sizing_definite_uses_shared_optional_basis_resolution
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri08_remediation_sizing_resolution_has_one_owner
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout
CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
cargo fmt --check
git diff --check
```

**Dependency:** T02. Intended commit: `refactor(sizing): centralize resolution`.

## 4 Completion

R01 is accepted only when all three ordered task ranges have independent CLEAN
task reviews; the cycle plan is status-complete; final checks and one holistic
review are CLEAN; `AR-001` and `AR-003` satisfy their exact outcomes; public API,
behavior, features/dependencies/MSRV, unsafe/suppression, and frozen artifacts
remain exact; and the cycle is published and remotely read back.

Final commands:

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri08_remediation_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --features layout-golden-generate --all-targets -- -F unsafe-code -D warnings
cargo fmt --check
git diff --check

expected_paths="$({ printf '%s\n' 'plans/cycles/P01-I08-S02-R01-neutral-engine-and-sizing-contracts.md'; while IFS= read -r span; do git diff --name-only "$span"; done <<< "$TASK_SPANS"; } | LC_ALL=C sort -u)"
actual_paths="$(git diff --name-only fcaf08b36149bc61f45d283759149ef8748401b8..HEAD | LC_ALL=C sort -u)"
test "$actual_paths" = "$expected_paths"

if git diff --word-diff=porcelain --word-diff-regex='[[:alpha:]_][[:alnum:]_]*' fcaf08b36149bc61f45d283759149ef8748401b8..HEAD -- '*.rs' | rg '^\+.*\b(allow|expect)\b'; then exit 1; fi
allow_hits="$({ git ls-files -z -- '*.rs'; git ls-files -z --others --exclude-standard -- '*.rs'; } | sort -zu | xargs -0 rg -n -U --pcre2 '#\s*\[\s*allow\b' | rg -v '^src/contract_tests\.rs:65:|^src/lib_tests\.rs:984:|^src/lib_tests\.rs:985:|^src/lib_tests\.rs:2180:' || true)"
test -z "$allow_hits"

unsafe_hits="$({ git ls-files -z -- '*.rs'; git ls-files -z --others --exclude-standard -- '*.rs'; } | sort -zu | xargs -0 rg -n '\bunsafe\b' | rg -v 'safe_fallback returns unsafe|Some\("async" \| "unsafe" \| "default" \| "extern"\)|removed phase-unsafe surface remains|starts_with\("unsafe "\)|strip_prefix\("unsafe "\)|parse_align_content\("unsafe end"\)|parse_align_items\("unsafe first baseline"\)' || true)"
test -z "$unsafe_hits"
if { git ls-files -z -- '*.rs'; git ls-files -z --others --exclude-standard -- '*.rs'; } | sort -zu | xargs -0 rg -n -U '\b(no_mangle|export_name|link_section|naked)\b|(^|[^[:alnum:]_"])extern[[:space:]]*"'; then exit 1; fi

test "$(shasum -a 256 tests/layout/browser_parity/corpus.toml | awk '{print $1}')" = c6e6f1422e14a5e4aa474c143998063ce0de4d0a9123b69875b35a4ed009a8f6
test "$(shasum -a 256 tests/layout/browser_parity/scripts/gentest/test_helper.js | awk '{print $1}')" = c684c7f167d95997a4a9f0250467bbaf72c1b73e69e0f707a2ef32f4d25f7f36
test "$(shasum -a 256 tests/layout/browser_parity/xml/generation-reports/all.json | awk '{print $1}')" = c10dc550d260a239c8bf9dd553f5272ca3bcc2826099bc182f800986b8b94c0e
test "$(find tests/layout/browser_parity/html -type f -name '*.html' | wc -l | tr -d ' ')" = 1448
test "$(find tests/layout/browser_parity/xml -type f -name '*.xml' | wc -l | tr -d ' ')" = 5776

test -z "$(git status --porcelain=v1)"
```

`TASK_SPANS` is the newline-delimited ordered list of all task implementation and
fix spans accepted by task reviews. The public API and production-source
inventories are enforced by the remediation anchors and full library tests.

After publication and remote readback, run the following from the repository
root before R02 planning:

```sh
stale_processes="$(ps -axo pid=,comm=,args= | awk '$2 ~ /^(cargo|rustc|surgeist-layout-generate|surgeist_layout-)/ && $0 ~ /surgeist-layout|surgeist_layout/ { print }')"
test -z "$stale_processes"
cargo clean
test ! -e target
test -z "$(git status --porcelain=v1)"
```

Required handoff: immutable R01 candidate with exact reviewed planning
revisions, task ranges, command evidence, review verdicts, compatibility and
artifact results, publication/readback SHA, cleanup evidence, and R02 readiness.

No blocker is currently known. A required public/API or behavior change,
generator/artifact change, unsafe, new dependency/feature, or edit outside the
specified responsibility boundary returns a blocker or planning revision rather
than widening this cycle.
