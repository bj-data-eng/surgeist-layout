# P01 Surgeist Layout Findings Resolution Index


Design owner: `surgeist-layout`

## 1 Authority And Outcome

This index is the initiative-wide coverage contract for resolving the verified
findings in the current canonical artifact
`plans/P01-layout/P01-initial-review-findings.md`. That file normalizes the
historical committed artifact at Git object path
`d8819de2fceeb130726f1b87f1172b60750e4455`, repository directory `plans`,
basename `2026-07-10-surgeist-layout-full-code-review-findings.md`.
Both have SHA-256
`6b29ff91a886af83e5c47aa2f9441165350719a1f436c11f96e4c2c0033744fb`.

The outcome is a layout engine whose public layout-ready contracts, algorithms,
outputs, and conformance evidence resolve all 59 findings from first principles.
Every finding has one closure owner below. Contributing initiatives may supply a
prerequisite, but they do not duplicate closure ownership.

## 2 Program Boundary

This program owns changes to the `surgeist-layout` public front door, semantic
types, algorithms, caches, diagnostics, fixture schemas, browser-parity harness,
oracle tests, generated browser XML, and crate documentation.

It does not own authored CSS, cascade or style resolution, retained identity,
text shaping, rendering, root facade composition, root adapters, root API
artifacts, or sibling repositories. A layout initiative may define the complete
typed layout-ready contract those owners consume. Root integration is outside
this leaf repository and specification; required adapter and facade handoffs
remain root-owned.

Backward compatibility is not required before the first release. Specifications
must choose the correct typed model without compatibility aliases, duplicate
lowering paths, permissive legacy states, or panic-based unsupported behavior.
No initiative may add or retain Surgeist-owned `unsafe`.

The findings ledger remains immutable intake evidence. This index assigns
technical closure ownership; each direct specification defines its finite
product evidence.

## 3 Shared Resolution Invariants

1. Public inputs make invalid intrinsic states unconstructable and distinguish
   authored, layout-ready, algorithm, fixture, and output phases where their
   invariants differ.
2. Contextual resolution receives every influencing basis or context input
   explicitly. Missing context returns a semantic result; it never panics,
   guesses, or introduces an identity or revision where the normalized value has
   no identity-bearing semantics.
3. Cached and uncached computations are observably equivalent for every output
   field and failure state.
4. Shared logical-axis types own writing-mode and direction mapping. Formatting
   algorithms do not independently reinterpret physical axes.
5. Layout computes geometry only. Text shaping, style resolution, retained-tree
   identity, painting, and live scroll state stay outside the crate boundary.
6. Unsupported but well-formed layout requests are typed and testable until the
   owning initiative implements them; temporary diagnostics are not closure for a
   capability finding.
7. Every behavior finding closes with focused evidence through the real layout
   front door plus the applicable browser, oracle, property, and scalar coverage.
8. Assurance findings are implemented inside the initiative whose behavior they
   must detect. `FRI-13` owns only the final aggregate release gate.
9. Generated XML changes come only from the crate-owned generator and retain
   source-fixture provenance. An unsupported or quarantine bucket remains visible
   and cannot count as conformance.
10. Public contract changes identify their required root integration handoff;
    this leaf does not implement root adapters or facade composition.

## 4 Finding Ownership

| Initiative | Planning state | Bounded outcome | Solely owned findings | Required inputs |
| --- | --- | --- | --- | --- |
| `FRI-01` Compute, resolution, and diagnostic contracts | Planned as `P01/I01` | Fallible computation, normalized affine length-percentage resolution, semantically complete caching, and validated numeric measurement boundaries | `CORE-001`, `CORE-002`, `CORE-003`, `CORE-004`, `CORE-007`, `DIAG-001` | Current public compute, cache, value, and traversal contracts |
| `FRI-02` Logical geometry and shared writing-mode substrate | Planned as `P01/I02` | One typed logical-axis model drives block, flex, grid, and scroll-direction behavior, including vertical and sideways modes, and supplies the substrate consumed by inline layout | `BLOCK-003`, `FLEX-001`, `GRID-004`, `OVERFLOW-004`, `TEST-005` | `FRI-01` result and error contract |
| `FRI-03` Box participation contracts | Planned as `P01/I03` | Layout-ready inputs encode order, replaced behavior, and parent formatting participation without contradictory or dead fields; all affected algorithms consume those facts | `MODEL-001`, `CORE-005`, `BLOCK-007` | `FRI-01`; logical roles from `FRI-02` where axis-sensitive |
| `FRI-04` Property-specific sizing values | Planned as `P01/I04` | Box, track, flex-basis, intrinsic sizing, and calc functions use property-appropriate typed values; invalid cross-property states such as box-size `fr` are unconstructable | `MODEL-005`, `MODEL-007` | `FRI-01`; logical axes from `FRI-02` |
| `FRI-05` Overflow and scroll geometry | Planned as `P01/I05` | Overflow values, clipping, gutters, nested contribution, directional ranges, output helpers, and constructors form one coherent geometry contract across block, flex, and grid | `BLOCK-001`, `BLOCK-002`, `GRID-011`, `OVERFLOW-001`, `OVERFLOW-002`, `OVERFLOW-003`, `OVERFLOW-005`, `CORE-006`, `GRID-009`, `TEST-002` | `FRI-01`, `FRI-02`, and property types from `FRI-04` |
| `FRI-06` Inline formatting, line boxes, floats, and BFCs | Planned as `P01/I06` | Layout consumes typed shaped-text participants and computes horizontal and vertical wrapping, struts, baselines, line alignment, atomic boxes, line controls and clear, float exclusion, and BFC geometry | `BLOCK-014`, `FLOW-002`, `BLOCK-004`, `FLOW-001`, `FLOW-003`, `BLOCK-005`, `BLOCK-006`, `BLOCK-008`, `BLOCK-009`, `BLOCK-011`, `BLOCK-012`, `BLOCK-013`, `TEST-003`, `TEST-004` | `FRI-01` through `FRI-05`; root integration owns style/text lowering into the layout contract |
| `FRI-07` Flex algorithm completeness | Index coverage only | Flex auto margins, absolute-position interactions, intrinsic bases, collapsed-item struts, order, writing modes, replaced sizing, and overflow compose correctly | `FLEX-002`, `FLEX-003`, `FLEX-004`, `FLEX-005` | `FRI-01` through `FRI-05`; order/replaced roles from `FRI-03` |
| `FRI-08` Grid, subgrid, and grid-lanes completeness | Index coverage only | Placement demand, containing blocks, track sizing, template areas, auto-fit, named lines, Level 2 subgrid, lanes, writing modes, order, and overflow compose correctly | `GRID-001`, `GRID-002`, `GRID-003`, `GRID-005`, `GRID-006`, `GRID-007`, `GRID-008`, `GRID-010` | `FRI-01` through `FRI-05`; order roles from `FRI-03` |
| `FRI-09` Cross-format alignment semantics | Index coverage only | Baseline content distribution, block content alignment, text justification, and the full inline vertical-alignment set share typed logical semantics and are consumed correctly by their formatting contexts | `MODEL-006` | `FRI-02` through `FRI-04` plus stable inline, flex, and grid behavior from `FRI-06` through `FRI-08` |
| `FRI-10` Positioned layout | Index coverage only | Static, relative, absolute, fixed, sticky, and anchor-derived geometry use explicit containing-block and scroll context, including inline hypothetical positions | `MODEL-003`, `BLOCK-010` | `FRI-01` through `FRI-06` and alignment semantics from `FRI-09` where applicable |
| `FRI-11` Fragmentation and line limiting | Index coverage only | Fragmentainer, multi-column, multi-fragment output, break controls, line clamping, and block ellipsis form a complete layout-owned contract across every supported formatting context | `MODEL-004`, `FLOW-004` | Stable domain and display-system contracts from `FRI-01` through `FRI-10` and `FRI-12A` through `FRI-12E` |
| `FRI-12A` Normalized outer/inner display roles | Index coverage only | Layout-ready types represent valid outer participation, inner formatting context, list-item role, and table/ruby internal roles without owning DOM box construction or `display: contents` normalization | None; prerequisite to `FRI-12F` closure | Participation contracts from `FRI-03` |
| `FRI-12B` Inline-flex and flow-root composition | Index coverage only | Inline-flex participates atomically while using flex internally, and flow-root establishes an independent BFC, with browser parity for both compositions | None; prerequisite to `FRI-12F` closure | `FRI-06`, `FRI-07`, and `FRI-12A` |
| `FRI-12C` List-item and marker formatting | Index coverage only | Layout-ready list-item and marker roles produce intrinsic contributions, marker placement, baselines, and fragmentation inputs without counter or generated-content ownership | None; prerequisite to `FRI-12F` closure | `FRI-06`, `FRI-09`, and `FRI-12A` |
| `FRI-12D` Table formatting | Index coverage only | Table wrapper, grid, row group, row, column, cell, caption, intrinsic sizing, border-spacing, baseline, and positioned geometry have a complete layout-owned algorithm and parity surface | None; prerequisite to `FRI-12F` closure | `FRI-01` through `FRI-05`, `FRI-09`, `FRI-10`, and `FRI-12A` |
| `FRI-12E` Ruby formatting | Index coverage only | Ruby base, annotation, container, line contribution, alignment, and writing-mode geometry have a typed layout-ready contract and parity surface | None; prerequisite to `FRI-12F` closure | `FRI-02`, `FRI-06`, `FRI-09`, and `FRI-12A` |
| `FRI-12F` Layout containment and integrated display closure | Index coverage only | Layout and size containment affect formatting-context establishment and intrinsic sizing, and the complete normalized display-role surface from `FRI-12A` through `FRI-12E` is verified without half-modeled variants | `MODEL-002` | `FRI-04`, `FRI-11`, and completed `FRI-12A` through `FRI-12E` |
| `FRI-13` Release conformance gate | Index coverage only | Normal verification executes the complete supported browser corpus, restores current WPT provenance and classified coverage, and cannot report green while required parity is ignored, skipped, or unobserved | `TEST-001` | Closure evidence from `FRI-01` through `FRI-12F` |

The ownership table contains 59 distinct finding IDs. Each row is a direct,
one-hop specification boundary. `FRI-12F` is the sole closure owner for the broad
display finding; `FRI-12A` through `FRI-12E` are finite prerequisite initiatives,
not a nested planning index.

## 5 Dependency Facts

- `FRI-01` changes the computation substrate and therefore precedes algorithmic
  initiatives.
- `FRI-02`, `FRI-03`, and `FRI-04` are public model foundations. They close only
  behavior independent of later inline, alignment, and display-system work.
- `FRI-05` owns shared overflow output and contribution semantics; domain
  algorithms consume that contract instead of synthesizing local conventions.
- `FRI-06`, `FRI-07`, and `FRI-08` are distinct domain initiatives that consume
  the same completed shared contracts.
- Before `FRI-07` begins flex-completeness behavior, validate `MR-001` through
  `MR-006` from `plans/P01-layout/P01-I06-mechanical-refactoring-review-findings.md`
  against the published `FRI-06` candidate. Implement every still-applicable
  opportunity as the first `FRI-07` cycle, with behavior-preserving
  characterization evidence; record any disproven opportunity with exact source
  evidence. This ordering leaves all 59 existing closure-owner assignments
  unchanged and unduplicated; it does not close an `FRI-07` finding or authorize
  a future just-in-time cycle plan before `FRI-06` publication and the reviewed
  `FRI-07` specification and sequence.
- `FRI-09` follows the inline, flex, and grid initiatives because its alignment
  behavior consumes their completed line, baseline, and distribution mechanisms.
- `FRI-10` depends on inline static-position facts and shared scroll context.
- `FRI-12A` establishes normalized display roles. `FRI-12B` through `FRI-12E`
  each consume the named domain prerequisites and close one formatting system.
- `FRI-11` follows every formatting system it must fragment; it cannot be closed
  by adding an output container around unfragmented algorithms.
- `FRI-12F` adds containment and verifies the integrated display surface after
  the direct display-system initiatives and fragmentation are complete.
- `FRI-13` is the final aggregate gate. Comparator and fixture gaps named by
  other findings close earlier inside their behavior owners.

## 6 Planning Handoff

Execution follows the canonical `$surgeist-agent` planning pipeline. This index
defines only program coverage, boundaries, dependency facts, and product
acceptance.

## 7 Program Acceptance

The findings-resolution program is complete only when:

1. all 59 IDs retain exactly one closure owner and no verified finding is
   narrowed, silently deferred, converted into compatibility behavior, or hidden
   by an unsupported bucket;
2. every direct initiative satisfies its finite product acceptance criteria;
3. every representable valid layout-ready request returns correct geometry or a
   specified semantic error, with no panic, silent zero geometry, or guessed
   contextual value;
4. block, inline, float, flex, grid, subgrid, grid-lanes, positioned, overflow,
   display-system, and fragmentation behavior passes its owned scalar, writing
   mode, direction, edge, invalid, browser, and oracle evidence;
5. the supported checked-in XML corpus and current required WPT/conformance gate
   run in normal verification and are green without ignored aggregate tests or
   unobserved expectations;
6. the public front door, docs, feature matrix, generated fixtures, and root
   handoffs agree with the implemented contracts.
