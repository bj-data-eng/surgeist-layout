# FRI-06-C01 Public Inline Model And Transaction Substrate

Status: in_progress

Cycle ID: `FRI-06-C01`

Owning repository: `surgeist-layout`

Cycle base: `24bb3ccd0a4c9f54bc9eaa7958a9d2ea740bf859`

Reviewed specification:
`plans/specs/2026-07-17-surgeist-layout-fri-06-inline-formatting-floats-bfcs.md`
at normalized SHA-256
`7cb09e0a8e9036a406b39115ed8f6392df805116a762905a3510c7fe7355f970`,
commit `64a9ca96be3b29765b0ec2e7fb13de7e96934866`, sections `FRI-06.4`,
`FRI-06.5`, the validation/error/cache portions of `FRI-06.6`, the input,
transaction, public-surface, and compatibility rows of `FRI-06.9` and
`FRI-06.10`, and applicable acceptance items 2, 10, 11, 15, 16, and 17 in
`FRI-06.14`.

Reviewed sequence:
`plans/sequences/2026-07-17-surgeist-layout-fri-06-inline-formatting-floats-bfcs.md`
at normalized SHA-256
`ed9b4a5bac63617ad5d7d3c76791dd42d93089a210ab16c930a7e727ed7edd57`,
commit `24bb3ccd0a4c9f54bc9eaa7958a9d2ea740bf859`, entry `FRI-06-C01`.

## Outcome

Add the complete validated public input/output substrate for shaped inline text,
atomic participation, fragments, float-exclusion queries, canonical non-box
tree nodes, exact errors, unit-key-preserving invalidated layout, committed
fragment restoration, and immutable-prepare/infallible-commit batch application.
Leave text line construction, mixed atomic/control behavior, float placement,
and shape-provider invocation to C02 through C05.

## Boundary

At the cycle base, `LayoutInputOf` has only box, line-break, and boundary states;
`NodeInputOf` has no atomic/exclusion fields or canonical non-box value; batch
output has node/cache entries only; `CacheKeyContext` is a unit; cache lookup has
no invalidation closure; and `LayoutTree` has neither fragment readback nor shape
query methods.

This cycle owns the reviewed public and private-field values in
`src/node_input.rs`, phase-specific fragment and transaction carriers in
`src/output.rs`, exact trait additions in `src/traits.rs`, invalidation,
validation, staging, and rounding integration in `src/compute.rs`, preservation
of the unit key in `src/cache.rs`, crate-root reexports/rustdoc, direct callers,
and focused model/contract/root/cache tests.

The cycle adds no production line construction. After canonical non-box
validation, a reached `LayoutInputOf::InlineText` returns the existing typed
`UnsupportedCapability(LaterFriBehavior)` without output or cache mutation until
C02 consumes it. Atomic participation is validated but not composed until C03.
A valid floating `FloatExclusion::Shape` likewise remains typed later behavior
until C05; invalid shape/float roles fail immediately. No valid new state may
panic, become a measured leaf, or silently use box or margin-box behavior.

This cycle does not change `src/inline.rs` line algorithms, `src/block.rs` float
placement/BFC behavior, browser helper/parser/serializer support, HTML,
`corpus.toml`, XML, reports, provenance, README, dependencies, features,
lockfile, MSRV, root/sibling repositories, generator architecture, or later FRI
behavior. No generation command runs. `just verify-generator` is compilation and
test evidence only; the single final full regeneration remains C06-owned.

## Impacts

Public API: intentional pre-release additions and breaks exactly matching the
specification compatibility table: `LayoutInputOf::InlineText`, two
`NodeInputOf` fields and `non_box()`, `VerticalAlign::Bottom`, invariant-bearing
inline/fragment/exclusion values and aliases, two `LayoutTree` default methods,
phase-specific fragment accessors, invalidated layout, invalidation readback,
two-phase `LayoutBatchSink`, `apply_to`, typed input/context/invariant errors,
and two exhaustive `LayoutOperation` variants. No alias or permissive conversion
is retained.

Dependencies, features, generated artifacts, examples, README, MSRV, root, and
siblings: unchanged. Crate-root rustdoc changes only as required to document the
new leaf public contract. Safety: all owned Rust remains unsafe-free; no new lint
allowance is permitted.

## Tasks

### `C01-T1` Validated Inline Inputs And Canonical Non-Box Pairing

**Files:** `src/node_input.rs`, `src/compute.rs`, `src/lib.rs`, direct layout-input
matches and constructors, and focused model/root/public-contract tests.

**Outcome:** Add opaque segment IDs, bidi levels, break kinds/opportunities,
whitespace edges, shaped segments/text, atomic participation, `InlineText`,
`Bottom`, canonical non-box construction, exact defaults/accessors/errors, and
the two new `NodeInputOf` fields. Validate non-box equality, childlessness,
measurement absence, atomic display pairing, replacement/discard state, and
shape-role compatibility before cache/output activity. Route otherwise-valid
text and shape capability to the bounded typed C02/C05 handoff.

**RED:** Add tests prefixed `fri06_c01_inline_model_` and
`fri06_c01_non_box_` first. They fail because the values, fields, variant,
constructors, validation, and typed errors do not exist. Record failures caused
by the missing contract rather than unrelated compilation setup.

**Acceptance:** Both scalar lanes cover every constructor/accessor/derive and
finite failure; bidi 0/125/126; all break kinds and replacement extents;
duplicate IDs; empty text; every whitespace edge; replacement/discard rejection;
atomic replacement rejection; defaults; exact non-box equality; children and
measurement rejection; atomic missing/extraneous facts; shape/non-float,
hidden, and absolute rejection; and explicit typed later behavior for valid
text/shape attempts. All exhaustive owned matches compile without wildcard
fallback, panic, synthetic measurement, or new lint suppression.

**Commands:**
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c01_inline_model_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c01_non_box_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
```

**Dependency:** Clean specification/sequence and recorded cycle base.

**Intended commit:** `api(layout): add validated inline input model`.

### `C01-T2` Phase-Specific Fragment Output And Warm Readback

**Files:** `src/output.rs`, `src/traits.rs`, `src/compute.rs`, `src/lib.rs`, cache/
rounding test trees, and focused output/root/cache/public-contract tests.

**Outcome:** Add immutable fragment/output-entry carriers and accessors,
separate unrounded/final fragment vectors and batch accessors, staged fragment
state, source-order identity, one-pass rounding, `LayoutTree` unrounded-fragment
readback, and exact warm missing-state invariant. Preserve node/cache batch
accessors and ordinary trees with no inline text.

**RED:** Add tests prefixed `fri06_c01_fragment_` first. They fail because no
fragment carrier, batch phase, tree readback, staged rounding source, or missing
warm-state error exists. Record the expected failures before implementation.

**Acceptance:** Both scalar lanes prove private output construction, all
accessors, nonempty and `Some(&[])` committed slices, source-tree then segment
order, separate unrounded/final geometry, one rounding pass, cold/staged and
warm/committed equality, no shape/text recomputation on readback, hidden absence,
and `None` on a warm inline-text path returning
`MissingCachedInlineFragmentState` without publishing a substitute. Existing
batch node/cache output and ordinary default trait implementors remain coherent.

**Commands:**
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c01_fragment_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout cache -- --nocapture
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
```

**Dependency:** `C01-T1` supplies text identity and non-box validation.

**Intended commit:** `api(layout): publish phase-specific inline fragments`.

### `C01-T3` Transactional Invalidation And Batch Application

**Files:** `src/cache.rs`, `src/output.rs`, `src/traits.rs`, `src/compute.rs`,
`src/lib.rs`, fake state stores, and focused cache/root/transaction/public tests.

**Outcome:** Add `compute_layout_invalidated`, source-order DFS reachability and
inclusive ancestor closure, exact unreachable error/operation, closure cache
bypass/staging, private batch closure plus `invalidated_nodes`, and the reviewed
`LayoutBatchSink` immutable preparation/infallible commit API with `apply_to`.
Keep `CacheKeyContext` a unit and ordinary `compute_layout` equivalent to an
empty dirty set.

**RED:** Add tests prefixed `fri06_c01_invalidation_` and
`fri06_c01_batch_transaction_` first. They fail because dirty nodes cannot bypass
stale cache hits, no path closure or unreachable diagnostic exists, and batch
application has no enforceable two-phase transaction. Record both expected
failures.

**Acceptance:** Exact trees prove empty, leaf, sibling, nested, duplicate, root,
and unreachable changed sets; source-order deduplication; closure-only bypass;
ordinary descendant hits; staged clear/store ordering; no unit-key change; no
mutation on layout or immutable preparation failure; an owned complete prepared
replacement; infallible exclusive node/fragment replacement; all clears before
all stores; dirty-state release only after `Ok`; and unchanged legacy
`compute_layout`. Contract fakes expose every mutation and fail if preparation
uses interior mutation or commit omits/reorders one state class.

**Commands:**
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c01_invalidation_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c01_batch_transaction_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
```

**Dependency:** `C01-T2` supplies every fragment phase the transaction owns.

**Intended commit:** `api(layout): add transactional cache invalidation`.

### `C01-T4` Float Exclusion Query Contract And Public Surface Closure

**Files:** `src/node_input.rs`, `src/traits.rs`, `src/compute.rs`, `src/output.rs`,
`src/lib.rs`, crate-root rustdoc, and focused provider/error/API/static tests.

**Outcome:** Finish the closed `FloatExclusion`, validated physical query and
interval carriers, default no-result tree method, exact missing/invalid/provider
error plumbing and operation/site context needed by C05, all scalar aliases and
reexports, compile/static compatibility evidence, and aggregate C01 public
documentation. Provider invocation and band refinement remain absent.

**RED:** Add tests prefixed `fri06_c01_float_exclusion_` and
`fri06_c01_contract_` first. They fail because query/result carriers, trait
method, diagnostics, aliases/reexports, and complete compatibility evidence are
absent. Record expected missing-surface failures.

**Acceptance:** Both scalar lanes prove every valid finite margin box/band/
interval, clipping and empty intersection, inverted/non-finite rejection,
default `MarginBox`, default provider `None`, provider error type preservation,
container/float/band diagnostics, and cache-neutral query values. Public/static
evidence covers every C01 addition/break, opaque payload, default/non-default,
constructor/accessor, exhaustive operation match, no cache revision field, no
compatibility alias, and no provider call or rectangular fallback for `Shape`.
Rustdoc and compile-fail examples are warning-free.

**Commands:**
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c01_float_exclusion_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c01_contract_
RUSTDOCFLAGS="-D warnings" CARGO_NET_OFFLINE=true cargo doc --locked --offline -p surgeist-layout --no-deps
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
```

**Dependency:** `C01-T3` completes error, batch, and cache plumbing.

**Intended commit:** `api(layout): close FRI-06 public substrate`.

## Cycle Acceptance

1. All four task ranges have reconstructed RED evidence, green acceptance, and
   independent clean task reviews.
2. Every C01 public value, alias, field, constructor, accessor, default,
   non-default, validation error, compatibility break, and scalar lane matches
   the reviewed specification without a permissive or mutable invariant path.
3. Non-box, atomic, replacement/whitespace, shape role, provider output, and
   invalidation reachability failures occur before cache/output publication and
   retain exact diagnostic context.
4. Fragment output is phase-specific, atomically associated with node/cache
   state, restorable from committed nonempty/empty slices, rounded once, and
   missing warm state fails rather than fabricating output.
5. Invalidated layout derives only the exact inclusive ancestor closure,
   preserves the unit key, bypasses stale hits without mutation, and applies one
   fully prepared replacement through infallible ordered commit. Every failed
   phase leaves old state and dirty subjects intact.
6. Valid text and shape remain explicit typed C02/C05 capability handoffs; no
   line algorithm, atomic/control composition, float placement, provider query,
   fallback, panic, or silent approximation enters C01.
7. Normal and generator-feature verification, docs, formatting, diff checks,
   and owned-Rust unsafe absence are clean with no generator-binary execution or
   artifact/input delta.
8. After every task is independently clean, the canonical Surgeist gate governs
   status completion, final checks, distinct exact-range holistic review,
   landing on local `main`, repeated main gates, immutable-SHA publication,
   remote readback, and resource cleanup. C02 cannot begin before C01 is
   published and remotely verified.

## Final Verification

```sh
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
RUSTDOCFLAGS="-D warnings" CARGO_NET_OFFLINE=true cargo doc --locked --offline -p surgeist-layout --no-deps
CARGO_NET_OFFLINE=true just fmt-check
git diff --check 24bb3ccd0a4c9f54bc9eaa7958a9d2ea740bf859..HEAD
git diff --name-only 24bb3ccd0a4c9f54bc9eaa7958a9d2ea740bf859..HEAD
git diff --name-only 24bb3ccd0a4c9f54bc9eaa7958a9d2ea740bf859..HEAD | rg -v '^(plans/cycles/2026-07-17-surgeist-layout-fri-06-c01-public-inline-model-transaction-substrate\.md|src/.*\.rs|tests/.*\.rs)$'
git status --short
rg -n --pcre2 '#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{' --glob '*.rs' src tests
```

The allowed-path negative filter must report no path, and status must be clean.
The unsafe scan must report no executable match. The complete cycle inventory
must contain only its plan and intended Rust source/tests/rustdoc. It contains no
README, helper, parser, serializer, HTML, manifest, XML, report, provenance,
dependency, feature, lockfile, MSRV, root, sibling, generated artifact, or
generator execution change.

## Handoff And Blockers

Only after clean task reviews, final checks, distinct holistic review, local-main
landing, publication, and remote readback does the completed cycle hand C02
validated text participants, canonical non-box pairing, fragment phases, warm
readback, transactional invalidation/application, and bounded float-exclusion
facts. It does not claim any FRI-06 finding closed or emit the final
root/text/shape candidate handoff.

A genuine blocker exists only if the reviewed public model cannot be represented
without a dependency/feature/MSRV change, unsafe code, generator expansion, a
cache revision token, a fallible mutation phase, or a product decision absent
from the clean specification. Such evidence returns to specification/sequence
review; it does not authorize a compatibility alias, silent fallback, broad lint
allowance, or scope expansion.
