# FRI-06 Inline Formatting, Floats, And BFCs Implementation Sequence

Status: draft

Sequence ID: `FRI-06`

Owning repository: `surgeist-layout`

## Authority

This sequence implements the independently reviewed specification at
`plans/specs/2026-07-17-surgeist-layout-fri-06-inline-formatting-floats-bfcs.md`,
normalized SHA-256
`98681bac979f68fa3bc380c349f7a04110f9a1f13d142625751fa9cbc5f1ffaf`,
committed as `cc2a8486f9e4e7719c9a28cc68321b7e630d9ded`.

The specification owns behavior, API, compatibility, ownership, artifacts,
errors, and acceptance. This sequence owns only durable dependency order. Only
the next ready cycle receives a detailed just-in-time plan.

## Sequence Boundary

Every cycle mutates only `surgeist-layout`. Root composition, text shaping and
source association, shape geometry, authored CSS/style lowering, facade/API
artifacts, and gitlink promotion remain the separate handoff in `FRI-06.12`.

No cycle adds a dependency, feature, MSRV change, unsafe code, generator
architecture, general text shaper/parser, general CSS parser, bidi analyzer,
shape engine, rendering state, or behavior owned by FRI-09 through FRI-13.
FRI-01's unit cache key remains unchanged. Transactional invalidation uses only
the reviewed FRI-06 entry point, exact ancestor closure, and two-phase batch
application.

Scoped generation remains optional diagnostic work rather than completion
evidence. The artifact cycle exclusively owns the bounded HTML/parser/helper,
manifest, XML, and report changes. Its final artifacts have one full unfiltered
existing-pinned no-fetch lineage after all owned inputs settle. Later closure is
read-only for generation inputs and outputs; a confirmed input defect returns to
that artifact cycle and invalidates its prior lineage.

## Ordered Cycles

### `FRI-06-C01` Public Inline Model And Transaction Substrate

**Specification sources:** `FRI-06.4`; `FRI-06.5`; cache, validation, and error
portions of `FRI-06.6`; module and compatibility portions of `FRI-06.10`.

**Prerequisites:** Published FRI-05 candidate and the clean reviewed FRI-06
specification revision recorded above.

**Entry state:** Layout has no shaped-text participant or fragment output,
non-box tree pairing, shape-exclusion model, exact invalidation closure, or
transactional fragment/cache application surface.

**Bounded outcome:** Add the private-field scalar-generic shaped segment, bidi,
break, whitespace, text, atomic participation, fragment, float-exclusion, and
provider-query values; canonical non-box pairing; reviewed errors and public
reexports; phase-specific fragment batch output; committed-fragment readback;
unit-key-preserving invalidated layout entry; and immutable-prepare/infallible-
commit batch application. Do not replace the line algorithm or invoke the shape
provider yet.

**Observable exit evidence:** Public construction admits only reviewed states in
both scalar lanes; contradictory non-box/atomic/shape roles and unreachable
dirty subjects fail with exact diagnostics; duplicate dirty subjects normalize;
inclusive root-path closure, stale-hit bypass, failed layout/preparation, ordered
commit, empty/nonempty fragment restoration, and missing warm fragment state all
match the transaction contract without a cache revision token.

**Handoff:** Later algorithms can consume validated participants and publish
fragments without redesigning public state, errors, invalidation, or commit
atomicity.

### `FRI-06-C02` Unified Shaped-Text Line Construction

**Specification sources:** `FRI-06.4 D-01` through `D-10`; shaped text and
fragment portions of `FRI-06.5`; text, break, bidi, whitespace, alignment, and
intrinsic portions of `FRI-06.7` through `FRI-06.9`.

**Prerequisites:** `FRI-06-C01` complete and remotely verified.

**Entry state:** Validated text and fragment phases exist, while production
inline layout remains box/control-only, splits horizontal and vertical behavior,
and lacks soft wrapping, source-associated text fragments, and per-line visual
ordering.

**Bounded outcome:** Replace text-only line construction with one logical-axis
builder that consumes shaped segments, performs greedy allowed/replacement/
mandatory breaking, edge-whitespace handling, min/max-content contribution,
per-line band and legacy alignment, complete-unit bidi ordering, physical
projection for all ten flows, and phase-correct text-node/fragment publication.
The full-band interface remains ready for later float exclusion; atomic and
existing control integration remain in C03.

**Observable exit evidence:** Both scalar lanes prove deterministic wrapping,
replacement ownership, rejected replacement/discard state, mandatory/final
break behavior, overwide progress, all-discarded anchoring, visual-slot gaps,
source-order batch identity, all flow mappings, per-line alignment, intrinsic
sizes, rounding, scroll contribution, and cold/warm fragment restoration without
synthetic measured text.

**Handoff:** One reviewed line source can accept atomic boxes, controls, and
float-adjusted bands without another axis algorithm.

### `FRI-06-C03` Mixed Atomic And Control Line Completion

**Specification sources:** `FRI-06.4 D-03`, `D-06` through `D-13`; atomic,
control, baseline, percentage, and non-box portions of `FRI-06.5` through
`FRI-06.9`.

**Prerequisites:** `FRI-06-C02` complete and remotely verified.

**Entry state:** Shaped text lines work through the unified builder, while atomic
boxes, line breaks, boundaries, top/bottom alignment, percentage block basis,
vertical clear, and fixed-size baseline paths still use incomplete legacy
composition.

**Bounded outcome:** Compose text, atomic boxes, visible/hidden breaks, boundary
markers, floats/out-of-flow placeholders, and source association through the one
participant stream. Complete mixed bidi slots, struts and empty lines,
per-control geometry/comparison, logical clear in all flows, inner/fallback
atomic baselines, top/bottom placement, definite percentage block basis, and
metric-aware fixed-size behavior. Remove the separate vertical and obsolete
whole-run shortcuts owned by FRI-06.

**Observable exit evidence:** Mixed text-atomic-control lines preserve exact
source and visual identity; leading/trailing/adjacent controls, empty lines,
vertical/sideways clear, unequal line alignment, baseline fallback/margins,
top/bottom expansion, replaced atomics, definite/indefinite percentages, and
fixed fast paths complete without panic, silent omission, or guessed facts.

**Handoff:** The complete inline participant engine is ready for production
float bands and BFC placement.

### `FRI-06-C04` Rectangular Float And BFC Geometry

**Specification sources:** `FRI-06.4 D-13` and `D-15`; rectangular float, clear,
BFC, sizing, baseline, scroll, and cache portions of `FRI-06.7` through
`FRI-06.9`.

**Prerequisites:** `FRI-06-C03` complete and remotely verified.

**Entry state:** Inline lines are complete against a full containing band, while
ordinary lines can overlap floats and current float/BFC placement is physical-
horizontal, incomplete for auto size/height, and not closed over current display
and overflow roles.

**Bounded outcome:** Make rectangular exclusions flow-relative and source
ordered; place line-left/right, opposing, stacked, cleared, and overwide floats;
query full line spans monotonically; map float/clear through containing flow;
enclose owned floats in auto block size; trap nested floats; and implement the
exact current flex/grid/grid-lanes plus non-replaced-overflow BFC avoidance and
auto-inline-size predicate. Keep provider-backed shapes for C05.

**Observable exit evidence:** Every float/clear side and flow mapping, mixed line
exclusion, finite-transition progress, overwide behavior, float-only auto height,
nested containment, ordinary block edge, current BFC predicate, auto/definite
width, intrinsic behavior, scroll contribution, rounding, and invalidation path
has production front-door proof with no overlap or second side table.

**Handoff:** Margin-box exclusion is complete and the bounded provider can refine
the same band query without changing placement ownership.

### `FRI-06-C05` Provider-Backed Shape Exclusion

**Specification sources:** `FRI-06.4 D-14`; shape provider, error, band, cache,
fake, and root-handoff portions of `FRI-06.5` through `FRI-06.12`.

**Prerequisites:** `FRI-06-C04` complete and remotely verified.

**Entry state:** Rectangular exclusion is complete, while `Shape` requests do
not yet invoke the reviewed tree provider or refine bands with typed empty,
partial, full, invalid, missing, and failed results.

**Bounded outcome:** Integrate the bounded physical band query into the existing
float exclusion pass, preserve exact container/float/band diagnostics, clip and
validate provider intervals, bound query repetition, and connect provider result
changes to the reviewed dirty-float ancestor transaction. Do not add shape
identity, shape parsing, a sibling dependency, or a general geometry engine.

**Observable exit evidence:** Empty, partial, full, clipped, invalid, missing,
and failed provider results; non-float shape rejection; query bounds; cache
invalidation; failed recomputation; and cold/warm/rounded geometry agree through
the real provider and block-line front doors in both scalar lanes.

**Handoff:** All FRI-06 production behavior and fixture-facing facts are stable
before browser adapter and artifact work.

### `FRI-06-C06` Bounded Fixture Activation And Final Lineage

**Specification sources:** `FRI-06.4 D-16`; browser/comparator portions of
`FRI-06.9` and `FRI-06.10`; `FRI-06.11`; artifact portions of `FRI-06.14`.

**Prerequisites:** `FRI-06-C05` complete and remotely verified; production and
all generation-input decisions stable.

**Entry state:** Product behavior is complete, while text/control fragments,
bottom alignment, finite shape bands, the exact 340 existing unsupported
variants, twelve new four-variant sources, comparator diagnostics, and final
derived artifacts are absent.

**Bounded outcome:** Add only the reviewed finite helper/parser/serializer facts,
activate all 85 existing FRI-06 sources, add exactly twelve named sources and 48
variants, compare control/fragment/source/line/visual/baseline geometry, and
derive the complete corpus once from settled inputs through the existing-pinned
no-fetch lineage. Generator architecture remains unchanged.

**Observable exit evidence:** The final full report has 5,712 generated, exactly
16 immutable missing-root unsupported variants, `filter: null`, reviewed browser
and helper/manifest provenance, and zero failure classes or scoped reports. All
388 activated/new variants compare; negative controls detect every named
geometry/identity mismatch; artifacts are derived rather than hand-edited.

**Handoff:** Generator inputs and outputs are frozen for read-only initiative
closure.

### `FRI-06-C07` Public Evidence And Leaf Candidate Closure

**Specification sources:** `FRI-06.10`; `FRI-06.12` through `FRI-06.14`.

**Prerequisites:** `FRI-06-C06` complete and remotely verified with its valid
final artifact lineage.

**Entry state:** Behavior and browser artifacts are complete, while aggregate
public docs/exports, compatibility inventory, finding trace, dead-code cleanup,
complete verification, and root/text/shape handoff remain unreconciled.

**Bounded outcome:** Reconcile the reviewed public front door and documentation,
remove every FRI-06-owned dead-code allowance and obsolete path, prove all 14
finding rows and initiative acceptance, and record the exact breaking leaf,
text/shape adapter, transactional invalidation, artifact, and root-promotion
handoff. Do not change generation inputs or outputs.

**Observable exit evidence:** Every acceptance item is traceable to current
source, focused and browser evidence, public/static negative surfaces,
transaction behavior, artifacts, docs, and compatibility accounting. Complete
default/generator verification, focused parity, corpus/Taffy checks, formatting,
warnings-denied Clippy, unsafe absence, and range/provenance review are clean.

**Handoff:** Publish the reviewed leaf candidate to remote `main`, verify remote
readback, and return its exact SHA plus the complete FRI-06 root/text/shape
integration contract.

## Sequence Completion

The sequence is complete when `FRI-06-C01` through `FRI-06-C07` satisfy their
observable exits in order and every `FRI-06.14` criterion is traceable. A later
cycle cannot begin before its predecessor is published and remotely verified.
A material specification change returns to specification review before this
sequence is revised.
