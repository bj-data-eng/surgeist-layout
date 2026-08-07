# P01-I07-S01-C03 Flex Composition Closure

Status: in_progress

Cycle ID: `P01/I07/S01/C03`

Owning repository: `surgeist-layout`

Cycle base: `d4915cb6bc5bab629b2236f28b024c41f5feb88a`

Reviewed specification:
`plans/specs/P01-I07-flex-algorithm-completeness.md`, normalized semantic-content
SHA-256 `9e2a899476d27e09133a05531cb4bb4dfab1479d949d66167548c26ee1972b57`,
commit `451954e6aab6529ce7464c299be7e2aff6ea3753`: `FRI-07.4 D-14`, complete
`FRI-07.9`, and the composition, documentation, negative-control, finding-
closure, and acceptance portions of `FRI-07.11`, `.12`, `.14`, `.15`, and `.16`.

Reviewed implementation sequence:
`plans/sequences/P01-I07-S01-flex-algorithm-completeness.md`, normalized
SHA-256 `50f29d416df158ed8ceb799d4592fc2d53e36e6512a676bf498b05722a15b964`,
commit `0f0c19e50d8f6c27e300a5bc652e21ee5145b7cc`, entry
`P01/I07/S01/C03`.

Bounded outcome: close the combined behavior of `FLEX-002` through `FLEX-005`
through real public-front-door layouts, preserve the sole existing owners for
order, axes, replaced sizing, overflow, cache, and transaction phases, and
complete crate documentation for intrinsic flex bases and normalized flex-item
collapse without claiming root-owned style resolution.

## 1 Boundary

The remotely verified C02 candidate at the cycle base is the immutable entry
state. C01 separately closes direct intrinsic flex-basis measurement, ordinary
cross-axis auto margins, and inset-aware absolute flex-child auto margins. C02
separately closes the public collapse model, finite strut replay, zero committed
collapsed output, and collapse composition with order, axes, replaced sizing,
overflow, cache, transaction, rounding, and both scalar lanes.

C03 adds no fifth capability and does not reimplement those individual
matrices. Its missing proof is one composed public layout in which min/max-
content flex items, ordinary cross-axis auto margins, an absolute flex child
with auto margins and insets, and a collapsed flex item all participate while
stable order-modified source association and existing flow projection remain
observable. Bounded rotations then prove that changing each owned input changes
only its expected geometry or used-margin result.

The combined layout must also prove settled overflow and scrollbar behavior,
cold/warm cache equality, atomic provider failure before either collapse round,
rounding, and f32/f64 agreement. Existing C01 and C02 focused groups remain the
individual negative controls. New tests may reuse test-only builders and output
helpers, but production remains on the existing collected-item sequence,
`FlexAxes`, resolved-basis states, ordinary cross-margin helper, absolute
inset-aware margin phase, and finite collapse orchestration.

If the composed tests pass at the task base, they are characterization evidence
and no production edit is authorized. A failing public-front-door case permits
only the smallest correction in the existing owning phase, retained as a
failing regression until green and independently reviewed. It does not permit
a parallel owner, general refactor, or altered requirement.

Out of scope: new public fields or variants; general visibility; authored CSS
or computed-style parsing; root lowering or facade work; general positioned
layout; new sizing behavior; fixture-specific production branches; HTML,
fixture, parser, helper, manifest, XML, report, corpus, browser pin, or generator
changes; generation or artifact-writing modes; generator architecture;
dependencies, features, MSRV, manifests, lockfiles, scripts, CI, lints, and
unrelated cleanup. No new allow, expect, or suppression and no Surgeist-owned
`unsafe` are permitted.

## 2 Impacts

Public API: no signature, field, variant, reexport, compatibility, or capability
change. Existing `FlexBasisOf::MIN_CONTENT`, `MAX_CONTENT`,
`FlexItemCollapse`, and `NodeInputOf::flex_item_collapse` behavior is composed
and documented.

Production behavior: unchanged unless a new composed public-layout regression
proves a genuine interaction defect. Any correction stays in the existing
owner and preserves every individual C01/C02 focused group.

Dependencies, features, manifests, lockfiles, MSRV, and root: unchanged. Root
continues to own computed `visibility` lowering, facade composition, generated
API artifacts, and gitlink promotion.

Generated artifacts: unchanged. C03 changes no fixture or generator input and
runs no scoped or full generation. `just verify-generator`,
`just corpus-check`, and `just taffy-check` are read-only verification.

Docs/examples: `README.md` and crate-level documentation in `src/lib.rs` gain
the completed intrinsic-basis and normalized collapse boundary. They must not
claim general visibility, authored-style parsing, painting, root integration,
browser-fixture completion, or later-owned flex behavior.

## 3 Tasks

### 3.1 `P01/I07/S01/C03/T01` Compose All Four Flex Capabilities

**Files/area:** `src/flex_tests.rs` and only the existing owning production
portion of `src/flex.rs`, `src/compute.rs`, or `src/sizing.rs` if a composed
public-front-door regression exposes a genuine defect.

**Outcome:** add a bounded `fri07_c03_composed_layout_` family whose single
public flex tree contains order-modified min-content and max-content items,
ordinary cross-axis auto margins, an inset-positioned absolute child with auto
margins, and a collapsed item whose first-round used cross size becomes a
strut. Use the existing measurement provider, public request, and batch output.
Pair deterministic controls with the existing `proptest` dependency's bounded
finite strategies; do not add a generator, corpus, or reusable test framework.

**RED evidence:** add the composed tests before any production correction and
run them at the exact task base. If the four completed capabilities already
compose, record the passing characterization and do not fabricate RED. Any
observed wrong geometry, margin, source association, measurement constraint,
strut, or contribution remains failing until the smallest owner-local fix.

**Acceptance:** paired controls independently rotate min/max basis, normal and
collapsed state, item order/source position, containing flow and flex reversal,
wrap mode, replaced state, ordinary cross-auto-margin edge pattern, absolute
inset/auto-margin pattern, and normalized overflow. Exact assertions cover
provider constraints, item size/location, used margins, zero collapsed output,
strut-preserved line size, absolute geometry, source indices, root scroll
geometry, and no collapsed contribution. Property-generated bounded finite
inputs cover that complete dimension set and assert finite non-negative box
sizes, stable source association, at-most-two collapse rounds, and the paired
control invariants. Both scalar lanes agree within the existing named tolerance
and all C01/C02 focused groups remain green.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri07_c03_composed_layout_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri07_c01_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri07_c02_
cargo fmt --check
```

**Dependency:** reviewed planning packet only. **Intended commit:**
`test(layout): compose completed flex capabilities` when characterization is
green. When RED proves a production defect, commit the failing evidence first
as `test(layout): expose composed flex interaction`, then commit the owner-local
correction as `fix(layout): correct composed flex interaction`.

### 3.2 `P01/I07/S01/C03/T02` Close Combined State And Failure Evidence

**Files/area:** `src/flex_tests.rs` and only T01-authorized production owners if
this task proves a distinct genuine defect.

**Outcome:** extend the composed public tree with
`fri07_c03_composed_state_` controls for cache, transaction, overflow
settlement, exact collapse-round bounds, rounding, and scalar behavior without
copying a cache key, transaction layer, overflow pair, or round orchestrator.

**RED evidence:** write the state/failure tests before any task-local production
correction. Passing behavior is valid characterization. A genuine failure must
name the first observable divergence and remain RED through the public compute
entry point until its smallest in-scope correction.

**Acceptance:** cold and warm batches agree for all outputs and committed cache
facts; provider failure during intrinsic measurement and during the second
round's leaf resolution after second collection commits neither partial output
nor cache state; recovery matches a fresh tree; measurement traces prove no
more than the existing normal and collapsed rounds; rounded and unrounded
source-associated outputs remain coherent; overflow and scrollbar settlement
exclude first-round collapsed facts; and f32/f64 results agree. Controls also
prove unchanged later-owned flex-basis capability payloads and inert collapse
on the composed absolute child.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri07_c03_composed_state_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri07_c03_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri07_c01_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri07_c02_
CARGO_NET_OFFLINE=true just verify
cargo fmt --check
```

**Dependency:** T01. **Intended commit:**
`test(layout): close composed flex state evidence` when characterization is
green. When RED proves a distinct production defect, commit the failing evidence
first as `test(layout): expose composed flex state defect`, then commit the
owner-local correction as `fix(layout): correct composed flex state`.

### 3.3 `P01/I07/S01/C03/T03` Document The Completed Leaf Boundary

**Files/area:** `README.md`, crate-level documentation in `src/lib.rs`, and
existing public-item documentation only if required for a valid rustdoc link.

**Outcome:** document that direct min-content and max-content flex bases retain
distinct intrinsic measurement constraints, and that
`NodeInputOf::flex_item_collapse` is a normalized layout-ready flex effect with
normal default, finite strut participation, zero committed collapsed geometry,
and hidden descendants. State that root owns computed-style lowering from
`visibility: collapse`, while rendering owns painting.

**RED evidence:** deterministic documentation inspection at the task base shows
that README and crate-level docs describe typed flex-basis values generally but
do not yet state completed intrinsic basis behavior or the normalized collapse
boundary. No fabricated runtime failure is required for documentation.

**Acceptance:** README and crate docs agree with public names and implemented
semantics, rustdoc links resolve under configured doctests, and wording claims
no authored CSS parser, general visibility model, root integration, browser
artifact completion, inline-flex completeness, or later-owned behavior.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --doc
CARGO_NET_OFFLINE=true just verify
git diff --check
```

**Dependency:** T01 and T02 behavior accepted. **Intended commit:**
`docs(layout): describe completed flex semantics`.

## 4 Completion

The canonical implementation, task-review, status, holistic-review, landing,
publication, readback, and cleanup lifecycle applies. C03 acceptance is:

1. all four FRI-07 findings compose in one public layout with exact independent
   geometry, margin, strut, source, overflow, cache, transaction, rounding, and
   scalar evidence;
2. every existing owner remains sole and every C01/C02 focused group stays
   green;
3. any production edit is justified by recorded RED and changes only the owning
   phase; otherwise C03 remains tests and documentation;
4. crate docs accurately describe intrinsic bases, normalized collapse, and the
   root/rendering ownership boundary; and
5. no public API, dependency, feature, MSRV, root, fixture, parser, helper,
   corpus, generated artifact, browser pin, generator, lint, suppression,
   script, CI, or unrelated change enters the candidate.

At the complete-status head, run:

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri07_c03_composed_layout_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri07_c03_composed_state_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri07_c03_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri07_c01_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri07_c02_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
git diff --exit-code d4915cb6bc5bab629b2236f28b024c41f5feb88a..HEAD -- . ':(exclude)plans/cycles/P01-I07-S01-C03-flex-composition-closure.md' ':(exclude)README.md' ':(exclude)src/lib.rs' ':(exclude)src/flex.rs' ':(exclude)src/flex_tests.rs' ':(exclude)src/compute.rs' ':(exclude)src/sizing.rs'
! git diff --unified=0 d4915cb6bc5bab629b2236f28b024c41f5feb88a..HEAD -- src/lib.rs | rg --pcre2 '^[+-](?![+-]|//!)'
! git diff --unified=0 d4915cb6bc5bab629b2236f28b024c41f5feb88a..HEAD -- '*.rs' | rg --pcre2 '^\+(?!\+\+).*#\s*\[.*\b(?:allow|expect)\s*\('
! rg -n --pcre2 '#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{' --glob '*.rs' src tests
git diff --check d4915cb6bc5bab629b2236f28b024c41f5feb88a..HEAD
git status --short
```

The repository exclusion gate, crate-doc-only check, changed-line lint check,
and unsafe scan print no output; status is clean. The generator-feature, corpus,
and Taffy commands are read-only and do not authorize regeneration. The
remotely verified C03 handoff gives C04 behavior-complete, documented leaf
semantics so fixture inputs can settle without redesign. Genuine blockers are
limited to those defined by the installed workflow; none is currently known.
