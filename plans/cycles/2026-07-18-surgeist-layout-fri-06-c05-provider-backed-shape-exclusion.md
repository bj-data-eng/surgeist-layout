# FRI-06-C05 Provider-Backed Shape Exclusion

Status: complete

Cycle ID: `FRI-06-C05`

Owning repository: `surgeist-layout`

Cycle base: `18032d13dd8bb204187ade7238505ca9210ffddd`

Reviewed specification:
`plans/specs/2026-07-17-surgeist-layout-fri-06-inline-formatting-floats-bfcs.md`
at normalized SHA-256
`98681bac979f68fa3bc380c349f7a04110f9a1f13d142625751fa9cbc5f1ffaf`,
commit `cc2a8486f9e4e7719c9a28cc68321b7e630d9ded`, decision `D-14` and the
shape-provider, error, band, cache, fake, and root-handoff portions of
`FRI-06.5` through `FRI-06.12`.

Reviewed sequence:
`plans/sequences/2026-07-17-surgeist-layout-fri-06-inline-formatting-floats-bfcs.md`
at normalized SHA-256
`20a17aaab4bd74566f6dbd0820da861ed17e9e6b9845debe06b1ee5177e8db0f`,
commit `0402af2c16ff53623fde19d7bc3e73c0475a25c8`, entry `FRI-06-C05`.

## Outcome

Consume the reviewed `LayoutTree` shape-exclusion provider through C04's one
flow-relative float ledger and band query. A requested shape supplies a bounded
physical interval for the exact overlapping candidate band; empty, partial,
full, and clipped results refine that band, while missing, invalid, and failed
results retain exact typed container/float/query diagnostics and publish no
partial layout state. Provider changes use the existing dirty-node transaction,
unit cache context, and successful batch replacement.

## Boundary

C04 is published and remotely verified at the cycle base. It supplies one
source-ordered rectangular ledger, complete-span finite transitions, unified
line bands, current BFC avoidance, float lifecycle, and provider-free `Shape`
physical collision behavior. C01 already supplies the public
`FloatExclusion::{MarginBox, Shape}` model, validated physical query and
interval carriers, defaulted `LayoutTree::float_exclusion_interval`, typed
role/provider errors, and invalidated root transaction. Production intentionally
does not yet invoke the provider and valid `Shape` still reports the staged later
capability.

This cycle owns only provider activation, query/result validation and physical
projection inside the existing ledger, exact query accounting, and provider
dirty/cache/rounding/failure behavior. It adds the reviewed private originating
query to `FloatExclusionIntervalOf` and the exhaustive
`FloatExclusionIntervalErrorOf::QueryMismatch { expected, actual }` state. It
may not add raw permissive output, shape identity, provider revision, cache-key
state, a sibling dependency, or a general geometry engine.

C06 owns parser/helper/comparator changes, HTML, fixtures, XML, reports,
provenance, and the one final full regeneration. C05 corrects only the public
crate rustdoc that currently describes provider invocation as deferred. No generator command or
architecture change, authored CSS, shape parsing or resolution, shape margin,
transform/reference-box policy, text/shaping change, README/example docs,
manifest, dependency, feature, lockfile, root, sibling, or generated artifact
enters C05. No new lint allowance or Surgeist-owned `unsafe` is permitted.

The validated mechanical opportunities remain recorded in
`plans/2026-07-18-surgeist-layout-mechanical-refactoring-review-findings.md`.
MR-001, MR-004, and MR-005 begin only after this cycle is published and remotely
verified; broader MR-002 and MR-003 remain after C07. None enters C05.

## Impacts

The provider method and query/interval constructors and accessors remain the C01
surface; `FloatExclusionIntervalOf` gains only private provenance. The new
exhaustive public `QueryMismatch` error variant is the reviewed intentional
pre-release breaking correction. Existing `LayoutTree` implementors keep the
default missing-provider method, and `MarginBox` layouts never call it. The unit
`CacheKeyContext`, dependencies, features, MSRV, lockfile, browser corpus, docs,
root handoff ownership, and generated artifacts remain unchanged except for the
required provider-activation crate rustdoc correction. Generator execution is
absent.

## Tasks

### `C05-T1` Activate The Typed Provider Front Door

**Files:** `src/node_input.rs`, `src/compute.rs`, `src/block.rs`, directly
required `src/inline.rs`, `src/traits.rs`, `src/lib.rs`, and focused
contract/lib/block/root Rust tests.

**Outcome:** Add the exact originating query as private interval provenance and
the named `QueryMismatch { expected, actual }` construction/provider-output
error. Allow a valid visible in-flow floating `Shape` role past root validation
and invoke the existing tree provider only from the canonical overlapping-band
query. Construct the exact finite ordered physical query from the final float
margin box, containing `FlowAxes`, and candidate block-axis span. Outer `None` maps to
`LayoutMissingContext::FloatExclusionProvider`; provider `Err(M)` maps to
`LayoutErrorKindOf::Measurement(M)`; an interval whose private actual query is
not the exact expected query maps to
`LayoutInvalidInputOf::FloatExclusionProviderOutput`. Every error uses
`LayoutOperation::FloatExclusionQuery` and
`LayoutErrorSiteOf::ContainerSubject { container, subject: float }`, with no
margin-box fallback or partial completed batch.

The invariant-bearing interval constructor remains the only raw-endpoint
boundary: non-finite and inverted endpoints fail there. Layout compares private
query provenance before consuming the already-clipped interval; it does not
reclip a mismatched value or create a second permissive provider carrier.
Update crate rustdoc from staged C01 non-invocation language to the active typed
provider/error/invalidation contract without adding general shape guidance.

**RED:** Add production-front-door tests prefixed
`fri06_c05_provider_error_` and `fri06_c05_provider_role_` first. At the exact
task base, a valid shape returns `LaterFriBehavior` without a query, while
missing/failed/mismatched providers cannot reach their reviewed
site/operation/kind.
Preserve reconstructible RED evidence; retain already-correct invalid-role and
constructor tests as characterization rather than false RED.

**Acceptance:** Both scalar lanes prove one valid shape reaches the provider;
hidden, non-floating, and absolute shapes fail before provider/cache activity;
margin-box and non-overlapping shape cases make zero provider calls; missing and
failed providers plus a valid interval constructed from a different valid query
retain exact typed diagnostics and leave all batch state absent. Raw invalid
endpoints fail only at construction. Static evidence removes only the staged
C01 non-invocation/later capability assertions and proves no provider fallback.
The public crate rustdoc names active provider invocation and no longer claims
that band refinement is deferred.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c05_provider_error_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c05_provider_role_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c01_float_exclusion_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout
CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
CARGO_NET_OFFLINE=true just fmt-check
```

**Dependency:** Reviewed plan at `in_progress`. **Intended commit:**
`feat(layout): activate shape exclusion provider`.

### `C05-T2` Refine Bands With Physical Shape Intervals

**Files:** `src/block.rs`, directly required `src/inline.rs`, and focused
block/inline/root provider tests.

**Outcome:** Replace a `Shape` float's rectangular inline interval with the
provider's interval for the exact candidate span. `Ok(None)` contributes no
exclusion; partial, full, and clipped intervals map from the physical inline
axis back to the containing logical start/end side in every flow. Same-side
entries choose the farthest inward result, opposing entries can close the band,
and next-transition ownership remains C04's finite margin-box block endpoint.
Float, line, and current BFC candidate purposes share the same provider-aware
ledger path; placement ownership and final-only publication do not move.

Each overlapping shape float is queried at most once for one float/band pair in
one candidate pass. A retry at a strictly later transition is a new band; a
same-cursor line reselection against the returned band reuses the first result.
Intrinsic or indefinite line-band computation never invokes the provider.

**RED:** Add front-door tests prefixed `fri06_c05_shape_band_`,
`fri06_c05_shape_flow_`, and `fri06_c05_shape_query_` first. At T2 base the
provider can be reached but empty/partial/full geometry, physical-axis reversal,
or same-pass accounting is incomplete. Preserve exact RED evidence.

**Acceptance:** Both scalar lanes prove empty, partial, full, disjoint/clipped,
zero-width, opposing, stacked, cleared, and overwide results through real block
lines and BFC/float consumers. All ten flows preserve mapped side identity;
query records contain the exact final physical margin box, axes, and ordered
band endpoints. One float/band pair is queried once per candidate pass, no
provider call occurs for margin-box, non-overlap, intrinsic, hidden, absolute,
or failed-prevalidation cases, and no second ledger/band table exists.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c05_shape_band_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c05_shape_flow_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c05_shape_query_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c04_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout
CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
CARGO_NET_OFFLINE=true just fmt-check
```

**Dependency:** T1 is task-clean. **Intended commit:**
`feat(layout): refine bands with shape intervals`.

### `C05-T3` Close Provider Invalidation And Lifecycle

**Files:** `src/compute.rs`, `src/block.rs`, directly required cache/contract
support only, and focused cache/transaction/root/provider Rust tests.

**Outcome:** Prove provider result or failure changes use
`compute_layout_invalidated` with the affected float as a dirty subject. The
existing root-to-dirty closure bypasses stale node hits; successful preparation
and commit replace node/fragment state and clear closure caches before stores.
Failed provider layout or immutable batch preparation publishes nothing and
retains the caller's dirty state. The unit cache context and cache-key fields do
not change.

Cold, warm, dirty, and rounded paths preserve the same final geometry and source
association. Warm valid output does not rerun provider queries; dirty provider
input reruns only the required layout pass; rounding never reruns the provider,
line selection, or band construction. Query records and transaction mutations
are observational test state, not production cache state.

**RED:** Add front-door tests prefixed `fri06_c05_provider_cache_`,
`fri06_c05_provider_dirty_`, and `fri06_c05_provider_atomicity_` first.
Reconstruct RED only for genuine missing lifecycle behavior at exact T3 base;
record already-correct generic transaction behavior honestly.

**Acceptance:** Both scalar lanes prove cold/warm equality, exact dirty closure,
provider empty-to-partial/full and success-to-failure transitions, stale-hit
bypass, rounded equality, bounded query counts, source order, scroll/baseline/
content stability, layout failure atomicity, and preparation failure atomicity.
Static evidence proves `CacheKeyContext` remains unit, no provider revision or
identity field exists, and no provider query occurs during rounding or intrinsic
computation.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c05_provider_cache_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c05_provider_dirty_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c05_provider_atomicity_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c05_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c04_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout
CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
CARGO_NET_OFFLINE=true just fmt-check
```

**Dependency:** T2 is task-clean. **Intended commit:**
`fix(layout): close shape provider lifecycle`.

## Completion

After all three task ranges are independently clean, make the plan's separate
status-only `complete` commit and set the immutable cycle head. Run:

```sh
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c05_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c04_
CARGO_NET_OFFLINE=true just fmt-check
git diff --check 18032d13dd8bb204187ade7238505ca9210ffddd..HEAD
git diff --name-only --no-renames 18032d13dd8bb204187ade7238505ca9210ffddd..HEAD
! git diff --name-only --no-renames 18032d13dd8bb204187ade7238505ca9210ffddd..HEAD | rg -q -v '^(plans/(specs/2026-07-17-surgeist-layout-fri-06-inline-formatting-floats-bfcs|sequences/2026-07-17-surgeist-layout-fri-06-inline-formatting-floats-bfcs|cycles/2026-07-18-surgeist-layout-fri-06-c05-provider-backed-shape-exclusion)\.md|src/(block|block_tests|cache_tests|compute|contract_tests|inline|inline_tests|lib|lib_tests|node_input|root_tests|traits)\.rs)$'
test -z "$(git ls-files --others --exclude-standard)"
! git diff --unified=0 18032d13dd8bb204187ade7238505ca9210ffddd..HEAD -- '*.rs' | rg -q '^\+.*#\s*\[\s*(allow|expect)\s*\('
! rg -n 'layout_inline_segments|provider_revision|shape_revision|float_exclusion_revision' src
! rg -n --pcre2 '#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{' $(git ls-files --cached --others --exclude-standard '*.rs')
test -z "$(git status --porcelain=v1)"
```

The changed-path allowlist is exactly the revised specification and sequence,
this plan, plus `src/block.rs`,
`src/block_tests.rs`, `src/cache_tests.rs`, `src/compute.rs`,
`src/contract_tests.rs`, `src/inline.rs`, `src/inline_tests.rs`,
`src/lib.rs`, `src/lib_tests.rs`, `src/node_input.rs`, `src/root_tests.rs`, and
`src/traits.rs`; allowed paths need not all change. Any other path fails
completion. The commands fail on a new `allow`/`expect`, legacy second band,
provider/cache revision state, nonignored untracked file, executable unsafe in
the complete tracked/nonignored owned Rust set, or dirty worktree. No temporary
manifest is created.

A fresh `surgeist-holistic-reviewer` must return `CLEAN` for exact range
`18032d13dd8bb204187ade7238505ca9210ffddd..cycle_head`. Prove local `main` is
that candidate, rerun the complete final command set without changing the head,
publish by fast-forward to the authority remote `main`, fetch/read back, and
prove local, tracking, `FETCH_HEAD`, and observed remote `main` agree. Remove
every cycle-owned temporary resource.

The handoff records the published C05 candidate, stable production/provider
facts for C06, and the exact post-C05 insertion point for MR-001, MR-004, and
MR-005 before any C06 plan or generator input change. No C06 implementation or
generator run begins before that containment window is resolved and published.
Blocker: none at planning time.
