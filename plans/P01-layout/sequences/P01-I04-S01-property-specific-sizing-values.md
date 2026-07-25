# P01-I04-S01 Property-Specific Sizing Values Implementation Sequence

Sequence ID: `P01/I04/S01`

Owning repository: `surgeist-layout`

## 1 Authority

This sequence implements the independently reviewed specification
`plans/P01-layout/initiatives/P01-I04-property-specific-sizing-values.md`
at content SHA-256
`601f4ad4700827465096dc62c029b4d8147336b8593b6b80ae7abfb09fc22577`,
committed as `49ede2ba2672a91f99ba193651dbb1350ede7b80`.

The specification is authoritative for behavior, API, ownership, artifact
counts, non-goals, and acceptance. This sequence owns only the ordered durable
implementation boundaries. Only the next ready cycle receives a detailed
just-in-time plan.

## 2 Sequence Boundary

All cycles mutate only `surgeist-layout`. Root adapters, facade exports, API
artifacts, and the root gitlink remain root-owned handoff work. No cycle adds a
dependency, feature, MSRV change, unsafe code, generator architecture, or
general CSS parser.

Scoped generation may be used as an optional diagnostic while an owned fixture
bug is being diagnosed. It is never entry or exit evidence. Only `C05` changes
generation inputs and performs the one final full regeneration after those
inputs settle; later cycles use read-only artifact checks unless a corrected
input invalidates and replaces that run.

## 3 Ordered Cycles

### 3.1 `P01/I04/S01/C01` Validated Sizing Calculation Model

**Specification sources:** `FRI-04.4 D-02`, `D-03`; `FRI-04.5` calculation
surface; `FRI-04.6` calculation status matrix; `FRI-04.8` calculation evidence.

**Prerequisites:** Published FRI-03 candidate and reviewed FRI-04 specification.

**Entry state:** The crate has only affine `LengthPercentageOf`; no iterative
sizing calculation or calc-size calculation model exists.

**Bounded outcome:** Add the private iterative validated program substrate and
the ordinary and calc-size calculation values, calculation-shape and finite
coefficient errors, exact syntactic missing-basis semantics, calc-size
percentage semantics, and both scalar-lane model evidence. Keep the new model
disconnected from property bases, `NodeInputOf`, and production algorithms.

**Observable exit evidence:** Focused tests prove every calculation shape,
invalid shape/coefficient construction, deep iterative evaluation/drop, missing
and definite bases, overflow, signed zero, and calc-size size/percentage
coefficients. Existing public behavior and browser artifacts are unchanged.

**Handoff:** A reviewed calculation substrate is ready for property wrappers
and the coherent public field migration.

### 3.2 `P01/I04/S01/C02` Property Domains And Public Field Migration

**Specification sources:** `FRI-04.4 D-01`, property-basis construction in
`D-03`, `D-04`, `D-05`; `FRI-04.5` public contract; `FRI-04.6`
construction/default matrices; `FRI-04.9` value, node, front-door, and parser
outlines.

**Prerequisites:** `P01/I04/S01/C01` complete.

**Entry state:** Calculation values exist, but `DimensionOf` still owns box,
flex, and broad track conversion paths.

**Bounded outcome:** Introduce the four closed property wrappers and
property-specific calc-size bases, enforce each property constructor's
`Any`/`size` restriction, validate track flex factors, revise track breadths,
migrate `NodeInputOf`, every crate construction, and the existing fixture
adapter's simple-value paths, then remove `Dimension`/`DimensionOf`, dangerous
conversions, and legacy reexports in one coherent public break.

**Observable exit evidence:** The crate and existing corpus compile and pass
with preferred/min/flex `Auto`, maximum `None`, property-specific calc-size
bases and `Any` rejection, track-only validated flex, no legacy dimension
surface, no invalid cross-property constructor, and no generated artifact delta.

**Handoff:** Every production consumer receives an explicit property domain;
calculation-aware algorithm resolution can proceed without a compatibility path.

### 3.3 `P01/I04/S01/C03` Numeric Calculation Consumption

**Specification sources:** `FRI-04.4 D-02`, `D-04`, `D-05`; the supported
numeric rows of `D-06`; `FRI-04.6` missing-basis and status matrices;
`FRI-04.8` front-door and track evidence.

**Prerequisites:** `P01/I04/S01/C02` complete.

**Entry state:** Algorithms consume property-specific values but only the
legacy-equivalent simple affine behavior is proven.

**Bounded outcome:** Resolve affine/min/max/clamp calculations at every current
leaf, root, block, flex, grid, grid-lanes, positioned, and track call site;
apply the non-negative used range; preserve property-specific missing-basis
rules; and retain invalid numeric failures without `NonNumeric` dispatch.

**Observable exit evidence:** Real front-door and scalar tests cover every
numeric row, nested functions, negative results, missing basis, overflow, track
cyclic handling, and fit-content calculation limits. Existing browser artifacts
remain unchanged and readable.

**Handoff:** Numeric calculation behavior is complete; only contextual
keyword/calc-size dispatch and later-owner capability routing remain.

### 3.4 `P01/I04/S01/C04` Contextual And Calc-Size Dispatch

**Specification sources:** `FRI-04.4 D-03`, `D-05`, `D-06`; `FRI-04.5` error
front door; `FRI-04.6` calc-size status; `FRI-04.8` capability and flex evidence.

**Prerequisites:** `P01/I04/S01/C03` complete.

**Entry state:** Numeric calculations are correct, while contextual keywords,
flex content semantics, calc-size bases, and later-owner results are not yet
exhaustively connected.

**Bounded outcome:** Implement calc-size `Any` and `FullPercentage`, supported
preferred intrinsic routes, flex `Auto` versus `Content`, and the closed
property/behavior/algorithm/axis capability payload for every unsupported D-06
cell. Remove all ordinary sizing dependence on `NonNumeric` and all silent
auto/max-content fallback.

**Observable exit evidence:** Table-driven tests cover every D-06 cell and exact
payload, supported routes return geometry, flex intrinsic bases remain distinct,
and later finding ownership is preserved without browser unsupported expansion.

**Handoff:** Production sizing semantics and capability boundaries are stable;
fixture capture can target only the supported FRI-04 surface.

### 3.5 `P01/I04/S01/C05` Bounded Fixtures And Final Derivation

**Specification sources:** `FRI-04.4 D-07`; `FRI-04.7`; fixture/parser/generator
parts of `FRI-04.8` and `FRI-04.9`.

**Prerequisites:** `P01/I04/S01/C04` complete and all production/parser value
decisions stable.

**Entry state:** The supported behavior is implemented, but helper capture,
serializer support, the three HTML sources, and derived browser evidence are
absent.

**Bounded outcome:** Add only the finite property-specific fixture grammar,
helper/serializer preservation, parser invalid-state diagnostics, three active
HTML sources, exact inventory tests, and the canonical unsupported-projection
test. After every generation input settles, perform one full ExistingPinned
regeneration and retain its complete derived corpus and provenance.

**Observable exit evidence:** The repository contains 1,409 HTML and 5,280 XML,
the 12 owned outputs pass parity, all 5,280 outputs carry current provenance,
the sole report has 356 canonically unchanged unsupported tuples and zero
failure classes, and all artifact checks after regeneration are read-only.

**Handoff:** The supported public and browser surfaces are stable with final
derived artifacts; documentation and aggregate initiative evidence can close.

### 3.6 `P01/I04/S01/C06` Public Evidence And Candidate Closure

**Specification sources:** `FRI-04.5`, `FRI-04.8`, `FRI-04.9`, `FRI-04.10`,
and `FRI-04.12`.

**Prerequisites:** `P01/I04/S01/C05` complete with the final generated inventory.

**Entry state:** Product behavior and artifacts are complete, but aggregate
public-contract, documentation, traceability, and root-handoff evidence has not
yet been reconciled as one initiative result.

**Bounded outcome:** Reconcile reexports, crate and fixture documentation,
public examples, compile-time surface evidence, all focused matrices,
`MODEL-005`/`MODEL-007` traceability, and the finite root integration handoff.
Do not change generation inputs or derived XML.

**Observable exit evidence:** Every FRI-04 acceptance item is backed by current
source/tests/docs/artifacts, configured default and generator feature matrices
are green, full corpus/parity checks are green, no prohibited legacy or scope
expansion remains, and the leaf candidate report contains the exact breaking
API and root-lowering obligations.

**Handoff:** FRI-04 is a complete leaf candidate. `FRI-05` is the next ready
initiative in the findings-resolution index; root promotion remains a separate
root-owned cycle.

## 4 Sequence Completion

The sequence is complete when `P01/I04/S01/C01` through `P01/I04/S01/C06` satisfy their
observable exits in order and the specification's initiative acceptance is
fully traceable. A later cycle may not begin early merely because its files are
convenient to edit. A material specification change returns to specification
review before this sequence is revised.
