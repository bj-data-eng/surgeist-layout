# FRI-06-C03 Mixed Atomic And Control Line Completion

Status: in_progress

Cycle ID: `FRI-06-C03`

Owning repository: `surgeist-layout`

Cycle base: `ea6fa7c26b00d4c61ad7e8d115bd45dcb36f7962`

Reviewed specification:
`plans/specs/2026-07-17-surgeist-layout-fri-06-inline-formatting-floats-bfcs.md`
at normalized SHA-256
`7cb09e0a8e9036a406b39115ed8f6392df805116a762905a3510c7fe7355f970`,
commit `64a9ca96be3b29765b0ec2e7fb13de7e96934866`, section `FRI-06.4`
decisions `D-03` and `D-06` through `D-13`; the atomic, control, baseline,
percentage, and non-box portions of `FRI-06.5` and `FRI-06.6`; the participant,
break, bidi, alignment, atomic-baseline, control/clear, mixed-source, size,
cache, scroll, and rounding portions of `FRI-06.7` through `FRI-06.10`; and the
applicable acceptance items in `FRI-06.14`.

Reviewed sequence:
`plans/sequences/2026-07-17-surgeist-layout-fri-06-inline-formatting-floats-bfcs.md`
at normalized SHA-256
`ed9b4a5bac63617ad5d7d3c76791dd42d93089a210ab16c930a7e727ed7edd57`,
commit `24bb3ccd0a4c9f54bc9eaa7958a9d2ea740bf859`, entry `FRI-06-C03`.

## Outcome

Complete one logical inline participant engine for shaped text, atomic boxes,
visible and hidden line breaks, inline boundaries, and non-participating
float/out-of-flow placeholders. Consume atomic bidi and following-break facts;
preserve source association while assigning mixed visual slots; finish struts,
controls, clear, atomic baseline/top/bottom placement, percentage block basis,
and fixed-size baseline behavior; and remove the obsolete text-only, vertical,
and whole-run shortcuts owned by FRI-06.

## Boundary

At the cycle base, shaped text uses `layout_text_run`; box/control runs use
`layout_inline_run`; vertical box/control runs divert again through
`layout_vertical_inline_run`. `layout_inline_run_children` accepts an all-text
run but returns `mixed_inline_later_capability_error` when any text shares a run
with an atomic or control. The legacy atomic path does not consume
`AtomicInlineParticipationOf`, wraps only between boxes, and does not include
atomics or boundaries in bidi visual slots.

Atomic child computation currently withholds the containing block basis even
when definite. Its baseline path treats `Top` as a synthetic zero baseline,
does not distinguish `Bottom`, and does not force block-end margin fallback for
non-visible used overflow. The fixed-size `ComputeSize` predicate treats text as
baseline-bearing but excludes every line break and boundary. Existing vertical
clear tests retain panic behavior, and FRI-06-owned inline/control helpers remain
behind dead-code allowances.

This cycle owns the one mixed participant source and line records in
`src/inline.rs`, production lowering/publication and directly required fast-path
or error cleanup in `src/block.rs` and `src/compute.rs`, and focused Rust tests.
Text-only C02 behavior, fragment phases, transaction/cache invalidation,
rounding, scroll accumulation, and existing valid atomic/control behavior remain
unless the reviewed mixed contract corrects them.

C04 owns float-adjusted line bands, rectangular float placement, BFC avoidance,
and float auto sizing. C05 owns provider-backed shapes. C06 owns parser/helper,
comparator, HTML, fixtures, XML, reports, and the one final full regeneration.
No authored text, shaping, glyph/font dependency, synthetic text measurement,
manifest, dependency, feature, root, sibling, fixture, browser, report,
provenance, or generator execution enters C03. No new lint allowance or
Surgeist-owned `unsafe` is permitted.

## Impacts

Public API and type shape: unchanged; this cycle consumes the C01 public model.
Dependencies, features, lockfile, MSRV, docs/examples, root handoff, generated
artifacts, and fixture lineage: unchanged. Generator architecture and execution:
absent. Behavior advances represented mixed inline inputs from typed later or
legacy incomplete paths to the reviewed C03 contract. Owned Rust remains free of
`unsafe`, and obsolete FRI-06 dead-code allowances in the consumed paths are
removed.

## Tasks

### `C03-T1` Compose Text And Atomic Boxes Through One Break And Bidi Source

**Files:** `src/inline.rs`, `src/block.rs`, `src/compute.rs` only for deleting the
obsolete mixed-later helper, and focused inline/block/root Rust tests.

**Outcome:** Extend the C02 logical source with atomic units retaining child and
source identity, margin-box logical advance, selected baseline facts, supplied
`BidiLevel`, and supplied following break. Lower text and visible in-flow atomic
children in one source-ordered stream, excluding hidden, floated, and absolute
children from participation without collapsing source indices. Select lines
greedily across text and indivisible atomics; consume prohibited/allowed/
mandatory atomic opportunities; include atomic margins in min/max-content; run
complete-unit bidi per selected line; and publish atomic nodes plus text
fragments from the same line/visual source.

**RED:** Add production-front-door tests prefixed `fri06_c03_mixed_`,
`fri06_c03_atomic_break_`, and `fri06_c03_mixed_bidi_` first. At exact T1 base
they fail with the typed mixed-inline later capability or legacy box-only
wrapping/visual order. Preserve reconstructible test-only RED evidence and prove
temporary resources cleanly removed.

**Acceptance:** Both scalar lanes prove text before/between/after multiple
atomics, prohibited/allowed/mandatory opportunities, latest rollback, overwide
atomic progress, mandatory groups, text/atomic min/max-content, nested even/odd
mixed bidi, separate line reorder, stable source-node/segment association, atomic
and discarded-text visual-index gaps, hidden/absolute/float source gaps, and
deterministic repeatability. Required atomic participation is consumed exactly
once; no default bidi or break is guessed. Text is never measured as a leaf.
Text-only C02 wrapping, whitespace, fragment, alignment, and intrinsic evidence
stays green. As an explicit interim boundary, any run containing both text and a
visible or hidden line-break/boundary control still returns the existing typed
mixed-inline later capability before child computation or staging; text+atomic
runs no longer do. T1 does not add an opaque control unit or invent partial
control break/bidi semantics. T2 removes this final control-specific gate.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c03_mixed_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c03_atomic_break_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c03_mixed_bidi_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout
CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
CARGO_NET_OFFLINE=true just fmt-check
```

**Dependency:** Reviewed plan at `in_progress`. **Intended commit:**
`feat(layout): compose mixed text and atomic lines`.

### `C03-T2` Integrate Controls, Empty Struts, And Logical Clear

**Files:** `src/inline.rs`, `src/block.rs`, and focused inline/block/root Rust
tests.

**Outcome:** Lower visible line breaks and inline boundaries into the normalized
participant source. A boundary occupies one base-level bidi unit and contributes
metrics without an independent break. A visible break contributes zero advance,
commits before reordering, publishes at visual line end, and seeds the required
following strut. Hidden breaks contribute no line or metrics and publish hidden
output. Validate `Clear::{Left,Right,Both}` against containing `FlowAxes`, retain
one mapped line-start/line-end post-line clear intent in the unified report, and
accept every clear value in all ten flow mappings. C04 alone applies that intent
to active exclusions and float-adjusted bands.

**RED:** Add front-door tests prefixed `fri06_c03_control_`,
`fri06_c03_strut_`, and `fri06_c03_clear_` first. At exact T2 base, mixed
text/control runs return the retained typed later capability before staging,
while no-text controls still follow incomplete legacy composition; boundary
visual slots are absent, adjacent/final control struts are incomplete, or
vertical and sideways clear panics/rejects. Preserve reconstructible test-only
RED evidence.

**Acceptance:** Both scalar lanes prove leading, trailing, adjacent, only-child,
and final visible breaks/boundaries; hidden breaks; control-only and empty
post-mandatory lines; first/last baselines; exact zero-size source-associated
control positions; boundary base-level visual slots among nested mixed bidi;
visible breaks fixed at visual line end; unequal line alignment; and all ten
flows times `Clear::{None,Left,Right,Both}` without panic or control-owned flow
override when no exclusion is active. Private builder evidence proves the mapped
post-line intent is none, line-start, line-end, or both from the containing flow.
Existing supported rectangular-clear cases remain regression-green, but C03
adds no all-flow active-exclusion advancement, float-band query, or cleared-float
placement evidence. Boundaries never clear. Separate control-only placement does
not remain; any legacy rectangular-clear bridge stays isolated for C04 rather
than entering the logical builder. `mixed_inline_later_capability_error` and its
text/control gate are absent after this task.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c03_control_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c03_strut_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c03_clear_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout
CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
CARGO_NET_OFFLINE=true just fmt-check
rg -n 'mixed_inline_later_capability_error' src
```

The final search must return no matches. **Dependency:** T1 is task-clean.
**Intended commit:**
`feat(layout): integrate inline controls and logical clear`.

### `C03-T3` Complete Atomic And Control Line Metrics And Percentage Basis

**Files:** `src/inline.rs`, `src/block.rs`, and focused inline/block/root Rust
tests.

**Outcome:** Resolve baseline, line-over `Top`, and line-under `Bottom` as
distinct line states after baseline sizing. For a baseline-aligned atomic, use a
usable inner block-axis baseline only with visible used overflow; otherwise use
the block-end margin edge. Include physical/logical margins and fallback
geometry exactly once. Give atomic child sizing the containing block's definite
block basis when present and preserve `None` when indefinite. Apply the same
line-over/line-under expansion discipline to metric controls.

**RED:** Add front-door tests prefixed `fri06_c03_atomic_baseline_`,
`fri06_c03_top_bottom_`, and `fri06_c03_percentage_` first. At exact T3 base,
missing/non-visible baseline fallback, bottom margins, bottom placement,
top/bottom expansion, replaced overflow conversion, or definite percentage
block sizing returns legacy geometry. Preserve reconstructible test-only RED.

**Acceptance:** In both scalar lanes and representative parallel/opposing flows,
prove visible inner first/last selection, absent-inner fallback, every
non-visible used-overflow fallback, block-end margin edge, positive/negative
atomic margins, replaced-hidden conversion, `Top` and `Bottom` placement after
baseline sizing, multiple mixed top/bottom participants expanding one line,
metric control placement, and stable first/last container baselines. A definite
containing physical/logical block size resolves atomic percentage block sizing;
an indefinite basis remains unresolved through existing sizing rules. No raw
authored overflow or anonymous line extent is substituted.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c03_atomic_baseline_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c03_top_bottom_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c03_percentage_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout
CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
CARGO_NET_OFFLINE=true just fmt-check
```

**Dependency:** T2 is task-clean. **Intended commit:**
`fix(layout): complete atomic inline line metrics`.

### `C03-T4` Close Fixed-Size, Projection, And Publication Lifecycle

**Files:** `src/inline.rs`, `src/block.rs`, `src/compute.rs` only for directly
required lifecycle cleanup, focused cache/contract/inline/block/root Rust tests.

**Outcome:** Make the fixed-size `ComputeSize` fast path retain layout whenever
text or a visible metric-bearing control can establish requested baselines or
required output, while size-only controls preserve the valid early return.
Project every mixed line only from the unified logical source in all ten flows;
remove the separate vertical and text-only whole-run shortcuts; and
close scroll, cache, invalidation, fragment, and rounding behavior for mixed
atomics/controls without changing the C01 transaction or unit cache key.

**RED:** Add front-door tests prefixed `fri06_c03_fixed_`,
`fri06_c03_projection_`, and `fri06_c03_lifecycle_` first. At exact T4 base,
metric controls can be erased by the definite-size fast path, legacy axis paths
or publication can diverge, or mixed cold/warm/rounded state lacks complete
proof. Preserve reconstructible test-only RED for each actual missing behavior;
do not claim false RED for already-correct transaction cases.

**Acceptance:** Both scalar lanes prove metric text/break/boundary retention,
size-only control fast-path call accounting, mixed soft/forced wrapping and
legacy left/right/center on unequal lines in all ten flows, source/visual/line/
baseline identity before and after rounding, one final scroll contribution,
cold/warm equality including text fragments and atomic/control nodes, exact
dirty-path replacement, and failure atomicity. One logical builder remains;
`layout_vertical_inline_run`, `layout_vertical_inline_lines`,
`layout_shaped_text_children`, and obsolete FRI-06 dead-code allowances are
absent. No shaping, arbitrary retry bound, duplicate projection, cache-context
field, partial publication, or panic remains for valid C03 input.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c03_fixed_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c03_projection_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c03_lifecycle_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c03_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout
CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
CARGO_NET_OFFLINE=true just fmt-check
rg -n 'layout_vertical_inline_run|layout_vertical_inline_lines|layout_shaped_text_children' src
rg -n '#\[allow\(dead_code\)\]' src/inline.rs
```

Both searches must return no matches. **Dependency:** T3 is task-clean.
**Intended commit:** `fix(layout): close mixed inline lifecycle`.

## Completion

After all four task ranges are independently clean, make the plan's separate
status-only `complete` commit and set the immutable cycle head. Run:

```sh
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c03_
CARGO_NET_OFFLINE=true just fmt-check
rg -n 'mixed_inline_later_capability_error|layout_vertical_inline_run|layout_vertical_inline_lines|layout_shaped_text_children' src
rg -n '#\[allow\(dead_code\)\]' src/inline.rs
git diff --unified=0 ea6fa7c26b00d4c61ad7e8d115bd45dcb36f7962..HEAD -- '*.rs' | rg '^\+.*#\s*\[\s*allow\s*\('
git diff --check ea6fa7c26b00d4c61ad7e8d115bd45dcb36f7962..HEAD
git diff --name-only ea6fa7c26b00d4c61ad7e8d115bd45dcb36f7962..HEAD
owned_rust_manifest="$(mktemp -t surgeist-layout-fri06-c03-owned-rust)"
trap 'rm -f "$owned_rust_manifest"' EXIT
git ls-files --cached --others --exclude-standard '*.rs' > "$owned_rust_manifest"
rg -n --pcre2 '#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{' $(cat "$owned_rust_manifest")
git status --short
```

The legacy/dead-code, newly added lint-allowance, and unsafe searches must return
no matches, with expected no-match statuses recorded and any textual unsafe
match classified. The diff-scoped allowance search covers every changed Rust
file; the inline-specific search additionally proves the consumed legacy
allowances are gone. The owned manifest includes tracked and non-ignored
untracked Rust, including test and generator Rust, and excludes ignored
dependency/build roots. The exact range may
contain only this cycle plan and expected Rust source/tests; specification,
sequence, report, parser, helper, HTML, fixture, XML, provenance, generator,
manifest, dependency, feature, lockfile, docs, root, sibling, and generated
artifact changes are forbidden.

The manifest assignment, exit trap, manifest construction, scan, and status
check execute in one shell scope. Record the unique path as cycle-owned before
the scan; the trap removes it on success, expected no-match exit, or failure.

A fresh `surgeist-holistic-reviewer` must return `CLEAN` for exact range
`ea6fa7c26b00d4c61ad7e8d115bd45dcb36f7962..cycle_head`. Land the reviewed
candidate on local `main`, or prove the sequential cycle already left local
`main` at that exact head, then rerun and record the complete final command set
there without changing the head. Only after those main-branch gates pass,
publish to the authority remote `main`, fetch/read back, and prove local,
tracking, and observed remote `main` agree with the candidate or a normal
descendant containing it. Remove every cycle-owned temporary resource.

The handoff records the published C03 candidate and confirms the unified inline
participant engine is ready for C04 rectangular float bands and BFC placement.
No C04 plan or implementation begins before remote verification. Blocker: none
at planning time.
