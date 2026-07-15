# FRI-03 Box Participation Contracts Implementation Sequence
## Authority

- Owning repository: `surgeist-layout`
- Reviewed specification: `plans/specs/2026-07-15-surgeist-layout-fri-03-box-participation-contracts.md`
- Specification SHA-256:
  `56efbca03febc725bee2d829da9bfdcf45f6194b24555eb22c1aa1082d9b12f2`
- Specification commit:
  `ad342c4526802460f89d6d02125f16e419b6f81b`
- Initiative index: `plans/specs/2026-07-11-surgeist-layout-findings-resolution-index.md`, `FRI-03`

This sequence orders complete implementation of `MODEL-001`, `CORE-005`, and
`BLOCK-007`. Detailed desired-state decisions remain owned by the reviewed
specification.

## Boundary

All cycles are owned by `surgeist-layout`; root and sibling repositories remain
outside mutation scope. Root integration is an archival handoff after the
reviewed leaf candidate; no cycle edits a root adapter, facade, gitlink, or API
artifact.

No cycle expands generator architecture. `C02` is the sole producer change:
exact order capture, exact flex-parent axes capture, three order sources,
bounded serializer/parser attributes, and mechanical derived-artifact refresh.
Later cycles only consume or verify that state. A
confirmed genuine bug discovered later requires explicit replanning before any
additional generator change.

Every cycle keeps item order separate from source identity, retains
source-indexed output/storage, and adds no compatibility alias or fallback. No
cycle changes dependencies, features, the lockfile, task-runner policy, browser
pin, launch profile, import provenance, MSRV, or the crate-wide `unsafe`
prohibition. Browser-backed work uses only the already-present pinned
executable under the canonical no-acquisition workflow.

## C01 - Exact Order And Source Identity

- Owner: `surgeist-layout`
- Outcome: Public scalar-independent `ItemOrder` and `SourceIndex` types replace
  the absent input and ambiguous output concepts; one crate-private stable
  `(item_order, source_index)` permutation is ready for later algorithms.
- Specification: `FRI-03.1`; order/source portions of `FRI-03.2`;
  `E-ORDER-INPUT`; `E-SOURCE-ID`; `D-01`; helper ownership in `D-02`; `D-06`;
  `FRI-03.5`; relevant `FRI-03.7` and `FRI-03.9`; acceptance items 1 and 2.
- Prerequisite: Published FRI-02 candidate at the sequence base.
- Entry state: `NodeInputOf` has no order value, and output/source carriers use
  ambiguous `order` naming for source sibling ordinal.
- Exit evidence: Exact signed/default order, total stable permutation, default
  zero input, typed source identity, unchanged default/source-order geometry,
  preserved hidden/root/standalone/rounded/batch source identity, and no legacy
  public or private source-order aliases.
- Handoff: `C02` may serialize/parse `ItemOrder`; `C05` through `C07` are the
  only geometry consumers of the permutation.

## C02 - Bounded Fixture Schema And Corpus Baseline

- Owner: `surgeist-layout`
- Outcome: The producer emits exact item order and flex-item viewport parent
  axes; the three order sources, four scoped reports, generated inventory, and
  provenance are final before public context signatures change.
- Specification: fixture/parser scope in `FRI-03.2`; `E-PARITY`; `FRI-03.8`;
  fixture paths in `FRI-03.9`; generator constraints in `FRI-03.11`;
  artifact portions of acceptance items 7 and 8.
- Prerequisite: `C01`.
- Entry state: Helper JSON and XML cannot encode order or flex-item viewport
  parent axes; the corpus has 5,256 generated outputs and six reports.
- Exit evidence: Helper, serializer, parser, stale-provenance, report, and
  inventory checks prove exact order handling, strict parent-axis attributes,
  three new order sources, 16 updated flex-item-root XML files, 1,406 HTML,
  5,268 XML, 356 unchanged unsupported tuples, ten reports, current hashes, and
  byte-idempotent derived state.
- Handoff: `C03` immediately makes flex-parent metadata mandatory at the
  consumer/API boundary; `C04` through `C07` make generated expectations pass.

## C03 - Complete Containing Context And Cache Identity

- Owner: `surgeist-layout`
- Outcome: `ContainingLayoutContext` carries parent `FlowAxes` and
  `ParentFormattingContext` through every compute path and cache comparison;
  flex-item roots require the explicit axes already present in fixtures.
- Specification: parent-context scope in `FRI-03.2`; `E-PARENT-CONTEXT`;
  `D-03`; `FRI-03.5`; parent-context matrix in `FRI-03.6`; relevant
  `FRI-03.7`, `FRI-03.8`, `FRI-03.9`, and `FRI-03.11`; acceptance item 4.
- Prerequisites: `C01` and `C02`.
- Entry state: Recursive inputs and cache identity carry only flow axes;
  flex-item roots derive containing flow from the item.
- Exit evidence: One context value reaches direct leaf, viewport root,
  flex-item root, block, flex, ordinary-grid, grid-lanes, hidden, intrinsic,
  sizing, layout, and absolute paths; role-only cache differences miss;
  cached/uncached outputs agree in both scalar lanes; no old constructor or
  fallback remains.
- Handoff: `C04` consumes the role for collapse barriers; later algorithm cycles
  construct no independent context flag or flow mapping.

## C04 - Block And Root Participation

- Owner: `surgeist-layout`
- Outcome: Parent context gates only the current block box's boundary collapse,
  replaced block/root boxes no longer receive ordinary auto-inline fill, and
  block layout explicitly ignores item order.
- Specification: block/root and collapse scope in `FRI-03.2`;
  `E-BLOCK-REPLACED`; `E-COLLAPSE`; `D-04`; block/root rows of `D-05`;
  block/context matrices in `FRI-03.6`; relevant `FRI-03.8` and `FRI-03.9`;
  acceptance items 2, 5, and 6.
- Prerequisite: `C03`.
- Entry state: Collapse predicates cannot distinguish parent participation, and
  all non-table auto-inline block/root boxes receive ordinary fill even when
  replaced.
- Exit evidence: `BlockFlow` preserves valid collapse; `Flex`, `Grid`, and
  `NoParent` block only boundary collapse; block constants and measured-leaf
  output agree; replaced block, viewport-root, and flex-item-root controls pass;
  block order is ignored.
- Handoff: `BLOCK-007` and the block/root portion of `CORE-005` are closed.

## C05 - Flex Participation

- Owner: `surgeist-layout`
- Outcome: Flex consumes the canonical order-modified sequence before line
  construction and selects the specified replaced/non-replaced automatic main
  minimum without changing cross-axis stretch.
- Specification: flex order/replaced scope in `FRI-03.2`; `E-FLEX-ORDER`;
  `E-FLEX-REPLACED`; flex portions of `D-02` and `D-05`; flex rows/cases in
  `FRI-03.6`; relevant `FRI-03.8`, `FRI-03.9`, and `FRI-03.11`; acceptance
  items 2 and 6.
- Prerequisites: `C01`, `C02`, `C03`, and the item boundary behavior from `C04`.
- Entry state: Flex lines are built in source order, and automatic minimum
  always takes the larger content/transferred suggestion.
- Exit evidence: Negative/equal/positive order, source ties, wrapping, reverse
  directions, output identity, and both scalar lanes prove one sort before line
  construction; replaced auto-min takes the smaller suggestion; existing caps,
  clamps, floors, overflow, cross-axis stretch, and exact flex order outputs
  pass.
- Handoff: Flex's FRI-03-owned portions of `MODEL-001` and `CORE-005` are closed
  without claiming `FRI-07` completeness.

## C06 - Ordinary-Grid Participation

- Owner: `surgeist-layout`
- Outcome: Ordinary-grid order-sensitive placement traverses one canonical
  permutation while storage remains source-indexed; one replaced-aware normal
  alignment resolver preserves explicit stretch.
- Specification: ordinary-grid order/replaced scope in `FRI-03.2`;
  `E-GRID-ORDER`; `E-GRID-REPLACED`; ordinary-grid portions of `D-02` and
  `D-05`; grid rows/cases in `FRI-03.6`; relevant `FRI-03.8` and `FRI-03.9`;
  acceptance items 2, 3, and 6.
- Prerequisites: `C01` through `C04`.
- Entry state: Definite-major and remaining auto-placement traverse source
  order, and default auto-sized replaced items follow ordinary stretch.
- Exit evidence: Fully definite occupancy is order-independent; row/column,
  sparse/dense, and mixed placement phases traverse order-modified indexes and
  write source-indexed storage; replaced default/normal resolves to start;
  non-replaced normal, explicit replaced stretch, and exact grid order outputs
  pass.
- Handoff: `C07` receives the single permutation and normal-alignment helper.

## C07 - Grid-Lanes Participation

- Owner: `surgeist-layout`
- Outcome: Grid-lanes production placement and sequential intrinsic
  contributions consume ordinary grid's exact permutation, and pre-placement
  measurement uses its replaced-aware normal alignment.
- Specification: grid-lanes scope in `FRI-03.2`; `E-LANES-ORDER`;
  `E-GRID-REPLACED`; grid-lanes portions of `D-02` and `D-05`; grid-lanes rows
  and cases in `FRI-03.6`; relevant `FRI-03.8` and `FRI-03.9`; acceptance
  items 2, 3, and 6.
- Prerequisite: `C06`.
- Entry state: Production lanes placement and intrinsic contributions enumerate
  source order, and pre-placement span measurement defaults replaced items to
  stretch independently of ordinary grid.
- Exit evidence: Running offsets and overlapping-span intrinsic evidence prove
  placement and contribution traversal share the same order; source/subgrid
  reporting remains source-indexed; replaced default does not inject span known
  size; non-replaced default, explicit replaced stretch, both scalar lanes, and
  exact grid-lanes order outputs pass.
- Handoff: All FRI-03 algorithm consumers are complete; broader `FRI-08`
  placement, sizing, and subgrid findings remain untouched.

## C08 - Public Surface, Evidence, And Initiative Closure

- Owner: `surgeist-layout`
- Outcome: Public docs, exact browser topology, corpus metadata, source absence
  gates, finding evidence, and root integration requirements describe only the
  completed participation model.
- Specification: `FRI-03.5`; `FRI-03.7`; `FRI-03.8`; `FRI-03.9` through
  `FRI-03.12`; closure matrix `FRI-03.13`; and all acceptance items in
  `FRI-03.14`.
- Prerequisites: `C01` through `C07`; `C05` and `C06` may complete in either order after `C04`.
- Entry state: Every owned production branch is implemented, but
  initiative-wide topology, absence, documentation, report, and handoff evidence
  is not yet sealed as one candidate.
- Exit evidence: Exact 32-output inventory, four scopes, full 5,268/356 report,
  ten reports, provenance, generated-tree cleanliness, public API/docs, source
  absence gates, focused/full/generator/corpus/doc/rustdoc/format/diff checks,
  and unsafe scan are green while unrelated ignored aggregate parity remains
  visible and unclaimed.
- Handoff: Publish the independently reviewed leaf candidate, read it back from
  remote `main`, and record root-owned CSS order, replacedness, flex-parent
  axes, invalidation, consumer rename, facade, gitlink, integration-test, and
  API-artifact obligations without editing root.
