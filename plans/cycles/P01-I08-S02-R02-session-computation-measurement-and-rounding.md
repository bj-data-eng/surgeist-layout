# P01-I08-S02-R02 Session, Computation, Measurement, And Rounding Owners

Cycle ID: `P01/I08/S02/R02`

Owning repository: `surgeist-layout`

Status: reviewed

Cycle base: `9154973a9f810e766918abde4399603d88fe2e12`

Reviewed specification:
`plans/specs/P01-I08-grid-subgrid-and-grid-lanes-completeness.md`, normalized
semantic-content SHA-256
`e3ac1af46ff7868e12e8df01da1cd4b46edd972638f34483fedc524b9d830595`,
commit `28d4016e7bf1005b8541868e8b1d251b0e03012c`: `FRI-08.20` row
`AR-002`, `FRI-08.21`, `FRI-08.22`, the `engine::validation`,
`engine::session`, `engine::root`, `engine::rounding`, `measurement`, and
`engine::mod` portions of `FRI-08.23`, the named R02 anchors in `FRI-08.27`,
and `FRI-08.28(1)` through `FRI-08.28(4)`.

Reviewed implementation sequence:
`plans/sequences/P01-I08-S02-architectural-remediation.md`, normalized
semantic-content SHA-256
`6d08a4c1e63a2cfd5ab858757bd6e614c852749ce93bf54d31409aa5687b7c59`,
commit `fcaf08b36149bc61f45d283759149ef8748401b8`, entry
`P01/I08/S02/R02`.

Bounded outcome: validation, invalidation, staged session state, dispatch, root
computation, rounding, public leaf measurement, and batch assembly each have
one responsibility-shaped owner; the public transaction and API are unchanged;
and no production `compute.rs` owner remains.

## 1 Boundary

The clean, published, remotely read-back R01 candidate is the immutable cycle
base. Public `compute_layout`, `compute_layout_invalidated`, `compute_leaf`,
leaf-measurement types and errors, `LayoutTree`, error types, and root reexports
remain source-compatible. Validation order, topology and role diagnostics,
cache identity, invalidation closure and ordering, dispatch, hidden-state
reset, measurement passes, typed failures, source-ordered staging, rounding,
and atomic completed-batch assembly remain observable controls.

This cycle creates only `src/engine/validation.rs`,
`src/engine/session.rs`, `src/engine/root.rs`,
`src/engine/rounding.rs`, and `src/measurement.rs` as new production source
classes. It updates direct imports/callers, `engine/mod.rs`, `lib.rs`, the
production-source inventory, and focused tests required by the five tasks. It
removes `src/compute.rs` only after every production and test owner has moved.
The existing `src/compute_tests.rs` companion may retain its filename until
R07 test partitioning, but it must consume the new owners or crate-root facade
and cannot preserve a production `compute` module.

The algorithm-neutral result-transposition adapters `SizeResultExt` and
`EdgesResultExt`, plus their fallible length/auto resolution helpers, move to
the existing private `sizing::resolve` owner because they adapt the validated
sizing/value model and are direct consumers of its error. The inherent
`OptimalRegionInsetsOf::from_scroll_padding` constructor moves beside its
scroll type in `scroll.rs`; R03 later partitions that already-owned scroll
surface. These moves close otherwise ownerless residue and do not start R03.

Out of scope: behavior or public API changes; scroll construction or physical
decomposition beyond that one inherent-constructor relocation; block/flex/grid
physical decomposition; node projections; companion-test partitioning;
README/API map; dependencies, features, lockfile, MSRV, root/sibling work,
FRI-09 implementation, generator architecture, helper, authored HTML, manifest,
XML, report, browser execution, generation, acquisition, suppression, and unsafe
code.

Resolved ownership at cycle exit:

- `engine::validation` owns root/tree validation and invalidation closure;
- `measurement` owns the public leaf input, availability, standalone error, and
  fallible leaf computation model;
- `engine::root` owns hidden, ordinary-root, flex-item-root, and root-scroll
  computation;
- `engine::rounding` owns source-ordered node/fragment rounding and rounded
  scroll reconstruction;
- `engine::session` owns staged state, cache access, dispatch, fragment staging,
  and completed-batch assembly;
- `engine::mod` owns public session entry orchestration and private composition;
- `sizing::resolve` owns generic result-transposition/value-resolution adapters;
  and
- `lib.rs` remains the sole unchanged public facade.

Frozen artifact state remains: corpus manifest
`c6e6f1422e14a5e4aa474c143998063ce0de4d0a9123b69875b35a4ed009a8f6`,
helper `c684c7f167d95997a4a9f0250467bbaf72c1b73e69e0f707a2ef32f4d25f7f36`,
`all.json` `c10dc550d260a239c8bf9dd553f5272ca3bcc2826099bc182f800986b8b94c0e`,
1,448 HTML, and 5,776 comment-free XML.

## 2 Impacts

Public API classification: internal-only relocation; exact public names,
signatures, defaults, traits, error payloads, documentation, and root reexports
remain unchanged. Dependencies, features, lockfile, MSRV, generated artifacts,
fixtures, root integration, and browser state remain unchanged. All new engine
modules are private. Surgeist-owned code remains free of unsafe and no lint
suppression is added.

Each behavior-preserving task first runs its named existing characterizations on
the assignment base. New `fri08_remediation_*` structural anchors are expected
to fail on that base while the former owner remains; they supplement rather than
replace passing behavioral characterization. New anchors are appended after the
four existing quoted suppression fixtures so the exact fixture inventory does
not drift during the cycle.

## 3 Tasks

### 3.1 `P01/I08/S02/R02/T01` Extract Validation And Invalidation

**Files/area:** `src/engine/validation.rs`, `src/engine/mod.rs`,
`src/compute.rs`, `src/lib_tests.rs`, `src/invalidation_transaction_tests.rs`,
`src/contract_tests.rs`, and exact root/role validation tests.

**Outcome:** move the complete invalidation closure, topology traversal, root
request/tree validation, node-role validation, and exact validation-error
mapping to `engine::validation`. Orchestration calls that owner before creating
a session. Validation order and all source-ordered inclusive closure semantics
remain exact.

**Characterization/RED:** on the task base run `fri06_c01_invalidation_`,
`fri06_c01_non_box_`, `fri06_c05_provider_role_invalid_`, and
`root_request_rejects_invalid_definite_availability`. Add
`fri08_remediation_engine_validation_has_one_owner`; it fails structurally
while validation/invalidation declarations remain in `compute.rs`.

**Acceptance:** every characterization passes unchanged; the anchor proves one
validation owner, orchestration-before-session order, no duplicate in
`compute.rs`, and complete source inventory.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri06_c01_invalidation_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri06_c01_non_box_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri06_c05_provider_role_invalid_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout root_request_rejects_invalid_definite_availability
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri08_remediation_engine_validation_has_one_owner
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout
CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
cargo fmt --check
git diff --check
```

**Dependency:** none. Intended commit: `refactor(engine): extract validation`.

### 3.2 `P01/I08/S02/R02/T02` Extract Leaf Measurement

**Files/area:** `src/measurement.rs`, `src/compute.rs`, `src/error.rs`,
`src/tree.rs`, `src/lib.rs`, `src/sizing/resolve.rs`, direct block/flex/grid
result-adapter imports, the single inherent constructor in `src/scroll.rs`,
`src/lib_tests.rs`, `src/compute_tests.rs`, `src/leaf_tests.rs`, and exact leaf
and measurement tests.

**Outcome:** move `LeafMeasureInputOf`, `MeasurementAvailableOf`,
`LeafMeasureErrorOf`, their aliases/accessors, resolved leaf carriers, public
`compute_leaf`, tree-backed leaf measurement, pass settlement, output
validation, and exact error mapping to `measurement`. Move generic sizing-result
adapters to `sizing::resolve` and the scroll-padding inherent constructor beside
its scroll owner. `tree` imports the measurement input from its final owner;
`lib.rs` preserves the exact root facade.

**Characterization/RED:** on the task base run `fri04_c03_`,
`fri04_c04_leaf_block_positioned_`, `public_leaf_invalid_numeric_affine_`,
`fri05_c03_leaf_`, and `compute_layout_rejects_invalid_provider_output_without_batch`.
Add `fri08_remediation_measurement_has_one_owner`; it fails structurally while
the public model or leaf algorithm remains in `compute.rs` or the standalone
measurement error remains in `error.rs`.

**Acceptance:** exact measurement inputs, pass counts, geometry, scalar lanes,
provider/non-finite failures, operation/site mapping, and public paths remain
unchanged; the anchor proves one measurement owner and no parallel leaf path;
generic sizing adapters have one private owner.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri04_c03_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri04_c04_leaf_block_positioned_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout public_leaf_invalid_numeric_affine_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri05_c03_leaf_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout compute_layout_rejects_invalid_provider_output_without_batch
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri08_remediation_measurement_has_one_owner
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout
CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
cargo fmt --check
git diff --check
```

**Dependency:** T01. Intended commit: `refactor(measurement): extract leaf engine`.

### 3.3 `P01/I08/S02/R02/T03` Extract Root Computation

**Files/area:** `src/engine/root.rs`, `src/engine/mod.rs`,
`src/compute.rs`, `src/lib.rs`, direct test-only root imports,
`src/lib_tests.rs`, `src/root_tests.rs`, and exact root tests.

**Outcome:** move hidden computation, ordinary viewport root, flex-item root,
root edge and known-inline resolution, physical root coordinates, and root
scroll geometry/error mapping to `engine::root`. Session dispatch and public
orchestration consume this owner; test-only crate-root access remains exact.

**Characterization/RED:** on the task base run
`fri08_c07_t02_scroll_source_root_`, `fri05_c03_root_`,
`fri06_c03_lifecycle_`, and the `compute_layout_` tests. Add
`fri08_remediation_engine_root_has_one_owner`; it fails structurally while root
or hidden computation remains in `compute.rs`.

**Acceptance:** root context, parent basis, hidden reset, cache behavior,
canonical root scroll geometry, diagnostics, and atomic batch behavior remain
unchanged; the anchor proves one root owner and no root algorithm in session.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri08_c07_t02_scroll_source_root_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri05_c03_root_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri06_c03_lifecycle_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout compute_layout_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri08_remediation_engine_root_has_one_owner
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout
CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
cargo fmt --check
git diff --check
```

**Dependency:** T02. Intended commit: `refactor(engine): extract root computation`.

### 3.4 `P01/I08/S02/R02/T04` Extract Session State And Services

**Files/area:** `src/engine/session.rs`, `src/engine/mod.rs`,
`src/compute.rs`, direct production/test imports, `src/lib_tests.rs`, and exact
cache/root/transaction tests.

**Outcome:** move the complete `ComputeSession` and
`StagedInlineFragmentGroup` state, construction, source-ordered completion,
completed-batch assembly, staged node/fragment mutation, warm inline replay,
subtree restoration, algorithm dispatch, hidden dispatch, tree-backed
measurement handoff, and the `Traverse`, `Compute`, `CacheAccess`, and `Round`
service implementations to `engine::session`. Move its staging helpers and the
test-only hidden-request trace with it. `compute.rs` temporarily retains only
public entry orchestration and the rounding algorithm after this task; it may
construct and complete the crate-private session through narrow
`pub(super)` entry methods, but cannot access its fields.

**Characterization/RED:** on the task base run `fri06_c02_cache_`,
`fri06_c03_lifecycle_`, `fri06_c01_batch_transaction_`,
`fri06_c05_provider_atomicity_`, and the `compute_layout_` tests. Add
`fri08_remediation_engine_session_transaction_equivalence`; it fails
structurally while session state, service implementations, dispatch, or batch
assembly remains in `compute.rs`.

**Acceptance:** cold/warm/dirty cache behavior, provider behavior, dispatch,
hidden state, source order, invalidation replacement, error rollback, and batch
atomicity remain exact; the named anchor proves a single session owner and that
`compute.rs` has no staged field, service implementation, dispatch, or batch
assembly. The rounding algorithm remains unchanged and callable through the
neutral `Round` service.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri06_c02_cache_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri06_c03_lifecycle_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri06_c01_batch_transaction_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri06_c05_provider_atomicity_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout compute_layout_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri08_remediation_engine_session_transaction_equivalence
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout
CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
cargo fmt --check
git diff --check
```

**Dependency:** T03. Intended commit: `refactor(engine): extract session state`.

### 3.5 `P01/I08/S02/R02/T05` Extract Rounding

**Files/area:** `src/engine/rounding.rs`, `src/engine/mod.rs`,
`src/compute.rs`, `src/lib.rs`, direct test-only rounding imports,
`src/lib_tests.rs`, and exact cache/root rounding tests.

**Outcome:** move recursive source-ordered node rounding, fragment rounding,
cumulative coordinates, typed overflow failures, and rounded canonical-scroll
reconstruction to `engine::rounding`. The `Round` service implementation stays
with session state; the rounding algorithm resides only here and consumes that
neutral service.

**Characterization/RED:** on the task base run `fri06_c02_rounding_`,
`fri06_mr02_layout_round_`,
`fri05_c04_flex_round_cache_publication_has_one_canonical_geometry_path`, and
`fri05_c05_grid_round_cache_has_no_independent_scrollbar_projection`. Add
`fri08_remediation_engine_rounding_has_one_owner`; it fails structurally while
the rounding algorithm remains in `compute.rs`.

**Acceptance:** exact traversal order, cumulative rounding, fragments,
baselines, scroll reconstruction, scalar lanes, failure atomicity, and test-only
crate-root access remain unchanged; the anchor proves one rounding owner and no
rounding algorithm in session or `compute.rs`.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri06_c02_rounding_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri06_mr02_layout_round_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri05_c04_flex_round_cache_publication_has_one_canonical_geometry_path
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri05_c05_grid_round_cache_has_no_independent_scrollbar_projection
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri08_remediation_engine_rounding_has_one_owner
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout
CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
cargo fmt --check
git diff --check
```

**Dependency:** T04. Intended commit: `refactor(engine): extract rounding`.

### 3.6 `P01/I08/S02/R02/T06` Extract Public Orchestration And Remove Compute Owner

**Files/area:** `src/engine/mod.rs`, `src/compute.rs`, `src/lib.rs`, direct
production/test imports, `src/lib_tests.rs`, `src/compute_tests.rs`, exact
root/transaction tests, and production-source inventory.

**Outcome:** move public `compute_layout` and `compute_layout_invalidated`
composition to `engine::mod`, using validation, session, root, and rounding only
through their narrow owned entry points. Relocate the remaining internal
`compute.rs` unit tests to their semantic engine/measurement owners. Remove
production `compute.rs` and every `crate::compute` reference without changing
the public facade. Replace the pre-existing `compute_layout`
`clippy::type_complexity` expectation with a private result alias at its new
owner; do not relocate or add a suppression.

**Characterization/RED:** on the task base rerun `fri06_c02_cache_`,
`fri06_c03_lifecycle_`, `fri06_c01_batch_transaction_`, and the
`compute_layout_` tests. Extend
`fri08_remediation_engine_session_transaction_equivalence` and
`fri08_remediation_public_api_inventory_is_compatible`; they fail structurally
while public orchestration or production `compute.rs` remains.

**Acceptance:** validation-before-session order, public entry behavior,
dispatch, rounding publication, rollback, and batch atomicity remain exact; the
two anchors prove the complete final ownership graph, absent production
`compute.rs`, complete source inventory, and unchanged public root API.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri06_c02_cache_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri06_c03_lifecycle_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri06_c01_batch_transaction_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout compute_layout_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri08_remediation_engine_session_transaction_equivalence
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri08_remediation_public_api_inventory_is_compatible
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout
CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
cargo fmt --check
git diff --check
```

**Dependency:** T05. Intended commit: `refactor(engine): extract public orchestration`.

## 4 Completion

R02 is accepted only when all six ordered task ranges have independent CLEAN
task reviews; the cycle plan is status-complete; final checks and one holistic
review are CLEAN; `AR-002` has its exact disposition; production `compute.rs`
and `traits.rs` are absent; every `FRI-08.23` engine/measurement responsibility
has one owner; public API and transaction semantics, features/dependencies/MSRV,
unsafe/suppression, and frozen artifacts remain exact; and the cycle is
published and remotely read back.

Final commands:

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri08_remediation_
CARGO_NET_OFFLINE=true just verify

# `cargo clean` intentionally removed the leaf source cache at the preceding
# cycle boundary. Reuse only the already-present exact local pinned checkout;
# never run the acquisition-capable importer. If this cache is absent or its
# revision differs, stop and report the generator/Taffy gate blocked.
source_cache=/Users/codex/Development/surgeist/crates/surgeist-layout/target/surgeist-sources/taffy/d1ff7e339b9ee35b33858779f8d7653197e93d92
destination_parent=/Users/codex/Development/surgeist-layout/target/surgeist-sources/taffy
destination_cache="$destination_parent/d1ff7e339b9ee35b33858779f8d7653197e93d92"
test -d "$source_cache"
test "$(git -C "$source_cache" rev-parse HEAD)" = d1ff7e339b9ee35b33858779f8d7653197e93d92
test ! -e "$destination_cache"
mkdir -p "$destination_parent"
cp -R "$source_cache" "$destination_cache"
test "$(git -C "$destination_cache" rev-parse HEAD)" = d1ff7e339b9ee35b33858779f8d7653197e93d92

CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --features layout-golden-generate --all-targets -- -F unsafe-code -D warnings
cargo fmt --check
git diff --check

expected_paths="$({ printf '%s\n' 'plans/cycles/P01-I08-S02-R02-session-computation-measurement-and-rounding.md'; while IFS= read -r span; do git diff --name-only "$span"; done <<< "$TASK_SPANS"; } | LC_ALL=C sort -u)"
actual_paths="$(git diff --name-only 9154973a9f810e766918abde4399603d88fe2e12..HEAD | LC_ALL=C sort -u)"
test "$actual_paths" = "$expected_paths"

if git diff --word-diff=porcelain --word-diff-regex='[[:alpha:]_][[:alnum:]_]*' 9154973a9f810e766918abde4399603d88fe2e12..HEAD -- '*.rs' | rg '^\+.*\b(allow|expect)\b'; then exit 1; fi
normalized_allow_inventory="$({ git ls-files -z -- '*.rs'; git ls-files -z --others --exclude-standard -- '*.rs'; } | sort -zu | xargs -0 rg -n -U --pcre2 '#\s*\[\s*allow\b' | sed -E 's/^([^:]+):[0-9]+:/\1:/' | LC_ALL=C sort || true)"
expected_allow_inventory="$(printf '%s\n' \
  'src/contract_tests.rs:        !text_source.contains("#[allow(dead_code)]"),' \
  'src/lib_tests.rs:        "#[allow(clippy::too_many_arguments)]",' \
  'src/lib_tests.rs:        "#[allow(dead_code)] /* between attributes */ #[cfg_attr(not(test), cfg(test))] pub(crate) fn hidden() { scrollbar_size; }",' \
  'src/lib_tests.rs:        "#[allow(dead_code)]",' | LC_ALL=C sort)"
test "$normalized_allow_inventory" = "$expected_allow_inventory"

unsafe_hits="$({ git ls-files -z -- '*.rs'; git ls-files -z --others --exclude-standard -- '*.rs'; } | sort -zu | xargs -0 rg -n '\bunsafe\b' | rg -v 'safe_fallback returns unsafe|Some\("async" \| "unsafe" \| "default" \| "extern"\)|removed phase-unsafe surface remains|starts_with\("unsafe "\)|strip_prefix\("unsafe "\)|parse_align_content\("unsafe end"\)|parse_align_items\("unsafe first baseline"\)' || true)"
test -z "$unsafe_hits"
if { git ls-files -z -- '*.rs'; git ls-files -z --others --exclude-standard -- '*.rs'; } | sort -zu | xargs -0 rg -n -U '\b(no_mangle|export_name|link_section|naked)\b|(^|[^[:alnum:]_\"])extern[[:space:]]*\"'; then exit 1; fi

test "$(shasum -a 256 tests/layout/browser_parity/corpus.toml | awk '{print $1}')" = c6e6f1422e14a5e4aa474c143998063ce0de4d0a9123b69875b35a4ed009a8f6
test "$(shasum -a 256 tests/layout/browser_parity/scripts/gentest/test_helper.js | awk '{print $1}')" = c684c7f167d95997a4a9f0250467bbaf72c1b73e69e0f707a2ef32f4d25f7f36
test "$(shasum -a 256 tests/layout/browser_parity/xml/generation-reports/all.json | awk '{print $1}')" = c10dc550d260a239c8bf9dd553f5272ca3bcc2826099bc182f800986b8b94c0e
test "$(find tests/layout/browser_parity/html -type f -name '*.html' | wc -l | tr -d ' ')" = 1448
test "$(find tests/layout/browser_parity/xml -type f -name '*.xml' | wc -l | tr -d ' ')" = 5776

test -z "$(git status --porcelain=v1)"
```

`TASK_SPANS` is the newline-delimited ordered list of all task implementation and
fix spans accepted by task reviews. Public API and production-source inventories
are enforced by the remediation anchors and full library tests.

After publication and remote readback, run the following from the repository
root before R03 planning:

```sh
stale_processes="$(ps -axo pid=,comm=,args= | awk '$2 ~ /^(cargo|rustc|surgeist-layout-generate|surgeist_layout-)/ && $0 ~ /surgeist-layout|surgeist_layout/ { print }')"
test -z "$stale_processes"
cargo clean
test ! -e target
test -z "$(git status --porcelain=v1)"
```

Browser execution, generation, import/acquisition, and tracked artifact mutation
remain prohibited throughout the cycle.

Required R03 handoff: after CLEAN holistic review, exact publication, remote
readback, process hygiene, successful `cargo clean`, and proof that `target/` is
absent, record the immutable published candidate SHA; the reviewed
specification, sequence, and R02 plan revisions; the six ordered accepted task
and any fix ranges with review verdicts; final command, public compatibility,
dependency/feature/MSRV, suppression/unsafe, and frozen-artifact evidence; the
remote-readback result; and cleanup proof. R03 is ready only from that exact
candidate. Blocker disposition: no implementation or external blocker is known
at planning time; if the already-present pinned Taffy cache is absent or wrong,
or if any task requires behavior/API/artifact/generator scope expansion, stop and
report the exact blocker rather than acquiring or widening scope.
