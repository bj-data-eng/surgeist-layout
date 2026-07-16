# FRI-05 Overflow And Scroll Geometry Implementation Sequence

Status: draft

Sequence ID: `FRI-05`

Owning repository: `surgeist-layout`

## Authority

This sequence implements the independently reviewed specification at
`plans/specs/2026-07-16-surgeist-layout-fri-05-overflow-scroll-geometry.md`, SHA-256
`b2bc5d8cf1f7b65dcef74adf34b5b63ab1f8b519fcbd1094ff4e335ab419286f`,
committed as `5a51d0f67ef781eef724f86a9232bc3616c3773f`.

The specification is authoritative for behavior, API, ownership, artifacts, non-goals, and
acceptance. This sequence owns only durable ordering; only the next ready cycle receives a detailed just-in-time plan.

## Sequence Boundary

All cycles mutate only `surgeist-layout`. Root adapters, sibling CSS/style models, facade
exports, root API artifacts, and the root gitlink remain the separate handoff in `FRI-05.12`.
No cycle adds a dependency, feature, MSRV change, unsafe code, generator architecture, general CSS parser, retained snap registry, or later formatting behavior.

Scoped generation is optional iteration-only diagnosis and is never entry or exit evidence.
Before `C06`, `C01` may change only the fixture parser needed by the `NodeInputOf` migration: preserve the five accepted tokens and `Auto`, then apply exact CSS coupling to legacy authored pairs before atomic construction.
Focused parser/corpus evidence covers omitted axes and every existing cross-group fixture; helper, serializer, HTML, manifest, XML, reports, and provenance remain unchanged.
`C06` removes that transition when it switches to computed-style lowering and exclusively owns every other fixture semantic, token, input, or artifact change.
Once those inputs settle, `C06` performs one full ExistingPinned regeneration. `C07` is read-only for generated artifacts; a confirmed input bug returns to `C06`, invalidates the prior run, and permits one replacement full regeneration after corrected inputs settle.
The aggregate `parity-all` release gate remains FRI-13-owned.

## Ordered Cycles

### `FRI-05-C01` Canonical Overflow And Scroll Input Model

**Specification sources:** `FRI-05.4 D-01`, `D-02`; input portions of `FRI-05.5`, `FRI-05.6`, `FRI-05.8`, and `FRI-05.9`.

**Prerequisites:** Published FRI-04 candidate, clean FRI-05 specification review, and its recorded source base.

**Entry state:** Overflow is a mutable two-axis point without `Auto`; the parser carries legacy authored pairs; scroll properties remain deferred or unrepresentable; phase-unsafe predicates remain.

**Bounded outcome:** Add the validated thirteen-pair computed-overflow model,
all closed layout-ready D-02 input values and defaults, scalar-lane construction,
phase-correct computed/used predicates, the coherent `NodeInputOf` migration, and the bounded legacy fixture-pair transition.
Keep canonical derived geometry and formatting integration for later cycles.

**Observable exit evidence:** Every valid and invalid pair, default, scalar
validation, replaced used conversion, generic phase predicate, block pair
predicate, public input construction, and existing legacy fixture pair is proven without a deferred FRI-05
capability or compatibility alias.

**Handoff:** All algorithms receive canonical source facts; geometry can be derived without raw specified overflow or placeholder scroll values.

### `FRI-05-C02` Canonical Scroll Geometry Substrate

**Specification sources:** `FRI-05.4 D-03` through `D-11`; output portions of
`FRI-05.5`, `FRI-05.6`, `FRI-05.8`, and `FRI-05.9`.

**Prerequisites:** `FRI-05-C01` complete.

**Entry state:** Canonical inputs exist, while current rectangles, legacy facts, output constructors,
clipping, ranges, gutters, accumulation, and rounding do not form one coherent derivation.

**Bounded outcome:** Add finite-end rectangles, private used-overflow
derivation, axis clips, target metadata, proportional gutter saturation,
format-origin/alignment ranges, the shared accumulator, one canonical geometry
factory, nested target carrier, and source-fact rounding. Keep the production
switch and legacy-surface removal in the dependent integration cycle.

**Observable exit evidence:** Constructor/property tests prove coherence,
finite failures, partial clips, target presence, all flow projections, range
origin rules, positive-outset contribution, axis independence, saturation, and
rounding reconstruction in both scalar lanes without a second derived source.

**Handoff:** One reviewed substrate can replace format-local geometry logic.

### `FRI-05-C03` Root Leaf And Block Geometry Integration

**Specification sources:** `FRI-05.4 D-03` through `D-12`; root/leaf/block portions of `FRI-05.5` through `FRI-05.8`.

**Prerequisites:** `FRI-05-C02` complete.

**Entry state:** Canonical geometry is available but root, leaf, and block paths still use local accumulation, mutable scrollbar facts, or incomplete output.

**Bounded outcome:** Integrate the canonical factory and shared accumulator in root, leaf, and block;
implement stable/auto/both-edge reservation and the monotone auto-gutter fixed point; close small boxes, negative margins, partial axes, current lines/floats/absolute children, output helpers, rounding, and final-cache publication;
remove replaced root/block geometry paths while retaining the shared output field until its remaining producers migrate.

**Observable exit evidence:** Root/leaf/block front doors prove coherent
geometry, computed block formatting behavior, nested propagation/trapping,
negative-margin and tiny-box closure, auto induction, range, helper agreement, cache equivalence, and no panic or unsupported result.
Root/block legacy facts and constructors are absent; shared output removal remains assigned to C05.

**Handoff:** Shared block-side behavior is stable for flex and grid consumers.

### `FRI-05-C04` Flex Scroll Geometry And Main-Axis Semantics

**Specification sources:** `FRI-05.4 D-01`, `D-06` through `D-11`;
`FRI-05.7` flex contract; flex portions of `FRI-05.8` and `FRI-05.9`.

**Prerequisites:** `FRI-05-C03` complete.

**Entry state:** Flex lacks canonical container output and retained child geometry, and both automatic-minimum callers can observe the wrong overflow axis.

**Bounded outcome:** Route both automatic-minimum callers through the completed
flex main physical axis, integrate pass-local reservation and canonical output,
retain in-flow/current absolute child geometry, apply shared contribution and
trapping, and derive reverse/wrap-reverse origins plus final content-distribution
subjects.

**Observable exit evidence:** Main-only versus cross-only overflow is distinct
for every flex direction and leaf flow mapping; nested and zero-area geometry,
auto coupling, reverse signed ranges, alignment-origin bounds, rounding, and
cached/uncached output pass through the real flex front door.

**Handoff:** Flex owns no separate scroll path; grid-family integration remains.

### `FRI-05-C05` Grid Family Scroll Geometry

**Specification sources:** `FRI-05.4 D-01`, `D-03`, `D-04`, `D-06` through `D-11`; `FRI-05.5`; grid portions of `FRI-05.7` through `FRI-05.9`.

**Prerequisites:** `FRI-05-C04` complete.

**Entry state:** Ordinary grid, subgrid, and grid-lanes still omit canonical
container geometry, use incomplete automatic/intrinsic overflow decisions, or
lose container-relative contribution origins.

**Bounded outcome:** Integrate reservation, retained child/current absolute
geometry, shared accumulation, canonical output, computed auto minimum,
flow-aware used-overflow intrinsic trapping, final track subjects, zero-axis
contribution, and container-relative origins. After the last producers migrate,
remove the shared mutable scrollbar output and all remaining compatibility projections.

**Observable exit evidence:** All five overflow values, replaced hidden,
ordinary/intrinsic-subgrid/lanes callers, every leaf flow mapping, nested
propagation/trapping, zero-area descendants, non-zero item origins, auto
coupling, signed ranges, rounding, and cache equivalence pass through production
grid front doors. `NodeOutputOf` exposes only canonical derived content/gutter
accessors, and no legacy output field or construction bridge remains.

**Handoff:** All owned formatting contexts emit only the shared geometry contract.

### `FRI-05-C06` Bounded Fixtures Comparator And Final Regeneration

**Specification sources:** `FRI-05.4 D-12`, `D-13`; fixture/comparator portions
of `FRI-05.8` and `FRI-05.9`; `FRI-05.11`.

**Prerequisites:** `FRI-05-C05` complete and all production and generation-input
decisions stable.

**Entry state:** Product behavior is implemented; the bounded legacy parser transition remains, while the exact computed-style fixture lowering, eleven sources, manifest records, comparator activation, and final derived browser corpus are absent.

**Bounded outcome:** Remove the legacy pair transition; add only the bounded computed-style fixture lowering, serializer/parser support, eleven active HTML sources and manifest records,
range-span comparator diagnostics, and frozen bucket/hash contract. After every
input settles, perform the one full regeneration and retain its canonical XML,
report, and provenance.

**Observable exit evidence:** The frozen manifest and full report agree at
5,324 generated, 356 unsupported, and zero failure classes; all eleven sources
produce four variants; comparator negative controls fail correctly; focused
parity and read-only corpus checks pass with no scoped report or hand edit.

**Handoff:** Generator inputs and derived artifacts are frozen for read-only
candidate closure.

### `FRI-05-C07` Public Evidence And Leaf Candidate Closure

**Specification sources:** `FRI-05.5`, `FRI-05.8` through `FRI-05.10`,
`FRI-05.12`, `FRI-05.14`, and `FRI-05.15`.

**Prerequisites:** `FRI-05-C06` complete with its valid final regeneration.

**Entry state:** Behavior and browser artifacts are complete, while aggregate
exports, documentation, traceability, removed-surface proof, and candidate
handoff remain unreconciled.

**Bounded outcome:** Reconcile public reexports, crate and parity documentation,
compile/static surface evidence, all ten finding rows, complete verification,
and the exact CSS/style/root integration handoff. Do not change generation
inputs, XML, or reports.

**Observable exit evidence:** Every initiative acceptance item is traceable to
current source, tests, docs, and artifacts; configured default/generator gates,
focused parity, corpus validation, unsafe absence, and diff/provenance review are
clean; no forbidden legacy or scope expansion remains.

**Handoff:** Publish the reviewed leaf candidate to remote `main`, verify remote
readback, and return its exact SHA plus the breaking root integration contract.

## Sequence Completion

The sequence is complete when `FRI-05-C01` through `FRI-05-C07` satisfy their
observable exits in order and every `FRI-05.15` criterion is traceable. A later
cycle may not begin before the preceding candidate is published and remotely
verified. A material specification change returns to specification review
before this sequence is revised.
