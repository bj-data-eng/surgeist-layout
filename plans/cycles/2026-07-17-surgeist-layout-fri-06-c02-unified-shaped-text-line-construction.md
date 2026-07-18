# FRI-06-C02 Unified Shaped-Text Line Construction

Status: in_progress

Cycle ID: `FRI-06-C02`

Owning repository: `surgeist-layout`

Cycle base: `1e5dfb927b92c41e1c5e952d97ba1f6450b8bb84`

Reviewed specification:
`plans/specs/2026-07-17-surgeist-layout-fri-06-inline-formatting-floats-bfcs.md`
at normalized SHA-256
`7cb09e0a8e9036a406b39115ed8f6392df805116a762905a3510c7fe7355f970`,
commit `64a9ca96be3b29765b0ec2e7fb13de7e96934866`, sections `FRI-06.4`
decisions `D-01` through `D-10`; the shaped-text, non-box, fragment, and cache
portions of `FRI-06.5` and `FRI-06.6`; the text, break, bidi, whitespace,
alignment, intrinsic, scroll, cache, and rounding portions of `FRI-06.7` through
`FRI-06.9`; the applicable `src/inline.rs`, `src/block.rs`, and focused-test
rows of `FRI-06.10`; and applicable acceptance items in `FRI-06.14`.

Reviewed sequence:
`plans/sequences/2026-07-17-surgeist-layout-fri-06-inline-formatting-floats-bfcs.md`
at normalized SHA-256
`ed9b4a5bac63617ad5d7d3c76791dd42d93089a210ab16c930a7e727ed7edd57`,
commit `24bb3ccd0a4c9f54bc9eaa7958a9d2ea740bf859`, entry `FRI-06-C02`.

## Outcome

Consume C01's validated shaped-text input through one text-only logical-axis
line source. Implement greedy break selection, replacement ownership, edge
whitespace, min/max-content contributions, complete-unit bidi ordering,
independent per-line legacy alignment, all ten `FlowAxes` projections, and
phase-correct text-node and fragment publication through the production block
front door. Preserve the completed transaction, cache, rounding, and scroll
contracts without shaping or synthetic measurement.

## Boundary

At the cycle base, `LayoutInputOf::InlineText` validates but deliberately returns
typed later behavior. `src/inline.rs` lays out only atomic boxes and controls,
uses separate horizontal and vertical implementations, has no soft text break
selection or bidi source, and reports no fragments. `src/block.rs` groups text
inside an inline run but rejects it before line construction. C01 already owns
validated text facts, fragment carriers, committed-fragment readback, exact
invalidation closure, and transactional publication; this cycle consumes those
contracts without changing their public shape.

This cycle owns a private text-only normalized participant path and logical line
records in `src/inline.rs`, incremental production composition/publication in
`src/block.rs` and `src/compute.rs`, directly required private plumbing, and
focused Rust tests.
Every selected line retains source anchors, its own available full containing
band, used inline extent, metrics, alignment offset, and visual slots before one
physical projection. A discarded segment remains a zero-advance bidi slot and
anchor but publishes no fragment. Public batch entries remain source-node then
segment order.

The line-band interface accepts the full containing band only. Float-adjusted
bands, shape-provider invocation, atomic participation facts, mixed text/box
bidi, visible break and boundary integration, clear, top/bottom atomic placement,
percentage atomic basis, and replacement of legacy box/control layout remain
C03 through C05. Existing valid atomic/control behavior stays unchanged until
C03; this cycle does not force those participants through an incomplete text
path. No authored text, glyph, font/shaping dependency, parser, fixture, helper,
HTML, manifest, XML, report, provenance, or generator execution enters C02.

## Impacts

Public API, dependencies, features, generated artifacts, README/examples,
lockfile, MSRV, root, and sibling repositories: unchanged. Behavior changes from
typed later capability to represented shaped-text layout through the existing
public C01 surface. Generator architecture and all generator inputs/outputs are
unchanged; no generation command runs. Safety: all owned Rust remains free of
unsafe. No new lint allowance is permitted, and every C02-owned dead-code
allowance made unnecessary by consumption is removed with the obsolete item or
attribute.

## Tasks

### `C02-T1` Logical Text Breaking, Whitespace, And Intrinsics

**Files:** `src/inline.rs`, `src/block.rs`, `src/compute.rs`, `src/output.rs` only
for directly required private construction, `src/inline_tests.rs`, and focused
block/root Rust test modules.

**Outcome:** Add private shaped-text participants and one monotone logical-axis
line-selection state. Consume segment extent, metrics, whitespace classification,
and following break facts; implement greedy latest-opportunity rollback,
selected replacement extent, mandatory and final commits, leading/trailing edge
discard, indivisible overwide progress, empty post-mandatory strut state, and
the reviewed min/max-content contributions. Wire the minimum production block
path needed for valid shaped text to reach this state and publish source-retaining
text node/fragment results, replacing typed later capability at this task base.

**RED:** Add tests prefixed `fri06_c02_break_`, `fri06_c02_whitespace_`, and
`fri06_c02_intrinsic_` first through `compute_layout` or production block
formatting; private builder tests are supplemental. At the exact task base they
fail because shaped segments cannot reach production line layout and break,
whitespace, replacement, fragment, and text intrinsic behavior are absent.
Preserve reconstructible test-only RED evidence and prove the temporary resource
is cleanly removed.

**Acceptance:** Both scalar lanes prove prohibited/allowed/latest rollback,
selected and unselected replacement, mandatory including final mandatory,
ordinary final commit, leading/trailing/both discard, a discarded opportunity,
all-discarded zero-fragment anchoring source, overwide first-unit progress,
multiple mandatory groups, deterministic repeatability, and exact min/max
content values. Replacement belongs only to its preserved preceding segment;
no reshape, measured leaf, arbitrary iteration bound, panic, or negative band
is introduced. Every named behavior has both-scalar production-front-door proof;
private line-state assertions only supplement it.

**Commands:**
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c02_break_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c02_whitespace_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c02_intrinsic_
CARGO_NET_OFFLINE=true just verify
```

**Dependency:** Clean reviewed C02 plan and exact cycle base.

**Intended commit:** `feat(layout): construct shaped-text logical lines`.

### `C02-T2` Per-Line Bidi, Alignment, And Physical Projection

**Files:** `src/inline.rs`, `src/block.rs`, `src/inline_tests.rs`,
`src/block_tests.rs`, `src/root_tests.rs`, `src/geometry.rs` only if an existing
`FlowAxes` projection helper is narrowly incomplete, and focused flow tests.

**Outcome:** Convert selected logical lines into one source-retaining projected
report. Apply stable descending-level reversal independently to each complete
line-unit slice, assign visual slots before placement, apply legacy left/right/
center from each line's own non-negative free space, and project segment rects,
baselines, zero-fragment anchors, and run geometry through `FlowAxes` for all ten
writing-mode/direction mappings. Keep source output order separate from visual
placement.

**RED:** Add tests prefixed `fri06_c02_bidi_`, `fri06_c02_alignment_`, and
`fri06_c02_flow_` first through `compute_layout` or production block formatting;
private builder tests are supplemental. At the exact task base they fail because
selected production text lines have no visual reordering, independent alignment,
physical fragment source, or all-flow projection. Preserve reconstructible
test-only RED evidence.

**Acceptance:** Both scalar lanes prove nested even/odd levels, stable equal-level
ordering, reorder reset per wrapped line, zero-advance discarded-slot gaps,
source segment order and IDs, unequal-line legacy left/right/center offsets,
overflow clamp-to-zero, physical rect/baseline/anchor geometry, aggregate run
extent, and horizontal-tb, vertical-rl, vertical-lr, sideways-rl, and sideways-lr
in both directions. Horizontal and vertical text use one line-selection and
projection source; no maximum-run proxy determines individual line placement.
Every named behavior has both-scalar production-front-door proof.

**Commands:**
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c02_bidi_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c02_alignment_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c02_flow_
CARGO_NET_OFFLINE=true just verify
```

**Dependency:** `C02-T1` supplies final selected logical line slices.

**Intended commit:** `feat(layout): project per-line shaped text`.

### `C02-T3` Source Association, Scroll, And Baseline Completion

**Files:** `src/block.rs`, `src/inline.rs`, `src/compute.rs`, `src/output.rs` only
for directly required private construction, and focused block/root/scroll tests.

**Outcome:** Complete the production path introduced by T1 across consecutive
text nodes and containing-block observables. Preserve source-tree then segment
ordering, publish each text node's complete unrounded fragment slice and ordinary
non-box union output, publish zero-fragment text at its retained anchor, include
actual fragment rects in containing scroll geometry exactly once, carry line/run
baselines and intrinsic results, and leave hidden text fragment-free. No
synthetic box, child traversal, or leaf measurement enters text layout.

**RED:** Add tests prefixed `fri06_c02_block_text_`,
`fri06_c02_fragment_publication_`, and `fri06_c02_scroll_` first. At the exact
task base they fail on incomplete multi-node association, union/anchor,
containing baseline, or scroll effects. Tests exercise `compute_layout` or
production block layout; private builder tests alone are not RED evidence.

**Acceptance:** Both scalar lanes prove one and multiple adjacent text nodes,
soft/mandatory wrapping, source-node and segment association, union location/
size/content size with zero edges and no text scroll container, all-discarded
anchor output, hidden absence, actual-fragment rather than union-proxy scroll
extent exactly once, containing baselines, min/max-content root requests, and
no node children/measurement/cache activity that violates canonical non-box
pairing. Existing atomic/control-only block tests remain unchanged and green.

**Commands:**
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c02_block_text_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c02_fragment_publication_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c02_scroll_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout block_atomic_inline_run_
CARGO_NET_OFFLINE=true just verify
```

**Dependency:** `C02-T2` supplies final physical text fragment sources.

**Intended commit:** `feat(layout): complete shaped-text publication`.

### `C02-T4` Cache, Rounding, And Text-Path Closure

**Files:** `src/compute.rs`, `src/block.rs`, `src/inline.rs`, focused
cache/invalidation/root/contract tests, and crate rustdoc only if the existing
C01 text contract requires a factual behavior update.

**Outcome:** Close C02 through the real cold, invalidated, warm-cache, rounded,
hidden, and failure paths. Ensure final fragments derive once from committed or
new unrounded line sources, dirty text bypasses stale node/fragment state, warm
text requires committed fragment state, and failed layout publishes nothing.
Remove the valid-text later-capability branch and every obsolete allowance or
text-only split made unreachable by C02 while retaining C03-owned legacy
atomic/control implementation.

**RED:** Add tests prefixed `fri06_c02_cache_`, `fri06_c02_rounding_`, and
`fri06_c02_transaction_` first. At the exact task base they fail because the
new production text path has not yet demonstrated cold/warm equivalence,
invalidation replacement, one-pass rounding, or failure atomicity end to end.
Preserve reconstructible public-front-door RED evidence.

**Acceptance:** Both scalar lanes prove cold versus warm equality, committed
nonempty and `Some(&[])` fragment restoration, missing warm state error,
root-to-dirty text closure bypass and replacement, duplicate dirty normalization,
normal versus rounded source/line/visual/replacement identity, fractional
geometry rounding exactly once, hidden state, deterministic replay, and zero
node/fragment/cache mutation after a failing computation or batch preparation.
Static evidence shows no valid-text `LaterFriBehavior`, no text measurement or
shaping call, no C02-owned dead-code allowance, no unsafe, and no change to unit
cache-key shape.

**Commands:**
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c02_cache_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c02_rounding_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c02_transaction_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c02_contract_
rg -n 'LaterFriBehavior' src/block.rs
rg -n 'shape|glyph|font|measure_leaf' src/inline.rs
rg -n '^pub struct CacheKeyContext;$' src/cache.rs
RUSTDOCFLAGS="-D warnings" CARGO_NET_OFFLINE=true cargo doc --locked --offline -p surgeist-layout --no-deps
CARGO_NET_OFFLINE=true just verify
```

The first two `rg` checks must report no line. The cache-key check must report
exactly the existing unit-struct declaration. `fri06_c02_contract_` inspects the
C02 text path and reports any C02-owned dead-code allowance, text capability
fallback, shaping/measurement call, or cache-key-shape drift with a named failure.

**Dependency:** `C02-T3` supplies production fragment publication.

**Intended commit:** `fix(layout): close shaped-text line lifecycle`.

## Cycle Acceptance

1. All four task ranges have reconstructible RED evidence, green acceptance,
   and independent clean task reviews.
2. Valid shaped text reaches production block layout in both scalar lanes and
   never invokes shaping, measurement, box layout, child traversal, or typed
   later capability.
3. One logical text line source implements deterministic break rollback,
   replacement, whitespace, mandatory/final lines, overwide progress,
   min/max-content, per-line bidi, independent alignment, and all ten flows.
4. Fragment identity remains source ordered while visual slots and geometry are
   line-local; discarded segments emit no fragment but retain deterministic
   slots/anchors, and selected replacement metadata stays on its source fragment.
5. Text node union output, zero-fragment anchoring, containing baselines, scroll
   contribution, cold/warm restoration, invalidation, failure atomicity, and
   one-pass rounding match C01's transaction contract.
6. Atomic boxes, existing controls, float bands, shape providers, fixtures, and
   generator inputs/outputs remain later-cycle behavior without regression,
   approximation, or architecture expansion.
7. Default verification, docs, formatting, diff checks, and owned-Rust unsafe
   absence are clean. No generator-feature command or generator binary runs.
8. After clean task reviews, the canonical gate governs status completion,
   exact-range holistic review, local-main gates, immutable-SHA publication,
   remote readback, and resource cleanup. C03 cannot begin before remote proof.

## Final Verification

```sh
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c02_contract_
rg -n 'LaterFriBehavior' src/block.rs
rg -n 'shape|glyph|font|measure_leaf' src/inline.rs
rg -n '^pub struct CacheKeyContext;$' src/cache.rs
RUSTDOCFLAGS="-D warnings" CARGO_NET_OFFLINE=true cargo doc --locked --offline -p surgeist-layout --no-deps
CARGO_NET_OFFLINE=true just fmt-check
git diff --check 1e5dfb927b92c41e1c5e952d97ba1f6450b8bb84..HEAD
git diff --name-only 1e5dfb927b92c41e1c5e952d97ba1f6450b8bb84..HEAD
git diff --name-only 1e5dfb927b92c41e1c5e952d97ba1f6450b8bb84..HEAD | rg -v '^(plans/cycles/2026-07-17-surgeist-layout-fri-06-c02-unified-shaped-text-line-construction\.md|src/.*\.rs|tests/.*\.rs)$'
git status --short
rg -n --pcre2 '#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{' --glob '*.rs' src tests
```

The allowed-path negative filter must report no path, status must be clean, and
the unsafe scan must report no executable match. The complete cycle inventory
contains only its plan and intended Rust source/tests/rustdoc. It contains no
README, parser, helper, serializer, HTML, manifest, XML, report, provenance,
dependency, feature, lockfile, MSRV, root, sibling, generated artifact, or
generator execution change.

The first two source checks must report no line; the cache-key check must report
exactly its unit-struct declaration. The static contract test must pass.

## Handoff And Blockers

Only after clean task reviews, final checks, distinct exact-range holistic
review, local-main landing, publication, and remote readback does C02 hand C03
one reviewed logical line source with shaped text, source/visual identity,
per-line geometry, all-flow projection, and transactional fragment publication.
It does not claim mixed atomic/control behavior, float exclusion, provider
integration, fixture activation, any FRI-06 finding closure, or the final root/
text/shape candidate handoff.

A genuine blocker exists only if represented shaped text requires a new
dependency/feature/MSRV, unsafe, generator expansion, public-model revision,
cache revision token, shaping/measurement call, or product decision absent from
the clean specification. Such evidence returns to specification/sequence review;
it does not authorize approximation, compatibility fallback, duplicated axis
algorithm, broad lint allowance, or scope expansion.
