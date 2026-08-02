# P01-I06-S01 Inline Formatting, Floats, And BFCs Implementation Sequence

Sequence ID: `P01/I06/S01`

Owning repository: `surgeist-layout`

## 1 Authority

This sequence implements the independently reviewed specification at
`plans/P01-layout/initiatives/P01-I06-inline-formatting-floats-bfcs.md`,
normalized semantic-content SHA-256
`4d383feef44dd58c53ea14ff9fd380effde9b42d5165fa3d8b1911ad56f5ab47`,
committed as `82b163d93edafa0f2b1ee42f6f7c273de876829d`.

The specification owns behavior, API, compatibility, ownership, artifacts,
errors, and acceptance. This sequence owns only durable dependency order. Only
the next ready cycle receives a detailed just-in-time plan.

## 2 Sequence Boundary

Every cycle mutates only `surgeist-layout`. Root composition, text shaping and
source association, shape geometry, authored CSS/style lowering, facade/API
artifacts, and gitlink promotion remain the separate handoff in `FRI-06.12`.

No cycle adds a dependency, feature, MSRV change, unsafe code, generator
architecture, general text shaper/parser, general CSS parser, bidi analyzer,
shape engine, rendering state, or behavior owned by FRI-09 through FRI-13.
FRI-01's unit cache key remains unchanged. Transactional invalidation uses only
the reviewed FRI-06 entry point, exact ancestor closure, and two-phase batch
application.

Scoped generation remains optional diagnostic work, never completion evidence,
and is repeated only when changed inputs/code or a failed evidence capture gives
it a concrete diagnostic purpose. C10 synthesis and C12's first structurally
successful full run are characterization only. C12 replaces
name/expectation-driven lowering with the closed explicit fixture-input contract.
Chrome and the browser helper own HTML parsing and actual-DOM marker validation;
Rust must not reconstruct HTML, CSS, or DOM topology and may account only for
helper-reported source-local marker use. C12 realizes only production work proven
by corrected-input diagnostics, records exact known Chrome failures or `None`,
and owns the only final full unfiltered existing-pinned lineage after inputs
settle.

## 3 Activation Recovery Evidence

The C10 diagnostic and C11 recovery boundaries use the exact activation,
fixture-correction, production-correction, baseline-helper,
semantic-preservation, and base-generated matrices retained in
`plans/P01-layout/P01-I06-S01-C10-public-comparison-census.tsv`, SHA-256
`0630d2606f1e53c56b69cd226665b899bbfd96ed60ad7ac3c80ec5d9423b5691`,
the diagnostic report narrative in
`plans/P01-layout/P01-I06-S01-C10-post-generation-census.md`, SHA-256
`2c4179f559c5fa9e93c6933e0ba1a4969b758fc4a4f738d619c2751796b8bf00`,
and the corrected recovery census in
`plans/P01-layout/P01-I06-S01-C10-second-lineage-census.md`, SHA-256
`a56b09ed4d68ee901dbc385db3d78b66bf5faeb82f844f1d531c94aef10a23b9`.
Those fixed predicates own recovery membership; no later cycle may add, omit,
reclassify, or dynamically widen a row. Aggregate failures remain outside this
initiative until the index-only release conformance gate receives its own plan.

## 4 Ordered Cycles

### 4.1 `P01/I06/S01/C01` Public Inline Model And Transaction Substrate

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

### 4.2 `P01/I06/S01/C02` Unified Shaped-Text Line Construction

**Specification sources:** `FRI-06.4 D-01` through `D-10`; shaped text and
fragment portions of `FRI-06.5`; text, break, bidi, whitespace, alignment, and
intrinsic portions of `FRI-06.7` through `FRI-06.9`.

**Prerequisites:** `P01/I06/S01/C01` complete and remotely verified.

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
existing control integration remain in C04.

**Observable exit evidence:** Both scalar lanes prove deterministic wrapping,
replacement ownership, rejected replacement/discard state, mandatory/final
break behavior, overwide progress, all-discarded anchoring, visual-slot gaps,
source-order batch identity, all flow mappings, per-line alignment, intrinsic
sizes, rounding, scroll contribution, and cold/warm fragment restoration without
synthetic measured text.

**Handoff:** One reviewed line source can accept atomic boxes, controls, and
float-adjusted bands without another axis algorithm.

### 4.3 `P01/I06/S01/C03` Post-C02 Sprawl Containment

**Specification sources:** Retained MR01 contract
`plans/P01-layout/P01-I06-S01-C03-post-c02-sprawl-containment.md`, SHA-256
`0c88ec011067e25d61d0ddfdf90ad47e9e5db0149dbdc214e8326668665711e4`,
clauses `FRI-06-MR01.1` through `.7`; compatible `FRI-06.4 D-02` and
validation/transaction/test-support portions of `FRI-06.5`, `.6`, and `.10`.
**Prerequisites:** `P01/I06/S01/C02` complete and remotely verified.
**Entry state:** The legacy C02-to-C03 window maps to canonical C02, inserted
C03, then former C03 at C04; equivalent non-box validation and scalar-lane
oracle-tree implementations remain duplicated.
**Bounded outcome:** Contain the concrete post-C02 duplication and
multi-responsibility hotspots without changing public behavior, fixture inputs,
generator architecture, or the next algorithm boundary.
**Observable exit evidence:** Every MR01 acceptance clause passes with no
observable behavior, fixture, artifact, or public-surface change.
**Handoff:** The unified line source remains the sole basis for C04.

### 4.4 `P01/I06/S01/C04` Mixed Atomic And Control Line Completion

**Specification sources:** `FRI-06.4 D-03`, `D-06` through `D-13`; atomic,
control, baseline, percentage, and non-box portions of `FRI-06.5` through
`FRI-06.9`.

**Prerequisites:** `P01/I06/S01/C03` complete and remotely verified.

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

### 4.5 `P01/I06/S01/C05` Rectangular Float And BFC Geometry

**Specification sources:** `FRI-06.4 D-13` and `D-15`; rectangular float, clear,
BFC, sizing, baseline, scroll, and cache portions of `FRI-06.7` through
`FRI-06.9`.

**Prerequisites:** `P01/I06/S01/C04` complete and remotely verified.

**Entry state:** Inline lines are complete against a full containing band, while
ordinary lines can overlap floats and current float/BFC placement is physical-
horizontal, incomplete for auto size/height, and not closed over current display
and overflow roles.

**Bounded outcome:** Make rectangular exclusions flow-relative and source
ordered; place line-left/right, opposing, stacked, cleared, and overwide floats;
query full line spans monotonically; map float/clear through containing flow;
enclose owned floats in auto block size; trap nested floats; and implement the
exact current flex/grid/grid-lanes plus non-replaced-overflow BFC avoidance and
auto-inline-size predicate. Keep provider-backed shapes for C06.

**Observable exit evidence:** Every float/clear side and flow mapping, mixed line
exclusion, finite-transition progress, overwide behavior, float-only auto height,
nested containment, ordinary block edge, current BFC predicate, auto/definite
width, intrinsic behavior, scroll contribution, rounding, and invalidation path
has production front-door proof with no overlap or second side table.

**Handoff:** Margin-box exclusion is complete and the bounded provider can refine
the same band query without changing placement ownership.

### 4.6 `P01/I06/S01/C06` Provider-Backed Shape Exclusion

**Specification sources:** `FRI-06.4 D-14`; shape provider, error, band, cache,
fake, and root-handoff portions of `FRI-06.5` through `FRI-06.12`.

**Prerequisites:** `P01/I06/S01/C05` complete and remotely verified.

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

**Handoff:** The focused C06 production baseline and fixture-facing facts are
stable for adapter preparation. Activated-fixture validation and any confirmed
production correction remain explicitly owned by C09.

### 4.7 `P01/I06/S01/C07` Post-C05 Sprawl Containment

**Specification sources:** Retained MR02 contract
`plans/P01-layout/P01-I06-S01-C07-post-c05-sprawl-containment.md`, SHA-256
`3e9d894791ffc3fa9ce772350a7fd9d667979dc5d86c37affebc2922b8c1322d`,
clauses `FRI-06-MR02.1` through `.9`; compatible `FRI-06.4 D-01`, `D-14` and
internal line/geometry/error/scalar portions of `FRI-06.5` through `.10`.
**Prerequisites:** `P01/I06/S01/C06` complete and remotely verified.
**Entry state:** The legacy C05-to-C06 window maps to canonical C06, inserted
C07, then former C06 at C08; proven-equivalent line scans and private
scalar/geometry helpers remain duplicated.
**Bounded outcome:** Apply the reviewed post-C05 mechanical containment before
fixture work, preserving the provider, cache, transaction, and public geometry
contracts without behavior expansion.
**Observable exit evidence:** Every MR02 acceptance clause passes with no
observable behavior, fixture, artifact, or public-surface change.
**Handoff:** Fixture adapter preparation starts from the contained C06 result.

### 4.8 `P01/I06/S01/C08` Finite Fixture Adapter Preparation

**Specification sources:** `FRI-06.4 D-16`; browser/comparator portions of
`FRI-06.9` and `FRI-06.10`; `FRI-06.11`; artifact portions of `FRI-06.14`.

**Prerequisites:** `P01/I06/S01/C07` complete and remotely verified, with
fixture and generator inputs unchanged.

**Entry state:** Product behavior and finite production fixture-facing facts have
focused C06 evidence but have not yet faced the activation matrix; C09 owns that
validation. The browser adapter cannot compare control/fragment output or lower
shaped text, atomic participation, bottom alignment, and finite shape-band
tables. Generation inputs and derived artifacts remain frozen. Final
generator-feature verification also exposed one pre-existing lifecycle test that
still models implicit file close instead of the generator's explicit lease-release
front door.

**Bounded outcome:** Extend only the Rust fixture adapter and comparator to parse,
lower, store, and compare the reviewed control, fragment, shaped-text, atomic,
bottom-alignment, and finite shape-band facts through production constructors and
front doors. Correct only that stale generator lifecycle test to use the existing
explicit release front door. Do not edit generator implementation, helper, HTML,
manifest, XML, report, or production source and do not run generation.

**Observable exit evidence:** Strict negative controls detect every named
geometry, identity, schema, role, query, and provider mismatch. Valid controls,
fragments, shaped/atomic input, bottom alignment, and empty/partial/full shape
bands agree through the real fixture-tree and production front doors. Default and
generator-feature verification pass with generation inputs and artifacts
unchanged, and the lease lifecycle test covers explicit release and reacquisition
without relying on implicit file-close timing.

**Handoff:** The finite adapter is stable. Browser activation may now diagnose
whether the frozen production handoff and proposed fixture inputs satisfy all 388
owned comparisons before any valid final derivation.

### 4.9 `P01/I06/S01/C09` Activated-Fixture Production Corrections

**Specification sources:** `FRI-06.4 D-03`, `D-06`, `D-07`, `D-10`, `D-12`, and
`D-13`; line, intrinsic, bidi, control, flow, and clear portions of
`FRI-06.7` through `FRI-06.9`; behavioral portions of `FRI-06.14`.

**Prerequisites:** `P01/I06/S01/C08` complete and remotely verified. Its invalid
diagnostic derivation is discarded and never counts as artifact lineage.

**Entry state:** Focused activation diagnostics and exact-base typed probes prove
52 valid FRI-06 variants across 21 sources still miss reviewed production
behavior: 12 intrinsic/track height results and 40 physical/logical placements.
The fixed baseline-helper subset instead requires C10 correction before lowering,
and all variants of `html/block/fri06_forced_break_strut.html` require only C10
fixture activation. Their typed production oracles already pass. The 408 pre-existing aggregate failures
remain outside this cycle, the final aggregate gate remains FRI-13-owned, and all
fixture-input failures remain C10-owned.

**Bounded outcome:** Correct only the confirmed FRI-06 production paths reached
by those 21 sources, preserving exact shaped-participant, intrinsic, bidi,
writing-mode, RTL, control, and logical-clear semantics through the
public compute front door. Do not edit fixture inputs or artifacts, run
generation, absorb inherited aggregate failures, or enter later-owned behavior.

**Observable exit evidence:** Exact source-level regressions prove all 52
validated production cases in both applicable directions and box models. Default
and generator-feature verification, formatting, Clippy, unsafe absence, and
unrelated focused regression suites are clean without fixture or artifact deltas.

**Handoff:** Production behavior required by the 388 owned fixture variants is
stable before helper, source, manifest, and final-lineage work resumes.

### 4.10 `P01/I06/S01/C10` Fixture Activation Diagnostic Boundary

**Specification sources:** `FRI-06.4 D-01`, `D-04`, `D-09`, `D-11`, `D-13`,
`D-16`; applicable `FRI-06.5`, `.7`, `.9` through `.11`, and `.14`.
**Prerequisites:** `P01/I06/S01/C09` complete and remotely verified.
**Entry state:** Production and finite-adapter contracts are available; the fixed 388-row activation union and generated lineage remain unresolved.
**Bounded outcome:** Establish the diagnostic recovery boundary for the fixed
activation union. No output grants completion or publication authority.
**Observable exit evidence:** The pinned recovery artifacts preserve exact
membership, baseline, and diagnostic partitions without serving acceptance.
**Handoff:** C11 receives the exact pinned recovery membership.

### 4.11 `P01/I06/S01/C11` Fixture Input Recovery And Characterization
**Specification sources:** `FRI-06.4 D-01`, `D-04`, `D-09` through `D-11`, and
`D-16`; metric-fragment, atomic-baseline, physical-placement, and browser-
comparator portions of `FRI-06.5`, `FRI-06.7`, `FRI-06.9`, `FRI-06.10`, and
`FRI-06.14`; `FRI-06.11`.
**Prerequisites:** `P01/I06/S01/C10` diagnostic handoff; the C09 production and
finite-adapter decisions remain authoritative.
**Entry state:** The pinned recovery census identifies unresolved fixture-input
categories and ten production rows; all generated output remains diagnostic.
**Bounded outcome:** Produce one settled input contract and exact remaining
production characterization without generation.
**Observable exit evidence:** The reviewed input freeze and characterization
cover the complete pinned recovery membership.
**Handoff:** C12 receives the settled input and production boundary.

### 4.12 `P01/I06/S01/C12` Final Production Correction And Lineage
**Specification sources:** `FRI-06.4 D-01`, `D-04`, `D-06`, `D-07`, `D-09`,
`D-11`, `D-12`, `D-13`, and `D-16`; line, metric-fragment, atomic-baseline,
physical-placement, comparator, fixture, and acceptance portions of `FRI-06.5`,
`FRI-06.7`, `FRI-06.9` through `FRI-06.11`, and `FRI-06.14`.
**Prerequisites:** `P01/I06/S01/C11` complete and remotely verified.
**Entry state:** C11's assumed input freeze was invalidated by the honest 5,712
full-run diagnostic: 314/388 activation rows fail, including 244 rows whose
adapted HTML lost pinned WPT default-block roles, while direction-derived bidi
levels and comparator-local identities are not final input evidence.
**Bounded outcome:** Preserve that run as diagnostic, restore the closed
fixture/comparator facts, and use scoped diagnostics only to realize exact
remaining production tasks. Freeze all inputs and production behavior before one
final full existing-pinned generation. A scoped repeat has a concrete diagnostic
cause in changed inputs/code or a failed evidence capture; generator architecture
change, speculative task, and later-owned behavior remain prohibited.
**Observable exit evidence:** Report is 5,712/16 and `filter: null`; expected-fail
inventory exactly equals the reviewed plan registry, normally zero; every 388 row
is browser-pass or has the reviewed synthetic substitute; the other 5,324 bodies
preserve semantics; no scoped report remains and FRI-13 stays unclaimed.
**Handoff:** C13 receives the published behavior-correct candidate with generator
inputs and outputs frozen; the successful assumption-failing run is not lineage.

### 4.13 `P01/I06/S01/C13` Validated Mechanical Consolidation
**Specification sources:** module-responsibility and test portions of
`FRI-06.9` and `FRI-06.10`; internal-discipline and acceptance portions of
`FRI-06.14`.
**Prerequisites:** `P01/I06/S01/C12` complete, published, remotely verified, and
handed off with its exact candidate and artifact hashes.
**Entry state:** The immutable C12 candidate closes FRI-06 behavior and lineage;
`plans/P01-layout/P01-I06-mechanical-refactoring-review-findings.md` contains six
earlier structural opportunities that must be revalidated against that result.
**Bounded outcome:** Validate `MR-001` through `MR-006`; implement every
still-applicable opportunity as behavior-preserving internal consolidation and
record exact source evidence for every disproven item. Preserve public API,
behavior, dependencies, features, fixtures, generator logic, generated artifacts,
and all 59 finding-owner assignments. Do not close an FRI-07 finding.
**Observable exit evidence:** Every MR item has one validated disposition;
characterization and applicable scaling evidence preserve the published C12
behavior and artifact hashes; configured checks and independent reviews are
clean; the final FRI-06 candidate is published and remotely verified.
**Handoff:** Return the final FRI-06 leaf candidate and complete MR disposition to
the P01/root coordinator before FRI-07 planning begins.

## 5 Sequence Completion

This realized sequence ends at `P01/I06/S01/C13`. Every listed cycle satisfies
its observable exit in order. No later cycle is represented. A material
specification change returns to specification review before this sequence is
revised.
