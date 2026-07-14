# FRI-02 Logical Geometry And Writing Modes Implementation Sequence

## Authority

- Owning repository: `surgeist-layout`
- Reviewed specification:
  `plans/specs/2026-07-12-surgeist-layout-fri-02-logical-geometry-writing-modes.md`
- Specification normalized SHA-256:
  `ed20972484206e22c3b28ab27671390a218d3083adf4b2480c8a4f78a702a177`
- Specification commit:
  `0fd7f3f67a825a2176e76c83ead44f76039498e7`
- Initiative index:
  `plans/specs/2026-07-11-surgeist-layout-findings-resolution-index.md`,
  `FRI-02`

This sequence orders the complete implementation of `BLOCK-003`, `FLEX-001`,
`GRID-004`, `OVERFLOW-004`, and `TEST-005`. Detailed design remains owned by
the reviewed specification.

## Sequence Boundary

All cycles are owned by `surgeist-layout`. Root and sibling repositories remain
outside the mutation boundary; only the final archival handoff describes their
later integration work. No cycle adds a compatibility alias or duplicates
writing-mode interpretation. An existing legacy surface may remain only while a
later named cycle still has a live consumer, and `C08` removes every such
temporary bridge before initiative closure.

Browser-backed evidence uses the already-present pinned executable and the
specification's `ExistingPinned` path unless the user separately authorizes an
acquisition. No cycle retunes the experimentally established fixture-generation
profile. C07 closes the observed whole-job, owned-process, cleanup, and sealed-
snapshot gaps without changing those settings.

## C01 - Shared Flow And Compute Context

- Owner: `surgeist-layout`
- Outcome: One canonical `FlowAxes` model represents all five writing modes and
  both used directions for physical/logical geometry other than scroll;
  physical public geometry and crate-private logical geometry are distinct; every
  compute entry path carries containing flow.
- Specification: `FRI-02.1` through `FRI-02.5`; the public geometry and
  containing-flow portions of `FRI-02.6` excluding its four scroll-conversion
  methods and complete public scroll surface; `FRI-02.7`; non-scroll errors in
  `FRI-02.12`; the shared/core rows of `FRI-02.14`; and construction and mapping
  evidence in `FRI-02.17`.
- Prerequisite: Published `FRI-01` candidate at the sequence base.
- Entry state: Writing-mode knowledge is split across physical helpers and
  algorithm-local mappings, and compute construction lacks containing flow.
- Exit evidence: All ten mapping rows and scalar lanes are covered; direct,
  root, flex-root, child, and hidden construction preserve flow and cache
  identity; production mapping delegates to the canonical owner.
- Handoff: `C02`, `C04`, `C05`, `C06`, and `C07` may consume the shared model
  without defining another mapping table.

## C02 - Signed Scroll Coordinate Contract

- Owner: `surgeist-layout`
- Outcome: Physical and flow-relative offsets and ranges have unambiguous signed,
  scalar-generic types, typed construction failures, and one projection path
  through the four scroll-conversion methods added to `FlowAxes` in this cycle.
- Specification: `FRI-02.4` decision `D-17`; the four `FlowAxes` scroll
  conversion methods and complete public scroll surface in `FRI-02.6`;
  `FRI-02.11`; applicable errors in `FRI-02.12`; scroll rows of `FRI-02.14`; and
  scroll evidence in `FRI-02.17`.
- Prerequisite: `C01`.
- Entry state: Scroll offsets and ranges use unresolved coordinate conventions
  and unsigned-origin assumptions.
- Exit evidence: Invalid/non-finite intervals fail semantically; conversion,
  clamp, round-trip, normal, and rounded projection evidence passes for all ten
  mappings and both scalar lanes; `OVERFLOW-004` is closed without claiming
  `FRI-05` geometry.
- Handoff: Later algorithm cycles use only the typed projection contract.

## C03 - Reproducible Browser Runtime

- Owner: `surgeist-layout`
- Outcome: Managed-pinned and existing-pinned resolution validate the actual
  executable version and share one manifest-owned launch profile, while
  non-browser corpus commands remain independent of browser state.
- Specification: parser/generator requirements in `FRI-02.13`, generator rows of
  `FRI-02.14`, and browser, feature, stability, and test contracts in
  `FRI-02.16` and `FRI-02.17`.
- Prerequisite: `C01`; no algorithm migration depends on this cycle.
- Entry state: Browser overrides are environment-driven, explicit paths are not
  version-validated, and primary/retry launch construction is duplicated.
- Exit evidence: Both resolution modes fail closed on pin/path/provenance errors;
  every launch site uses the exact shared profile; configured batch, ordering,
  DOM wait, retry, profile, polling, keychain-bypass, and version regressions pass;
  Taffy maintenance commands remain browser-free. The schema-v2 manifest and
  regenerated metadata temporarily inventory exactly the current `all.json` and
  nine committed pre-FRI-02 scoped reports, and `check-corpus` is green against
  that state.
- Handoff: The current nine scoped report records are the only temporary live
  inventory consumer. `C04` through `C07` add their five manifest entries and
  refresh the full report cumulatively; C07 also closes the later-observed total
  job/process/publication defect without retuning the profile. `C08` removes the
  nine temporary entries and prunes their files. No compatibility manifest reader
  survives `C03`.

## C04 - Logical Block Flow

- Owner: `surgeist-layout`
- Outcome: Ordinary block sizing, placement, edges, collapse, baseline, root,
  hidden, parallel, and orthogonal behavior follows containing logical axes;
  compute output carries typed physical block-margin collapse state.
- Specification: `FRI-02.4` decision `D-18`; public collapsible-margin output in
  `FRI-02.6`; `FRI-02.8`; block portions of `FRI-02.12` through `FRI-02.14`;
  block evidence in `FRI-02.17`; and acceptance items 4-5 in `FRI-02.20`.
- Prerequisites: `C01`, `C02`, and `C03`.
- Entry state: The signed scroll projection contract and reproducible browser
  runtime are complete, while ordinary block flow remains physically vertical
  outside limited inline-control paths and compute margin output exposes
  top/bottom fields plus an axis-free collapse-through boolean.
- Exit evidence: The typed carrier and block/measured-leaf matrices prove
  parallel/opposing collapse and orthogonal isolation in both scalar lanes;
  its containing-flow-aware query remains; all three loose fields are absent
  without aliases/conversions. Named algorithm evidence and the exact
  five-family, 20-output block browser matrix pass without absorbing `FRI-05`,
  `FRI-06`, or `FRI-10`; block/full reports validate and `BLOCK-003` is closed.
- Handoff: Later algorithms consume typed physical collapse state; no legacy
  vertical-only flow, loose margin field, or unqualified through query remains.

## C05 - Logical Flex Flow

- Owner: `surgeist-layout`
- Outcome: Current flex sizing, placement, wrapping, margins, alignment,
  baselines, absolute/static placement, and output projection use logical
  main/cross axes derived from `FlowAxes`.
- Specification: `FRI-02.9`, flex portions of `FRI-02.12` through `FRI-02.14`,
  flex evidence in `FRI-02.17`, and acceptance item 6 in `FRI-02.20`.
- Prerequisites: `C01`, `C02`, `C03`, and `C04`.
- Entry state: Flex direction helpers bind row and column to physical axes, and
  existing vertical fixtures do not reach non-leaf flex layout.
- Exit evidence: All five modes, both directions, all four flex directions, and
  wrap reversal pass focused evidence; the exact 20-family, 80-output non-leaf
  browser matrix reaches `compute_flex`; its manifest entry and refreshed full
  report validate; `FLEX-001` and `TEST-005` are closed.
- Handoff: Flex has no remaining consumer that justifies a legacy axis helper.

## C06 - Logical Ordinary Grid

- Owner: `surgeist-layout`
- Outcome: Ordinary grid keeps columns and rows logical through intrinsic sizing,
  track totals, areas, gaps, reruns, and physical projection.
- Specification: `FRI-02.10` excluding lanes/subgrid-specific behavior, ordinary
  grid portions of `FRI-02.13` and `FRI-02.14`, grid evidence in `FRI-02.17`,
  and the ordinary-grid part of acceptance item 7 in `FRI-02.20`.
- Prerequisites: `C01` through `C05`.
- Entry state: Grid axis roles partly map through physical width/height and leave
  vertical intrinsic dimensions unswapped.
- Exit evidence: Unequal intrinsic totals, areas, baselines, parallel, opposing,
  and orthogonal flows pass focused evidence and the exact nine-family,
  36-output ordinary-grid browser matrix; its manifest entry and refreshed full
  report validate.
- Handoff: `C07` receives logical ordinary-grid tracks and projection semantics.

## C07 - Logical Lanes And Subgrid

- Owner: `surgeist-layout`
- Outcome: Grid-lanes and subgrid inheritance, offsets, areas, and baseline
  projection preserve logical column/row identity across parent and child flows;
  their browser evidence runs through finite owned processes and one sealed,
  generator-bound artifact snapshot.
- Specification: lanes/subgrid behavior in `FRI-02.10`, their fixture matrices in
  `FRI-02.13`, its report/snapshot/oracle contract, corresponding generator rows
  of `FRI-02.14`, generator lifecycle and artifact impact in `FRI-02.16`, grid and
  generator evidence in `FRI-02.17`, and acceptance items 7 and 9 in `FRI-02.20`.
- Prerequisites: `C01` through `C06`.
- Entry state: Lanes and subgrid consumers still project inherited roles through
  physical assumptions. CDP work outside the DOM wait can hang indefinitely,
  browser launch can lose child ownership on timeout, cleanup can outlive its
  bound, and reports do not content-bind one admissible XML snapshot.
- Exit evidence: Parallel, opposing, orthogonal, inherited-track, area, and
  baseline evidence passes with exact 36-output grid-lanes and 36-output subgrid
  browser matrices; owned launch/job/teardown transitions are finite and tested;
  full and scoped publication share one generator-bound commitment and `all.json`
  seal; the transitional 15-report manifest and refreshed full run validate;
  `GRID-004` is closed without absorbing `FRI-08` defects.
- Handoff: All algorithm families are ready for initiative-wide surface and
  corpus closure.

## C08 - Corpus, Public Surface, And Initiative Closure

- Owner: `surgeist-layout`
- Outcome: The manifest, generated artifacts, report inventory, public API,
  documentation, and finding records describe only the completed FRI-02 model.
- Specification: report and oracle contracts in `FRI-02.13`, cleanup/documentation
  rows of `FRI-02.14`, `FRI-02.15` through `FRI-02.20`.
- Prerequisites: `C01` through `C07`.
- Entry state: Every algorithm and fixture family is implemented, but temporary
  legacy surfaces and initiative-wide corpus metadata may remain.
- Exit evidence: Old axis/helper surfaces and duplicate mappings are absent; the
  exact 208-output FRI-02 union and six-report manifest inventory validate; the
  full report has the specified counts with all 356 unrelated unsupported tuples
  unchanged; the nine temporary pre-FRI-02 manifest entries and files are gone;
  public docs, feature states, MSRV evidence, and all five finding closures agree.
- Handoff: Publish the reviewed layout candidate and provide root the archival
  adapter/facade/API-artifact obligations from `FRI-02.15`; do not edit root.
