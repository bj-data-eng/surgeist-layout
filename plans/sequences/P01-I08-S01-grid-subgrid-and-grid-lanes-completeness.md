# P01-I08-S01 Grid, Subgrid, And Grid-Lanes Completeness Implementation Sequence

Sequence ID: `P01/I08/S01`

Owning repository: `surgeist-layout`

## 1 Authority

This sequence implements the independently reviewed specification at
`plans/specs/P01-I08-grid-subgrid-and-grid-lanes-completeness.md`, SHA-256
`150c26e6c5b5fa703f090e861261ea2f03a7662caf4f83dfa52f49e40accb0ba`,
committed as `c7d10c23c0cdfebfba6a6606d9ea5b89352572f5`.

The specification owns behavior, public API, compatibility, ownership,
artifacts, errors, and acceptance. This sequence owns only durable dependency
order. Only the next ready cycle receives a detailed just-in-time plan.

## 2 Sequence Boundary

Every cycle mutates only `surgeist-layout`. Root computed-style lowering,
facade wiring, generated API artifacts, and gitlink promotion remain the
separate handoff in `FRI-08.18`.

No cycle adds a dependency, feature, MSRV change, unsafe code, authored CSS or
HTML parser, retained identity, rendering state, parallel flow/order/overflow
owner, reusable generator layer, second generator path, or behavior owned by
FRI-09 through FRI-13. `GRID-004`, `GRID-009`, and `GRID-011` remain closed by
their published owners.

The exact eighteen-source, 72-row browser surface and the single unfiltered
existing-pinned generation are owned by `FRI-08.13`. Scoped generation may
diagnose settled input while an owning cycle is in progress, but it is not
acceptance evidence and writes no report. All behavior, adapter, helper, HTML,
manifest, and provenance inputs settle before the one full run in C06.

Repository-wide `just parity-all` remains the FRI-13 aggregate release gate.
FRI-08 runs its exact owned rows and keeps FRI-09 baseline and FRI-10 positioned
negative controls visible without claiming or suppressing them.

## 3 Ordered Cycles

### 3.1 `P01/I08/S01/C01` Canonical Grid Topology And Placement

**Specification sources:** `FRI-08.5 D-01` through `D-06`, `D-12`, and `D-13`;
`FRI-08.7`; `FRI-08.9`; topology, placement, names, error, architecture, and
finding-closure portions of `FRI-08.12`, `FRI-08.14` through `FRI-08.17`, and
`GRID-001`, `GRID-005`, `GRID-008`.

**Prerequisites:** Published FRI-07 candidate and the clean reviewed FRI-08
specification revision recorded above.

**Entry state:** Expanded sized tracks, area-derived named-line counts, guessed
implicit demand, and final occupancy are separate facts. Placement receives a
fixed scalar-sized matrix, valid overflow returns a zero area, area-only
topology exists only incidentally when an item grows it, and duplicate tokens
multiply named occurrences.

**Bounded outcome:** Introduce one private axis topology that unifies explicit
track identity, template-area dimensions, named lines, auto-track pattern
phase, and integer placed areas. Move ordinary placement before scalar sizing,
use growable occupancy and exact cursor-driven implicit creation, preserve
leading/trailing auto-pattern phase, and materialize geometry only after tracks
settle. Deduplicate named lookup per line while retaining origin evidence.

**Observable exit evidence:** The span-after-occupied case grows one exact row;
definite overlap adds no row; dense/sparse, row/column flow, leading/trailing
implicit, automatic spans, order, absolute/display-none controls, area-only
empty grids, mixed area/list dimensions, negative lines, named spans, duplicate
origin collisions, f32/f64, invalid input, rollback, and cache cases pass. No
child-count/`div_ceil` demand or valid zero-area sentinel remains.

**Handoff:** C02 receives stable integer placements and track origin metadata;
it does not redesign names, areas, cursor semantics, or source association.

### 3.2 `P01/I08/S01/C02` Ordinary Track Sizing And Auto-Fit

**Specification sources:** `FRI-08.5 D-07`, `D-09` through `D-11`, and `D-21`;
`FRI-08.8`; ordinary-grid sizing and auto-fit portions of `FRI-08.12`,
`FRI-08.14` through `FRI-08.17`; `GRID-003`, `GRID-006`, and `GRID-007`.

**Prerequisites:** C01 complete and remotely verified.

**Entry state:** Track origins now survive placement, but auto-fit still needs
post-placement collapse. Any fit-content column still selects an alternate
collection-wide result, and stretch excludes auto-max tracks with non-auto
minimums.

**Bounded outcome:** Carry base size, growth limit, fit-content limit, flex
factor, auto-max eligibility, and auto-fit origin through one row/column solver.
Collapse ordinary auto-fit from placed occupancy with zero sizes and collapsed
adjacent gutters. Integrate fit-content into ordinary intrinsic/spanning/flex
phases and stretch every non-collapsed auto maximum from positive definite free
space.

**Observable exit evidence:** `[fit-content(50px),1fr]` resolves to `[20,180]`;
the existing fit-content/span families pass; overlapping children leave one
centered `40px` auto-fit track; all-empty, spanning, named-line, gap, percentage,
vertical/sideways, row/column, minmax-auto, min/max-content, scalar, rounding,
cache, error, and rollback controls match the oracle. No fit-content early
return or exact `auto/auto` stretch predicate remains.

**Handoff:** C03 consumes stable grid-axis tracks and shared track facts; it
adds the distinct Level 3 lanes policies without weakening ordinary behavior.

### 3.3 `P01/I08/S01/C03` Grid-Lanes Containing Blocks And Intrinsic Projection

**Specification sources:** `FRI-08.5 D-08`, `D-14` through `D-17`, and `D-21`;
`FRI-08.10`; lanes portions of `FRI-08.12`, `FRI-08.14` through `FRI-08.17`;
`GRID-002` and nested-lanes portions of `GRID-010`.

**Prerequisites:** C02 complete and remotely verified.

**Entry state:** Rows-only lanes measures percentage inline size from its own
tentative margin box; min/max-content container cases place against incomplete
intrinsic facts; lanes auto-fit inherits a child-count cap; an automatically
placed nested lanes subgrid returns a public unsupported state and its lower
bound is silently dropped.

**Bounded outcome:** Use the hybrid grid-area/container-content-box containing
block for all lanes measurement and final layout. Implement the separate Level
3 auto-fit heuristic, candidate-start intrinsic projection, equivalence-safe
virtual grouping, and descendant/edge flattening for nested indefinite lanes
subgrids. Remove the public nested-indefinite kind, constructor, and error
variant without a compatibility state.

**Observable exit evidence:** All four containing-block variants use width 100
with correct RTL placement; the eight min/max-content container variants pass;
lanes auto-fit explicit/automatic/span cases follow its heuristic; nested
subgrid descendants contribute at every allowed candidate with exact MBP and
half-gap maxima. Order, tolerance, both axes, all flow mappings, scalar, error,
provider, cache, and transaction controls pass, and removed-symbol API evidence
is exact.

**Handoff:** C04 receives complete Level 3 grid-axis sizing and placement with
no remaining nested-indefinite unsupported branch.

### 3.4 `P01/I08/S01/C04` Level 2 Subgrid And Overflow Composition

**Specification sources:** `FRI-08.5 D-18` through `D-21`; `FRI-08.11`;
subgrid, baseline-control, overflow, composition, architecture, and error
portions of `FRI-08.12`, `FRI-08.14` through `FRI-08.17`; remaining `GRID-010`.

**Prerequisites:** C03 complete and remotely verified.

**Entry state:** Inherited-axis traversal rejects a standalone-axis boundary.
New topology and sizing behavior has not yet been composed with the published
FRI-06 baseline carrier or the failing grid/subgrid FRI-05 overflow controls.

**Bounded outcome:** Make a standalone queried axis terminate ancestor
flattening and contribute one ordinarily measured grid-container leaf across
its translated span. Keep its local descendants inside ordinary local layout.
Compose completed topology, lanes, and subgrid sizing with immutable inherited
baseline views and canonical container-relative scroll contributions; correct
only defects exposed in these owned compositions.

**Observable exit evidence:** Standalone/inherited axes, nesting, reversal,
unequal gaps, MBP, area names, auto-flow, min-content, percentage, both scalar
lanes, and all flow mappings pass without the private unsupported error. The 16
named grid/subgrid overflow variants pass unchanged. FRI-06 first/last baseline
groups and no-fixed-point invariants remain green, while FRI-09/F10 controls
stay visible and unclaimed.

**Handoff:** C05 receives complete individual finding behavior through the
public front door with all residual GRID-010 capability branches closed.

### 3.5 `P01/I08/S01/C05` Grid Composition Closure And Settled Browser Inputs

**Specification sources:** `FRI-08.5 D-19` through `D-21`; complete
`FRI-08.12`; input, adapter, fixture, documentation, architecture, finding, and
pre-generation portions of `FRI-08.13` through `FRI-08.19`.

**Prerequisites:** C04 complete and remotely verified.

**Entry state:** The eight findings work in focused owners, but their combined
order, flow, replaced, percentage, overflow, scrollbar, baseline-control,
cache, transaction, and rounding interactions are not one accepted candidate.
The ten new fixture sources and exact 72-row manifest ownership are absent.

**Bounded outcome:** Exercise the complete composition matrix through public
layout and correct only in-scope interaction defects without adding duplicate
owners. Complete grid-boundary documentation. Add the exact ten new HTML
sources, adopt the exact eight existing controls, and settle every finite
helper/parser/manifest input required for honest input-derived lowering. Do not
run the full generator or modify generated XML/report artifacts.

**Observable exit evidence:** Every finding has a minimal and composed public
oracle in f32/f64; negative controls retain FRI-09/F10 ownership; adapter
rejection and fixture-independence tests pass; the exact eighteen sources and
four variants derive 72 unique rows; no fixture-name or expected-geometry
dispatch, new parser layer, dependency, feature, suppression, or generated
artifact delta exists.

**Handoff:** C06 receives behavior-complete, documented, immutable browser
inputs whose only remaining operation is authoritative full generation and
read-only validation.

### 3.6 `P01/I08/S01/C06` Browser Artifact Candidate

**Specification sources:** complete `FRI-08.13`; artifact, verification,
architecture, finding, handoff, and acceptance portions of `FRI-08.14` through
`FRI-08.19`.

**Prerequisites:** C05 complete and remotely verified; production, helper,
adapter, HTML, manifest, provenance schema, and expected geometry inputs are
settled and frozen.

**Entry state:** Ten new sources have no checked-in XML or report lineage; the
base report still contains 5,736 generated rows and its old manifest/helper
hashes.

**Bounded outcome:** Run exactly one unfiltered full existing-pinned generation
through the authoritative generator, producing the 40 new outputs and the sole
schema-3 report. Preserve comment-free XML and centralized provenance, the 16
unrelated unsupported rows, and the three FRI-07 expected-fail records. Add no
FRI-08 expected fail, quarantine, or failed-to-generate row. Validate all
artifacts and the exact owned parity set read-only after generation.

**Observable exit evidence:** Subject to the base-drift rule, the report has
5,776 generated XML, 16 unsupported variants, three unchanged expected-fail
source records, zero quarantined, and zero failed-to-generate. Every global,
source, linked-resource, and XML hash validates; XML is comment-free;
corpus/Taffy checks pass; all 72 owned rows pass; unrelated XML bodies change
only if the settled authoritative inputs require it. No second full run,
manual XML edit, second provenance authority, or acquisition occurs.

**Handoff:** Publish and remotely verify the behavior/artifact candidate with
exact source, report, helper, manifest, XML, browser, and negative-control
evidence before the final sprawl assessment begins.

### 3.7 `P01/I08/S01/C07` Validated Sprawl Containment And Final Candidate

**Specification sources:** the finite structural invariants in `FRI-08.14`;
final verification, responsibility, finding-closure, handoff, and acceptance
portions of `FRI-08.15` through `FRI-08.19`.

**Prerequisites:** C06 complete, published, and remotely verified; a fresh
holistic sprawl assessment has reviewed the exact FRI-08 implementation range
from the immutable initiative base through C06 and returned one finite,
identified finding set.

**Entry state:** All eight findings and their bounded artifact lineage are
closed in the immutable C06 candidate. Only the exact reviewed sprawl findings
remain for source validation and disposition.

**Bounded outcome:** Validate each assessed finding against current source.
Implement every confirmed in-initiative mechanical consolidation with
characterization evidence, or disprove it with an exact counterexample. Work is
limited to the finite assessment set and `FRI-08.14`; no new architecture
search, behavior, API, fixture, artifact, dependency, feature, or later-owned
scope enters C07. A genuine behavior/input/artifact defect reopens its exact
owning cycle rather than being hidden as refactoring.

**Observable exit evidence:** Every assessed finding has one recorded
disposition; all structural invariants hold; all eight finding closures, 72
owned rows, centralized provenance, public API removal, FRI-09/F10 controls,
dependencies, features, and artifact hashes remain unchanged unless an owning
contract was explicitly reopened. Full final verification and fresh holistic
review are clean without an unneeded generation run.

**Handoff:** Return the final FRI-08 leaf candidate, eight-finding closure,
finite sprawl dispositions, exact artifact lineage, public/root boundary, and
later-P01 continuation state.

## 4 Sequence Completion

This sequence ends at `P01/I08/S01/C07`; no later cycle is represented.
