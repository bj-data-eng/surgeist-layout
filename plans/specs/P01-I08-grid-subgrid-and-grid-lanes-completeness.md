# P01-I08 Grid, Subgrid, And Grid-Lanes Completeness

Design owner: `surgeist-layout`

## 1 FRI-08.1 Authority And Outcome

This specification is the authoritative desired-state contract for `FRI-08` in
`plans/P01-layout/P01-index.md`. It closes exactly these findings from
`plans/P01-layout/P01-initial-review-findings.md`:

- `GRID-001`, placement-derived implicit-track demand;
- `GRID-002`, grid-lanes containing-block definiteness;
- `GRID-003`, fit-content and flexible-track composition;
- `GRID-005`, template-area-created explicit topology;
- `GRID-006`, occupancy-derived auto-fit collapse;
- `GRID-007`, stretch of every auto-max track;
- `GRID-008`, one named-line occurrence per matching line; and
- `GRID-010`, the remaining representable Level 2 subgrid and Level 3
  grid-lanes capability gaps.

The outcome is one scalar-generic grid pipeline in which explicit topology,
placement, implicit growth, auto-fit occupancy, track sizing, inherited
subgrid contribution, grid-lanes intrinsic projection, and physical output are
phase-ordered facts. Ordinary grid, subgrid, and grid-lanes consume the
published writing-mode, order, replaced-item, overflow, transaction, cache, and
diagnostic contracts instead of reconstructing them locally.

This is an intentional breaking pre-release correction at crate version
`0.1.0`. Compatibility aliases for an unsupported nested-lanes state are not
required. No authored CSS grammar, DOM behavior, retained identity, painting,
or root facade behavior enters this leaf.

This specification supersedes stale source locations, corpus counts, baseline
coverage claims, and one incorrect expected value in the 2026-06-17 findings
snapshot. It does not change the finding IDs or their sole closure owner.

## 2 FRI-08.2 Ownership And Non-Goals

`surgeist-layout` owns:

1. the private expanded-grid topology and the phase ordering from normalized
   track components through placed integer grid areas;
2. ordinary-grid automatic placement, occupancy, implicit track creation,
   auto-fit collapse, track sizing, alignment, and geometry materialization;
3. template-area dimensions, named-line membership, named placement, subgrid
   line inheritance, and the distinction between explicit identity and sizing
   source;
4. grid-lanes track sizing, containing blocks, order-modified placement,
   running positions, intrinsic aggregation, and supported subgrid projection;
5. Level 2 subgrid intrinsic traversal across inherited axes and ordinary
   measurement at standalone-axis boundaries;
6. consumption of canonical overflow and writing-mode facts in the grid
   algorithms; and
7. focused, oracle, property, scalar, browser-parity, fixture, artifact,
   documentation, and sprawl evidence for this initiative.

Root `surgeist` owns authored CSS, cascade and computed-style resolution, box
generation, anonymous item construction, normalized display lowering, facade
composition, generated API artifacts, and the gitlink. This leaf publishes a
source candidate and handoff; it does not edit root.

The following remain outside FRI-08:

- new alignment values, baseline content distribution, and cross-format
  fallback semantics owned by `FRI-09`;
- general static, relative, absolute, fixed, sticky, and anchor positioning
  owned by `FRI-10`; existing grid-aligned absolute behavior remains a control;
- fragmentation owned by `FRI-11`;
- normalized outer/inner display roles and later display-system work owned by
  `FRI-12A` through `FRI-12F`;
- the aggregate release gate and wholesale WPT import owned by `FRI-13`;
- implementing every provisional Level 3 value, item-flow proposal, dense
  backfill proposal, stacking-axis baseline alignment, or fragmentation rule;
- authored CSS parsing, an HTML parser, browser-geometry inference, a new
  generator, generator architecture expansion, dependencies, features, MSRV
  changes, or unsafe code; and
- reopening `GRID-004`, `GRID-009`, or `GRID-011`, which retain their published
  FRI-02 and FRI-05 ownership.

Generator changes are allowed only when narrowly required to serialize or parse
the exact FRI-08 fixture inputs, add the finite sources, or correct a confirmed
genuine generator defect. Fixture identity and expected geometry never select
or alter layout input.

## 3 FRI-08.3 Initiative Base And Current Evidence

The initiative base is the remotely verified FRI-07 candidate
`238df34a713db4f90d7f194f6fdf89a994d34fa2`. Local `main`, `origin/main`, and
the authority remote ref were equal and the worktree was clean when this
specification was authored.

At that revision:

- `initialize_grid_tracks` expands auto-fit with `visible_child_count`, resolves
  named placements, sums placement cell spans, and uses `div_ceil` to estimate
  the other-axis track count before occupancy placement;
- `resolve_grid_child_areas` allocates a fixed occupancy matrix from those
  estimates and returns a zero-size out-of-range area when placement needs a
  track that does not exist;
- template-area facts enlarge `NamedGridLines`, but the expanded track vectors,
  explicit counts in `GridLines`, and area dimensions do not share one topology;
- `resolve_inline_tracks` returns immediately when any column has a fit-content
  maximum, bypassing flexible expansion for every other column;
- both ordinary track resolvers count stretch eligibility only for exact
  `min:auto/max:auto`, excluding `minmax(0,auto)` and every other auto maximum;
- `NamedGridLines::explicit_matches` emits an occurrence for each matching
  entry, so duplicate tokens on one line consume multiple occurrence numbers;
- rows-only grid-lanes constructs an item area whose inline size is the
  already-measured lane-axis margin box, so a percentage inline size loses the
  definite container content-box basis;
- `traverse_subgrid_intrinsic` returns
  `StandaloneSubgridTraversalUnsupported` at a standalone-axis boundary;
- `lane_intrinsic_sizing_with` returns
  `NestedGridLanesSubgridIndefiniteUnsupported`, and the production caller
  silently declines to merge that intrinsic lower bound; and
- the FRI-06 candidate already supplies axis-parametric inherited-subgrid
  baseline groups, owner-to-current maps, vertical/sideways projection, and a
  large focused baseline corpus. Those completed contracts are controls, not a
  second FRI-08 baseline implementation.

The browser-free canonical report at the base has schema version 3, 5,736
generated variants, 16 unchanged missing-root unsupported variants, three
FRI-07 expected-fail source records, zero quarantined, and zero
failed-to-generate. Its SHA-256 is
`5c560f240d27ad28d00023156b0bf2744aa8392d34fe916d800e02894e10353f`.
The corpus manifest SHA-256 is
`4419c4aab9429d1f81ac46426095719e19cf92cfbf51caf66d4f737c07c452cc`,
and the helper SHA-256 is
`caafa5a48787c9b80a45d8b2c8ac6f91b8ad7ab14a85e5bcdf3a3e922ebce019`.
There are 1,438 HTML sources and 5,736 comment-free XML outputs.

A browser-free diagnostic with `SURGEIST_PARITY_FILTER=grid` reports 90 current
failures: 62 ordinary-grid, 12 grid-lanes, and 16 subgrid variants. The FRI-08
owned rows include fit-content/span sizing, grid overflow consumption, all 12
grid-lanes rows, standalone subgrid sizing, and subgrid overflow composition.
The absolute percentage-containing-block rows remain FRI-10 work, and the
ordinary grid baseline rows remain FRI-09 work. FRI-08 does not claim green for
those later-owned rows.

### 3.1 Corrected GRID-005 Evidence

The old GRID-005 prose says a three-column template area with
`grid-auto-columns: 40px` and `grid-auto-rows: 20px` should intrinsically size
to `10x10`. That expected value is wrong. CSS Grid Level 2 says area-created
explicit tracks that lack an explicit track size are sized by the corresponding
`grid-auto-*` pattern. The repository's pinned Chrome
`149.0.7827.115` reports `120x20` for that exact scenario.

The live defect is topology, not a requirement to ignore `grid-auto-*` sizing:
the template dimensions must create explicit track identities even with no
items; negative line numbering and named placement must use those explicit
edges; and each area-only track must receive the correct `grid-auto-*` sizing
pattern. Incidental track growth caused by an item does not close GRID-005.

## 4 FRI-08.4 Normative Product Authorities

The normative product sources are:

- CSS Grid Layout Module Level 2 sections 5.3, 7.1 through 7.7, 8.5, 9, 11,
  and 12: `https://www.w3.org/TR/css-grid-2/`;
- CSS Grid Layout Module Level 3 Working Draft sections 2 through 6, using the
  published 21 January 2026 draft at specification time:
  `https://www.w3.org/TR/css-grid-3/`;
- CSS Box Alignment Level 3 for alignment terminology and safe overflow:
  `https://www.w3.org/TR/css-align-3/`; and
- CSS Overflow Level 3 for grid scrollable overflow:
  `https://www.w3.org/TR/css-overflow-3/`.

Level 3 is a Working Draft. FRI-08 implements only the already-representable
grid-lanes behavior named by this contract. A later draft change does not
silently broaden or mutate this initiative; it requires a separately reviewed
specification update.

The relevant normative anchors are explicit:

- automatic placement creates implicit rows or columns as needed and uses
  order-modified document order;
- auto-fit first behaves like auto-fill, then ordinary grid collapses repeated
  tracks that no in-flow item occupies or spans;
- an area-created explicit track without a corresponding explicit size uses the
  `grid-auto-*` sizing pattern;
- fit-content participates as max-content until its argument becomes the limit,
  while flexible tracks still resolve in the later flexible-track phase;
- stretch expands every track with an auto maximum using positive definite free
  space, regardless of its minimum;
- a grid-lanes item containing block is its grid area in the grid axis and the
  container content box in the stacking axis;
- auto-positioned grid-lanes items contribute to every candidate placement, and
  nested grid-lanes subgrid descendants receive every candidate projection plus
  the largest applicable edge contribution; and
- grid-lanes placement and ordinary grid placement both consume
  order-modified document order.

## 5 FRI-08.5 Resolved Design Decisions

| ID | Decision |
| --- | --- |
| `D-01` | Introduce one private `ExpandedGridTopology<S>`-equivalent owner for both axes. It retains expanded sizing functions, explicit start/count, auto-fit membership, auto-track pattern phase, named-line context, area facts, and placed integer areas. The public track and placement types do not become a second topology. |
| `D-02` | Ordinary-grid placement completes before track sizing. It operates on integer lines and a growable occupancy structure, not resolved scalar sizes. Geometry is materialized from settled track offsets only after placement, collapse, and sizing. |
| `D-03` | Explicit topology in each axis is the maximum of the explicitly sized track-list count and the valid template-area dimension. Area-only explicit tracks are sized from the corresponding `grid-auto-*` pattern but remain explicit for positive/negative line numbering and named placement. |
| `D-04` | Definite overlapping items mark the same cells and never create demand proportional to their count. Automatic placement grows the implicit grid exactly when its cursor/span cannot fit; it never returns an out-of-range zero area for a valid representable placement. |
| `D-05` | Before automatic placement, the non-flow axis grows for definite placements and the largest automatic span as required by Grid section 8.5. During placement, the flow axis grows monotonically. Leading and trailing implicit tracks preserve the existing forward/backward `grid-auto-*` pattern phase. |
| `D-06` | `display:none` and absolute children retain source slots but neither occupy cells nor create implicit demand. Ordinary in-flow children are traversed by the one stable `item_order_permutation`; dense mode may backfill cells but never changes source association. |
| `D-07` | Ordinary-grid auto-fit expands to the full auto-repeat count, participates in placement, then collapses exactly repeated tracks not occupied or spanned by an in-flow item. Collapsed tracks have zero fixed size, adjacent gutters collapse, and line identity remains. |
| `D-08` | Grid-lanes auto-fit uses the Level 3 pre-placement heuristic: explicitly occupied tracks remain, then the first `N` otherwise-unoccupied candidate tracks remain where `N` is the sum of automatic item spans; all other auto-fit tracks collapse and reject automatic placement. It does not reuse ordinary-grid post-placement occupancy or raw child count. |
| `D-09` | Fit-content is a per-track growth limit inside the general track sizing pipeline. No collection-wide fit-content early return is allowed. Other intrinsic and flexible tracks execute their ordinary phases, including `fr` expansion in definite space. |
| `D-10` | The track solver retains distinct base size and growth limit facts through intrinsic contribution distribution. Fit-content behaves as max-content until its argument limits further growth. Spanning contributions and flexible expansion consume the same settled facts in rows and columns. |
| `D-11` | Content-distribution normal/stretch expands every track whose maximum is `Auto`, including `minmax(fixed,auto)` and `minmax(intrinsic,auto)`. Only positive definite remaining space is divided, after gaps, fixed use, flex use, and intrinsic base sizes. The minimum remains a floor. |
| `D-12` | Named-line occurrence lookup counts a matching name at most once per grid line. Duplicate source tokens and collisions among explicit, inherited, local-subgrid, and area-generated origins may retain origin evidence, but lookup, named spans, and positive/negative occurrence arithmetic use a deduplicated ordered line set. |
| `D-13` | Template-area validation remains typed and axis-local. Invalid area syntax retains the existing named-grid diagnostic behavior and cannot enlarge topology. Valid area facts, track counts, named lines, placement, and subgrid clipping are all derived from the same canonical dimensions. |
| `D-14` | A grid-lanes item's containing block is hybrid: the selected track area in the grid axis and the container content box in the stacking axis. Parent size, percentage resolution, self-alignment, physical projection, and RTL/vertical reversal all consume that same hybrid box. |
| `D-15` | Grid-lanes intrinsic sizing keeps definite items at their exact grid-axis span and projects every automatic item across every possible start for its span. Aggregation may use virtual groups only when span, placement candidates, baseline-sharing role, and edge facts are equivalent. |
| `D-16` | A nested indefinite grid-lanes subgrid is flattened into descendant contribution groups. Each descendant is projected into every parent track it could occupy, with the maximum applicable start/end margin, border, padding, and half-gap facts. The parent never drops the lower bound or substitutes the subgrid wrapper's zero geometry. |
| `D-17` | Remove the public `LaneIntrinsicItemKind::NestedIndefiniteSubgrid` state, its constructor, and `LanePlacementError::NestedGridLanesSubgridIndefiniteUnsupported`. The ordinary definite/indefinite virtual-item contract is sufficient after production flattening. This breaking removal prevents a layout-ready request from encoding an unsupported algorithm branch. |
| `D-18` | In inherited-subgrid intrinsic traversal, a standalone axis is a measurement boundary, not an error. The standalone grid container contributes its measured minimum/min-content/max-content margin box as one leaf across its translated parent span; descendants are resolved inside its ordinary local grid and are not flattened across that boundary. |
| `D-19` | The FRI-06 inherited-axis baseline model remains authoritative. FRI-08 verifies it with new topology, sizing, overflow, order, and writing-mode cases. It does not recreate baseline groups or implement Level 3 stacking-axis baseline alignment, which the current Level 3 draft explicitly does not support. |
| `D-20` | Grid and subgrid feed final placed physical border boxes and descendants into the canonical FRI-05 scroll-contribution owner. Track-local coordinates, area-relative offsets, and inherited-axis views never replace container-relative physical geometry. Scrollable overflow must not change intrinsic eligibility unless the normative automatic-minimum rule requires it. |
| `D-21` | Existing `FlowAxes`, `ItemOrder`, normalized overflow, replaced state, cache identity, completed-batch transaction, and source-index contracts remain sole owners. FRI-08 adds no parallel axis switch, order sort, overflow rectangle, replaced heuristic, cache key, or partial publication path. |
| `D-22` | The final initiative cycle assesses the complete FRI-08 implementation range for architecture sprawl and implements every confirmed in-initiative consolidation after the behavior/artifact candidate is published. It changes no behavior, public API, fixture membership, generated lineage, dependency, feature, or finding ownership unless review proves a genuine defect and reopens its owning contract. |

Rejected alternatives:

- Preallocating by child count, total span area, or `div_ceil` cannot represent
  holes, overlap, dense backfill, definite-major placement, or cursor growth.
- Resolving placement from zero-valued fake tracks retains the fixed-capacity
  failure and confuses topology with size.
- Truncating auto-fit to item count fails overlap and spanning; deleting line
  identities after collapse breaks named and negative placement.
- Running a separate fit-content solver loses flexible and spanning phases.
- Treating only `auto/auto` as stretchable contradicts the max-track rule.
- Deleting all duplicate origin entries at parse time would discard provenance
  needed by subgrid area-name recomputation; lookup deduplication is sufficient.
- Measuring a rows-only lanes item against its own tentative width creates a
  percentage cycle and cannot provide the container content-box basis.
- Treating a standalone-axis descendant as inherited leaks its descendants into
  ancestor tracks; rejecting it leaves valid Level 2 behavior unsupported.
- Treating a nested lanes subgrid as one ordinary zero wrapper loses descendant
  candidate placement and edge contributions.
- Making every current grid parity failure FRI-08 work would absorb FRI-09
  baseline distribution and FRI-10 positioned-layout ownership.

## 6 FRI-08.6 Public Model And Compatibility

FRI-08 adds no public input field, output field, scalar alias, trait,
dependency, feature, or MSRV change. Existing public values already represent
the required ordinary-grid, template-area, auto-fit, named-line, subgrid,
grid-lanes, order, writing-mode, and overflow inputs.

The one public breaking correction is the removal specified by `D-17`:

```text
LaneIntrinsicItemKind::NestedIndefiniteSubgrid
LaneIntrinsicItemOf::nested_indefinite_subgrid(...)
LanePlacementError::NestedGridLanesSubgridIndefiniteUnsupported
```

`LaneIntrinsicItemKind` retains only definite and indefinite placement facts.
Callers that previously constructed the removed state lower an already
aggregated virtual contribution through `LaneIntrinsicItemOf::indefinite`.
Production does not use that convenience as a substitute for descendant
flattening; it derives the full candidate and edge matrix from the tree.

`SubgridTraversalError::StandaloneSubgridTraversalUnsupported` is private and
is removed. Invalid spans, missing intrinsic-min facts, invalid named input,
missing percentage bases, provider failures, and non-finite values retain their
existing typed error categories and transaction guarantees.

No compatibility boolean, deprecated constructor, silent conversion, zero-box
fallback, or catch-all unsupported value is introduced. Root must update any
direct reference to the removed public variants before gitlink promotion.

## 7 FRI-08.7 Canonical Topology And Placement

### 7.1 Topology Construction

For each axis, topology construction performs these steps exactly once:

1. validate track components and the template-area matrix;
2. expand fixed and automatic repetitions without a child-count auto-fit cap;
3. compute `sized_explicit_count` from the expanded template track list;
4. compute `area_explicit_count` from the valid area matrix;
5. set `explicit_count = max(sized_explicit_count, area_explicit_count)`;
6. append any area-only explicit track identities, sourcing their sizing
   functions from the axis's `grid-auto-*` pattern;
7. build named lines and area-generated names against that exact explicit
   count; and
8. retain auto-fit origin and auto-track phase metadata for every track.

An empty valid three-column area template therefore has three explicit column
tracks even with no children. If the column auto pattern is `[40px, 20px]`,
the area-only sizes are `[40px, 20px, 40px]`. Positive line 1, negative line
-1, `foo-start`, and `foo-end` all refer to the same explicit edges used by
placement and geometry.

If explicit sized tracks exceed the area dimension, the larger track list
remains authoritative and the area-generated lines occupy their specified
subset. Invalid or empty area facts create no tracks.

### 7.2 Placement Demand

The placement phase owns integer half-open areas
`[column_start,column_end) x [row_start,row_end)`. It does not receive resolved
track sizes. Its occupancy storage grows when implicit tracks are appended and
preserves all previously marked cells.

For row flow:

1. place fully definite items and mark their areas, allowing overlap;
2. process definite-row items in order-modified order;
3. establish the implicit column range from explicit columns, every definite
   column placement, and the largest unresolved column span;
4. place remaining items with the sparse or dense cursor, appending implicit
   rows whenever no non-overlapping position fits; and
5. record the final integer area for every in-flow item.

Column flow swaps the axes. Leading implicit tracks are included in the same
line coordinate system; prepending does not invalidate named explicit edges or
the phase of the repeating auto pattern.

The canonical GRID-001 cases are:

| Case | Required result |
| --- | --- |
| Three `40px` columns, middle cell occupied, later automatic span-two item | Add a second `20px` row; root `120x40`; item area columns 1-3 on row 2, geometry `(0,20) 80x20` |
| One `10x10` explicit cell with two definite overlapping children | Keep exactly one row and column; root `10x10`; both children share the same area |
| Automatic span wider than initial implicit width | Grow the non-flow axis before cursor placement; never clamp the valid span to zero |
| Dense item filling an earlier hole | Reuse the hole without changing source identity or shrinking already-created topology |
| Absolute or display-none child outside explicit lines | Preserve its source slot, but create no in-flow occupancy or implicit demand |

No valid in-flow placement returns a sentinel outside the grid. If a configured
implementation limit is later introduced, it requires a typed, reviewed limit
contract; FRI-08 does not silently clamp to a one-cell zero area.

### 7.3 Geometry Materialization

After placement, ordinary auto-fit collapse and track sizing settle the scalar
track sizes and offsets. Each recorded integer area is then materialized using
those offsets and active gutters. This separation guarantees that:

- placement never depends on intrinsic measurement order;
- scalar type does not change occupancy;
- collapsed tracks preserve line numbering while contributing zero size;
- subgrid child context receives settled inherited tracks; and
- child output, baseline collection, overflow, and source association consume
  the same area.

## 8 FRI-08.8 Auto-Fit, Track Sizing, And Stretch

### 8.1 Ordinary Auto-Fit

Every track created by `repeat(auto-fit, ...)` retains a repeat identity.
After ordinary-grid placement, a repeated track is occupied if any in-flow
placed area covers it. A track spanned by an item is occupied even when the
item begins or ends outside that repetition. Definite overlap does not increase
the occupied-track count.

Every unoccupied auto-fit track becomes collapsed:

- used min and max are a fixed zero;
- intrinsic contributions do not reopen it;
- the gutter on either side collapses when it would border a collapsed track;
- content distribution treats it as zero fixed space; and
- its grid lines and names remain addressable.

In the GRID-006 repro, three `40px` repetitions fit in `120px`; two children
overlap the first repetition; only that track remains non-collapsed and its
`40px` subject centers at x `40`.

Grid-lanes uses `D-08` instead because its placement consumes already-sized
tracks. Tests distinguish the two algorithms with explicit overlap, automatic
spans, holes, all-empty repetitions, named lines, and both flow axes.

### 8.2 Unified Track State

Each non-collapsed track carries at least:

```text
sizing functions
base size
growth limit
flex factor, when any
fit-content argument, when any
auto-max stretch eligibility
```

Intrinsic non-spanning and spanning phases update base sizes and growth limits.
Fit-content maximums participate as max-content until their argument caps the
growth limit. Flexible expansion then resolves every flex track against the
remaining definite space. A fit-content track in the same axis cannot end that
pipeline early.

For `[fit-content(50px), 1fr]` in a definite `200px` axis with settled
intrinsic bases `[20,0]`, the result is `[20,180]`. Equivalent row-axis,
vertical-writing, f32, and f64 cases must agree after physical projection.

Spanning-item tests cover the currently failing checked-in fit-content and
non-flex span families. FRI-08 fixes the general contribution phases; it does
not add fixture-name branches for those sources.

### 8.3 Stretch

After intrinsic and flexible sizing, calculate:

```text
remaining = definite_inner_size - active_gaps - sum(used_track_sizes)
```

If content distribution resolves to normal/stretch and `remaining > 0`, divide
it equally among every non-collapsed track whose maximum is `Auto`. Add that
share to the settled base size while preserving the minimum floor. Otherwise,
add zero.

The eligibility set includes `auto`, `minmax(0,auto)`,
`minmax(min-content,auto)`, and `minmax(max-content,auto)`. It excludes fixed,
fit-content, min-content-max, max-content-max, flexible, and collapsed tracks.
One `minmax(0,auto)` track in `100px` therefore resolves to `100px`.

## 9 FRI-08.9 Named Lines And Template Areas

Named-line lookup builds an ordered list of matching line numbers. For each
line, membership is boolean: any number of same-name entries on that line adds
that line exactly once. Positive occurrences walk start to end; negative
occurrences walk end to start; missing occurrences extend into the implicit
grid according to the existing typed rules.

For `[a a] 40px [a] 40px`, the matching line list is `[1,2]`; `a 2` resolves
to line 2 and x `40`. The same rule applies to named spans and to collisions
between explicit and area-generated or inherited names on one line.

Origin tags remain available for:

- distinguishing explicit, inherited, local-subgrid, and area-generated facts;
- clipping and recomputing area-generated names at subgrid boundaries; and
- source-order diagnostics.

They never multiply occurrences. A structural test proves no occurrence lookup
uses `flat_map` over raw entries without per-line deduplication.

Template-area facts are committed only after rectangular validation. Their
dimensions feed topology before named placement. Area-start/end names and
numeric positive/negative lines therefore cannot observe different explicit
counts. Tests cover areas larger and smaller than sized track lists, empty
containers, null cells, invalid rectangles, local/inherited name collisions,
subgrid clipping, both directions, and both scalar lanes.

## 10 FRI-08.10 Grid-Lanes Containing Blocks And Intrinsic Sizing

### 10.1 Hybrid Containing Block

Let `grid_area_axis` be the settled span in the grid axis and
`stacking_content_axis` be the grid-lanes container's content-box extent in the
stacking axis. Each in-flow lanes child receives a physical containing size
formed by projecting those two logical components through the container's
`FlowAxes`.

That same containing size is used for:

- percentage margin, padding, preferred, min, and max resolution;
- stretch and aspect-ratio preflight;
- intrinsic min/max measurement where the axis is definite;
- final child layout and subgrid child context; and
- RTL, vertical, and sideways physical placement.

The tentative lane-axis margin-box size is output from measurement, never its
own parent percentage basis. In the rows-only
`grid_lanes_item_containing_block_content_width` source, the container's
definite inline content width is `100px`; a `width:100%` child is `100px` in
both directions and RTL does not shift a zero-width substitute to x `100`.

### 10.2 Intrinsic Projection

The Level 3 intrinsic projection is finite:

1. measure each eligible child under min-content and max-content constraints
   using the hybrid contextual axes known at that phase;
2. retain minimum, min-content, max-content, automatic-minimum, MBP, span,
   candidate-start, and baseline-sharing facts;
3. group only equivalent facts and take componentwise maxima;
4. create virtual contributions at every candidate start; and
5. run the ordinary track contribution phases on those virtual items.

Definite placement has one candidate. Automatic placement has every start at
which its span fits. The result supplies both track lower bounds and the
container's min/max-content grid-axis sizing. It must close the checked-in
`grid_lanes_min_content_container_sizing` and
`grid_lanes_max_content_container_sizing` families without source-specific
offset rules.

### 10.3 Nested Indefinite Grid-Lanes Subgrid

Production collection descends through a grid-lanes subgrid in the grid axis.
For every descendant that can contribute:

- translate its local span to every parent candidate placement;
- preserve whether its local placement is definite or automatic;
- accumulate half-gap differences independently from physical MBP edges;
- for each possible start/end edge, retain the largest applicable edge fact;
- group equivalent translated candidates; and
- feed their virtual contributions into the same ordinary track solver.

If the subgrid itself is definitely placed, candidates are limited to its
spanned parent tracks. If it is automatically placed, candidates cover all
parent tracks where its span can fit. An automatic descendant is considered at
every possible track within those candidates. No placement result feeds back
into intrinsic sizing.

Provider failure, invalid value, non-finite measurement, and transaction tests
prove that flattening is fallible and publishes no partial batch. Cache cold
and warm runs consume identical projection facts.

## 11 FRI-08.11 Level 2 Subgrid Boundary And Composition

### 11.1 Standalone-Axis Boundary

Inherited-subgrid traversal continues to flatten descendants only while the
queried axis is inherited. When it reaches a node whose queried axis is
standalone:

1. stop ancestor-line translation at that node;
2. measure the standalone grid container under the contextual constraint
   supplied by its parent span;
3. create one leaf contribution for its margin box using minimum, min-content,
   and max-content results as required by the active sizing phase;
4. apply the accumulated outer MBP and half-gap facts once; and
5. do not expose the standalone node's internal tracks or children to the
   ancestor's inherited-axis solver.

The standalone node still performs ordinary local grid layout for its children.
Its other axis may remain inherited, and each axis applies this rule
independently. Reversal and writing-mode mapping occur at contextual boundaries
through `FlowAxes`, never by swapping source fields.

### 11.2 Baseline Preservation

FRI-08 changes topology and sizing beneath the FRI-06 baseline pipeline. It must
preserve:

- immutable ancestor baseline groups;
- direct-owner and inherited-current-grid target consumption;
- first/last role separation;
- half-gap, MBP, reversal, and owner/local track mappings;
- horizontal, vertical, and sideways physical projection; and
- no publication inverse or sizing fixed point.

New FRI-08 cases combine area-created tracks, implicit growth, auto-fit,
standalone-axis boundaries, and lanes subgrids with baseline controls. They are
regression evidence only. Ordinary baseline distribution failures remain
FRI-09.

### 11.3 Overflow And Scroll Composition

Grid-specific overflow closure uses the FRI-05 canonical contribution API.
Every in-flow child contributes its final container-relative physical geometry,
including inherited-subgrid descendants. Overflow visible/hidden/clip/scroll/
auto affects automatic minimums and propagation only through the published
normalized rules.

The required checked-in controls are:

- `grid/grid_overflow_inline_axis_scroll`;
- `subgrid/subgrid_overflow_hidden_does_not_prohibit`;
- `subgrid/subgrid_sibling_overflow_footer_second_matches_first`; and
- `subgrid/subgrid_sibling_overflow_footer_third_matches_first`.

Their 16 variants must pass without changing browser expectations, erasing
negative or reversed ranges, double-counting area origin, or adding a
grid-specific scroll rectangle.

## 12 FRI-08.12 Composition Matrix

Focused public-front-door evidence covers at least this matrix:

| Dimension | Required cases |
| --- | --- |
| Placement | definite overlap, definite-major, all-auto, span, sparse, dense, leading/trailing implicit, row/column flow |
| Topology | no explicit list, area-only, list larger/smaller than areas, empty grid, auto-pattern phase, negative lines |
| Auto-fit | ordinary occupancy, overlap, spanning, all empty, named lines, lanes heuristic, both axes |
| Sizing | fixed, percentage, auto, min/max-content, fit-content, flex, mixed spans, stretch, definite/indefinite constraints |
| Names | duplicate tokens, positive/negative occurrence, named span, area-generated collision, fixed repeat, subgrid inheritance |
| Lanes | rows-only/columns-only, percentage child, min/max-content container, definite/automatic span, order, tolerance, nested subgrid |
| Subgrid | one/both inherited axes, standalone boundary, nested depth, reversal, unequal gaps, MBP, area names, baseline controls |
| Flow | all five writing modes and both directions, parallel/opposing child flow, row/column logical axes |
| Participation | source order, signed item order, display none, absolute control, replaced/non-replaced, source index |
| Overflow | visible, hidden, clip, scroll, auto, nested propagation, reversed range, scrollbar settling |
| Reliability | f32/f64, cold/warm cache, invalidation, provider failure, non-finite value, rollback, deterministic rounding |

Properties and oracle tests vary dimensions independently and in bounded pairs;
they do not attempt a Cartesian explosion. Every finding has one direct minimal
repro plus at least one composed public-front-door case.

The following negative controls remain unchanged:

- ordinary grid baseline-distribution failures assigned to FRI-09;
- grid-aligned absolute percentage/static-position failures assigned to FRI-10;
- stacking-axis grid-lanes baseline alignment, which the selected Level 3 draft
  does not support;
- fragmentation; and
- default non-grid layout.

## 13 FRI-08.13 Browser And Artifact Contract

### 13.1 Exact Owned Source Inventory

FRI-08 owns exactly these eighteen four-variant sources.

Ten are new:

1. `grid/fri08_auto_placement_span_after_occupied.html`;
2. `grid/fri08_explicit_overlap_no_implicit_growth.html`;
3. `grid/fri08_fit_content_flex_composition.html`;
4. `grid/fri08_template_areas_explicit_tracks.html`;
5. `grid/fri08_auto_fit_occupied_track_collapse.html`;
6. `grid/fri08_stretch_minmax_auto.html`;
7. `grid/fri08_duplicate_line_name_token.html`;
8. `grid/fri08_grid_composition.html`;
9. `grid-lanes/fri08_nested_indefinite_subgrid.html`; and
10. `subgrid/fri08_standalone_intrinsic_composition.html`.

Eight are existing checked-in sources adopted as FRI-08 acceptance controls:

11. `grid/grid_overflow_inline_axis_scroll.html`;
12. `grid-lanes/grid_lanes_item_containing_block_content_width.html`;
13. `grid-lanes/grid_lanes_min_content_container_sizing.html`;
14. `grid-lanes/grid_lanes_max_content_container_sizing.html`;
15. `subgrid/subgrid_overflow_hidden_does_not_prohibit.html`;
16. `subgrid/subgrid_sibling_overflow_footer_second_matches_first.html`;
17. `subgrid/subgrid_sibling_overflow_footer_third_matches_first.html`; and
18. `subgrid/subgrid_standalone_axis_column_autoflow.html`.

This is 72 owned source/variant rows. Only the ten new sources add outputs, so a
single full generation from the immutable base inventory produces 5,776 XML
outputs if no independently authorized source changes first. A base drift
requires the just-in-time plan to recompute counts from the reviewed source set;
it does not permit changing the eighteen-source identity without a reviewed
specification amendment.

Every source uses ordinary authored CSS for the browser oracle and explicit
layout-ready facts for the Rust adapter. The helper may serialize only the
computed values already represented by public grid input. The adapter accepts a
finite token/value subset and rejects unknown explicit values; it never selects
facts from source name, expected geometry, or variant identity.

### 13.2 Generation And Provenance

Inputs and production behavior settle before one unfiltered full
existing-pinned generation. Filtered generation may be used only as a
report-free diagnostic during an owning cycle. It is not acceptance evidence.

The full run must:

- use the manifest-owned existing Chrome pin and launch profile without fetch;
- write the sole schema-versioned `all.json` provenance authority;
- produce comment-free XML;
- preserve global and per-output source/resource/XML hashes;
- prune only according to the authoritative generator;
- retain exactly the 16 unrelated missing-root unsupported variants;
- retain the three already-proven FRI-07 expected-fail source records unless
  their mandatory revalidation trigger fires; and
- add no FRI-08 expected fail, quarantine, or failed-to-generate record.

Pinned Chrome is the default oracle. A new expected fail would require the full
existing certainty, standards/WPT corroboration, synthetic public-front-door
substitute, exact reason, trigger, and independent review contract. No current
FRI-08 row is authorized for that exception.

After the full run, browser-free corpus, Taffy, report-lineage, exact inventory,
source hash, linked-resource hash, XML hash, no-comment, and focused parity
checks are the artifact evidence. Manual XML editing is forbidden.

## 14 FRI-08.14 Architecture And Sprawl Invariants

The completed initiative must satisfy all of these structural invariants:

1. one topology owner connects explicit counts, names, areas, placement,
   auto-fit identity, and track sizing;
2. no child-count, total-cell-count, or `div_ceil` preallocation determines
   final implicit demand;
3. no valid placement returns sentinel zero geometry because capacity was
   guessed too small;
4. one order-modified permutation is consumed by ordinary grid and the existing
   source-order rule remains explicit for any lanes phase where the normative
   algorithm requires it;
5. no collection-wide fit-content early return exists;
6. stretch eligibility is expressed by auto maximum, not one exact min/max
   pair duplicated across resolvers;
7. named occurrence lookup deduplicates per line in one owner;
8. ordinary and lanes auto-fit are separate named policies over shared track
   metadata, not implicit conditionals on display scattered through sizing;
9. standalone subgrid boundaries and inherited traversal are explicit variants,
   not errors or boolean history flags;
10. nested lanes subgrid projection reuses ordinary contribution state rather
    than a public unsupported item kind;
11. `FlowAxes`, canonical scroll contributions, cache identity, and completed
    batches retain sole ownership;
12. no fixture source/name/expected geometry branch exists in production or
    adapter code;
13. no new lint suppression, dependency, feature, unsafe code, generator path,
    or provenance authority exists; and
14. source modules remain responsibility-shaped: topology/placement, named
    lines, tracks, subgrid traversal, lanes, child layout, and tests do not
    duplicate one another's state machines.

After the behavior/artifact candidate is published, a fresh holistic sprawl
assessment reviews the complete FRI-08 implementation range. Every finding is
source-validated. Confirmed in-initiative mechanical consolidation is
implemented with characterization evidence; disproven or later-owned findings
receive exact dispositions. A final fresh holistic review follows that cycle.

## 15 FRI-08.15 Verification And Error Contract

Every implementation cycle uses RED/GREEN TDD for changed behavior. A RED test
must fail for the intended semantic reason on the assignment base. Refactors
without a behavioral RED first capture equivalent characterization.

Focused verification includes:

- direct topology and placement tests;
- track-solver and named-line unit tests;
- production/oracle comparison tests;
- public `compute_layout` composition tests;
- proptests for occupancy monotonicity, occurrence uniqueness, track floors,
  and scalar equivalence;
- exact removed-capability compile/API tests;
- focused browser-parity tests for all 72 owned rows;
- browser-free corpus/report/Taffy validation; and
- default and generator feature verification.

The final candidate runs at least:

```sh
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
cargo fmt --check
git diff --check
```

It also runs exact focused parity for the eighteen-source inventory and records
the later-owned baseline/positioned negative controls separately. Repository-
wide `just parity-all` remains FRI-13's aggregate release gate and is diagnostic
in FRI-08; FRI-08 cannot claim unrelated failures or hide its own.

For both f32 and f64, every valid representable request returns correct finite
geometry. Invalid values, required missing bases, provider failures, and
internal invariant failures return the existing typed error envelope with no
completed batch, partial tree state, or cache mutation. No GRID-001/005/010
case is converted into zero geometry, ignored lower bounds, or an unsupported
bucket.

The canonical repository-owned Rust unsafe scan covers every tracked and
non-ignored Rust source, including generated or untracked sources when present.
Every textual occurrence is classified; no unsafe code, unsafe block, unsafe
trait/impl, or unsafe lint relaxation is allowed.

## 16 FRI-08.16 Expected Source Responsibilities

Expected implementation areas are responsibility boundaries, not permission to
edit every file:

| Area | Responsibility |
| --- | --- |
| `src/grid/mod.rs` | Orchestrate canonical topology, placement-before-sizing, inherited contexts, and public computation without duplicating algorithms |
| `src/grid/placement.rs` | Growable integer occupancy, cursor placement, implicit demand, and post-sizing area materialization inputs |
| `src/grid/tracks.rs` | Expanded track metadata, fit-content/base/growth-limit phases, flexible expansion, auto-max stretch, and auto-fit collapse sizes |
| `src/grid/named.rs` | Shared explicit counts, area facts, origin-aware per-line membership, occurrences, and subgrid name projection |
| `src/grid/lanes.rs` | Hybrid containing blocks, Level 3 candidates, virtual intrinsic groups, lanes auto-fit policy, and removal of public unsupported state |
| `src/grid/subgrid.rs` | Inherited traversal, standalone measurement boundary, edge/gap facts, and retained FRI-06 baseline views |
| `src/grid/child.rs` | Consume settled areas, baseline controls, canonical scroll contribution, and physical publication |
| `src/node_input.rs`, `src/lib.rs`, `README.md` | Public removal/docs and unchanged normalized grid boundary; no new authored CSS model |
| `src/grid_tests.rs`, `tests/layout/browser_parity.rs` | Focused, oracle, property, scalar, composition, lineage, and negative-control evidence |
| generator/helper/HTML/manifest/XML/report | Exact finite adapter and artifact work in `FRI-08.13`, only through the authoritative generator |

Any implementation that needs a new cross-module carrier must place it with the
module that owns its invariants. A cycle plan names exact file ownership and
forbids unrelated cleanup.

## 17 FRI-08.17 Finding Closure Matrix

| Finding | Required closure evidence |
| --- | --- |
| `GRID-001` | Growable placement produces the span-after-hole row and overlap-without-growth results; no count estimate or zero-area fallback remains |
| `GRID-002` | Rows-only lanes percentage child receives the definite stacking-axis content box and all four existing variants pass in LTR/RTL and both box modes |
| `GRID-003` | Mixed fit-content/flex tracks execute one solver and produce `[20,180]`; checked-in fit-content/span families pass without fixture dispatch |
| `GRID-005` | Valid template areas establish explicit topology with correct `grid-auto-*` sizing, empty-grid behavior, line edges, names, and mixed list dimensions; the stale `10x10` expectation is explicitly rejected |
| `GRID-006` | Ordinary auto-fit collapses from placed occupancy and lanes auto-fit uses its Level 3 heuristic; overlap, spans, empty tracks, gaps, and line identity pass |
| `GRID-007` | Every auto-max track stretches from positive definite free space while its minimum remains a floor, in both axes and scalar lanes |
| `GRID-008` | Duplicate same-line tokens resolve as one occurrence for named lines and spans across explicit, area, inherited, and local origins |
| `GRID-010` | Standalone inherited traversal and nested indefinite lanes subgrid sizing are supported; published FRI-06 baseline behavior survives; owned lanes/subgrid sizing and overflow rows pass; no unsupported branch or exaggerated broad Level 2/3 claim remains |

Closure is public-front-door behavior plus architecture and artifact evidence.
Changing a private helper test alone is insufficient.

## 18 FRI-08.18 Publication And Root Handoff

Each completed cycle is committed, reviewed, accepted, published to authority
remote `main` with an exact compare-and-swap lease, and read back before the
next cycle. The artifact cycle publishes the sole full-regeneration candidate
before the sprawl cycle begins. The final initiative candidate is published and
remotely verified only after task review, coordinator acceptance, fresh
holistic review, final verification, and clean status.

The final leaf handoff records:

- exact candidate SHA and remote readback;
- the removal of the three public nested-indefinite unsupported symbols;
- unchanged public normalized grid inputs and outputs otherwise;
- closure evidence for all eight findings;
- the exact eighteen-source/72-row inventory;
- report, manifest, helper, and owned XML hashes and bucket counts;
- later-owned baseline and positioned-layout negative controls;
- validated sprawl dispositions; and
- the no-unsafe, no-acquisition, no-dependency/feature/MSRV-change results.

Root integration separately updates direct API use, facade artifacts, and the
gitlink. Root must not infer authored CSS from the leaf's finite fixture adapter
or copy private topology types across the crate boundary.

## 19 FRI-08.19 Product Acceptance

FRI-08 is complete only when all of the following are true:

1. all eight findings have the exact closure in `FRI-08.17`;
2. explicit topology is identical for track sizing, names, areas, positive and
   negative lines, placement, and subgrid inheritance;
3. every valid automatic placement receives an in-range integer area and exact
   implicit demand, including overlap, holes, spans, order, and dense flow;
4. ordinary and lanes auto-fit each use their specified occupancy policy with
   retained line identity and collapsed gutters;
5. fit-content, flex, spans, and auto-max stretch compose in both axes and both
   scalar lanes without an early alternative solver;
6. named occurrences count matching lines, not duplicate tokens;
7. grid-lanes children receive the hybrid containing block and nested
   indefinite subgrid descendants contribute across all candidates;
8. standalone subgrid axes terminate ancestor traversal with an ordinary
   measured leaf instead of an unsupported result;
9. FRI-06 baseline, FRI-05 overflow, FRI-03 order/replaced, FRI-02 flow, FRI-01
   error/transaction, and cache contracts remain sole owners and pass composed
   controls;
10. the exact 72 owned browser rows pass with honest input-derived lowering,
    no new expected fail/quarantine/failure bucket, comment-free XML, and
    `all.json` as the only provenance authority;
11. later FRI-09 and FRI-10 rows remain visible and separately owned rather than
    being suppressed or claimed;
12. every sprawl finding has a validated disposition and all `FRI-08.14`
    invariants hold;
13. default/generator verification, corpus/Taffy checks, strict Clippy,
    formatting, diff, unsafe, scope, and clean-worktree gates pass; and
14. the final leaf candidate is present on the authority remote with complete
    readback and root handoff evidence.
