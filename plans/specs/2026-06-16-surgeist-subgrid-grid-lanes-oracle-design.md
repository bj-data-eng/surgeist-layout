# Surgeist Subgrid And Grid-Lanes Oracle Design

## Purpose

This spec extends the base grid oracle design into the parts of CSS Grid that cannot be planned independently: base grid, subgrid, and grid-lanes. These features share placement, track sizing, axis mapping, intrinsic contribution, gap, margin, border, padding, and alignment behavior. The oracle must make those interactions inspectable without becoming a hidden browser engine.

The design uses WebKit's current grid implementation as inspiration, especially:

- `RenderGrid.cpp` for feature predicates, placement invalidation, and the subgrid/grid-lanes relationship.
- `GridTrackSizingAlgorithm.cpp` for intrinsic sizing phases and nested subgrid traversal.
- `GridMasonryLayout.cpp` for grid-lanes placement as a separate layout path.
- `GridLayoutFunctions.cpp` for flow-aware axis mapping, reversed subgrid direction, and extra margins from ancestor subgrids.
- `AncestorSubgridIterator.cpp` for ancestor subgrid traversal.

The oracle should not mirror WebKit's architecture. WebKit carries browser production concerns that Surgeist should keep out of test-only oracle code: render tree mutation, style adjustment, invalidation, line-name inheritance machinery, baseline caches, scroll containment, and legacy compatibility paths.

## Core Principle

The oracle answers:

```text
Given explicit grid, subgrid, or grid-lanes phase inputs, what should this phase produce?
```

It does not answer:

```text
Given a styled tree and browser-compatible CSS, what should every node's layout be?
```

Browser parity fixtures and `OracleTree` can compare production layout to oracle outputs. The oracle itself must stay phase-local, typed, explicit, and report-rich.

## Relationship To The Base Grid Oracle

The existing base grid oracle remains the foundation:

- `placement` owns numeric line/span resolution, auto-placement, cursors, occupancy, dense backfill, and implicit track expansion.
- `tracks` owns track initialization, intrinsic growth, spanning growth, flexing, stretch, and solved track offsets.
- `contributions` owns explicit item contribution facts.
- `alignment` owns post-sizing offset and gap distribution.
- `scenario` owns small composed cases.

Subgrid and grid-lanes extend these phases; they do not replace them.

## Non-Goals

- Do not parse CSS `subgrid`, `grid-lanes`, line names, or shorthands inside the oracle.
- Do not traverse production `NodeInput` trees from oracle solvers.
- Do not call `compute_grid`, child measurement, text measurement, style resolution, or retained tree APIs from oracle solvers.
- Do not implement browser line-name inheritance in the first oracle increment.
- Do not implement baseline caches in the oracle. Baseline facts may be explicit inputs.
- Do not model every writing mode at once. Add axis mapping as explicit data first, then add parity matrices gradually.
- Do not implement grid-lanes by modifying base grid placement in place.

## File Shape

The current oracle modules should evolve without collapsing into a monolith:

```text
crates/surgeist/tests/support/oracle/grid/
  mod.rs
  alignment.rs
  contributions.rs
  placement.rs
  scenario.rs
  tracks.rs
  axis.rs
  subgrid.rs
  lanes.rs
```

### axis.rs

Owns logical axis vocabulary used by grid, subgrid, and grid-lanes.

Responsibilities:

- Map parent axes to child axes using explicit writing-mode facts.
- Model reversed direction as data, not by reaching into production style.
- Provide helpers that correspond to WebKit's `flowAwareDirectionForGridItem`, `flowAwareDirectionForParent`, and `isSubgridReversedDirection`.

Inputs:

- parent writing mode
- child writing mode
- parent inline direction
- child inline direction
- queried axis
- explicit parent flipped-state in the resolved parent axis
- explicit child flipped-state in the resolved child axis
- optional explicit reversed-axis override for focused tests

Outputs:

- child-local axis
- parent-local axis
- reversed-axis boolean
- axis mapping report

### subgrid.rs

Owns subgrid-specific oracle vocabulary and phase reports.

Responsibilities:

- Decide whether a child participates as a subgrid in a queried axis from explicit feature facts.
- Translate parent track spans into subgrid-local track spans.
- Copy inherited track sizes into subgrid track space.
- Apply subgrid margin, border, padding, and gap-difference adjustments.
- Traverse explicit nested subgrid descriptors for intrinsic sizing.
- Produce leaf contribution lists for parent track sizing.

The key WebKit-inspired rule is:

```text
A grid item is a subgrid in an axis only when:
1. its axis declaration requests subgrid,
2. it is not forced into independent formatting behavior,
3. it has a parent grid,
4. the parent is not grid-lanes/masonry in that same resolved parent axis.
```

The oracle should encode this as an explicit predicate, for example:

```rust
SubgridEligibility {
    requested: bool,
    has_parent_grid: bool,
    independent_formatting_context: bool,
    excluded_from_normal_layout: bool,
    parent_is_lanes_in_resolved_axis: bool,
}
```

The `parent_is_lanes_in_resolved_axis` fact is computed after `flowAwareDirectionForGridItem`-style axis mapping. Outputs should include both the boolean and the reason when false.

### lanes.rs

Owns grid-lanes oracle vocabulary.

Responsibilities:

- Represent a container whose public display is grid-lanes.
- Split lanes into a grid axis and a lane/masonry axis.
- Derive the lane axis from explicit auto-flow facts: row auto-flow makes rows the lane axis; column auto-flow makes columns the lane axis. The opposite axis is the grid axis.
- Place definite grid-axis items.
- Place indefinite grid-axis items using running positions.
- Track lane-axis item offsets independently from base grid occupancy.
- Report lane content size.
- Model `flow-tolerance` as an explicit enum.

This follows WebKit's design lesson: grid-lanes should be a separate placement path, not a pile of conditionals inside ordinary grid auto-placement. WebKit calls this path masonry internally; Surgeist should use public-facing names in the oracle unless production code later chooses a different internal term.

## Phase Model

### 1. Axis Mapping

Every subgrid or grid-lanes scenario starts by resolving axis relationships.

Report:

```rust
AxisMappingReport {
    queried_axis: GridAxis,
    parent_axis: GridAxis,
    child_axis: GridAxis,
    parent_writing_mode: OracleWritingMode,
    child_writing_mode: OracleWritingMode,
    parent_direction: OracleDirection,
    child_direction: OracleDirection,
    parent_flipped_in_resolved_axis: bool,
    child_flipped_in_resolved_axis: bool,
    reversed: bool,
}
```

`reversed` means the parent flipped state in the resolved parent axis differs from the child flipped state in the resolved child axis. This follows WebKit's `isSubgridReversedDirection` shape without requiring the oracle to derive flipping from production style.

The first implementation may support only horizontal-tb plus RTL reversal. Vertical writing modes should be represented in the type shape immediately so tests can be added without redesign.

### 2. Base Placement

Base placement remains the source of truth for ordinary grid and for the grid axis of grid-lanes.

Subgrid changes:

- A subgrid's own placement in the parent grid establishes its inherited track span.
- Child items inside a subgrid resolve against subgrid-local lines in the subgridded axis.
- Standalone axes use ordinary grid placement and track sizing.
- Inherited track behavior requires a resolved placement span, even when the original style used auto or span placement.
- Named-line inheritance for auto/span subgrids inside a grid-lanes parent axis is explicitly unsupported in the first oracle increment.

Grid-lanes changes:

- The grid axis may use definite line placement.
- Indefinite grid-axis placement is resolved by the lanes algorithm, not base grid auto-placement.
- The lane axis uses a synthetic translated span that represents the lane stack rather than real base-grid tracks.

### 3. Track Sizing

Base track sizing remains the source of truth for non-subgridded axes and for the grid axis of grid-lanes.

Subgrid inherited-axis track sizing:

- Copy the parent's used track sizes over the subgrid's parent span.
- Reverse the copied track order when axis mapping says the subgrid direction is reversed.
- Subtract subgrid start margin/border/padding from the first inherited tracks, consuming from the start until exhausted.
- Subtract subgrid end margin/border/padding from the last inherited tracks, consuming from the end until exhausted.
- Adjust internal tracks for gap differences between the resolved subgrid gap and resolved parent gap by half the difference on each adjacent edge.
- Track whether the subgrid gap was `normal`; `normal` resolves to the parent gap before gap-difference math.

This mirrors WebKit's `copyUsedTrackSizesForSubgrid` behavior while keeping the inputs explicit.

Report:

```rust
SubgridTrackInheritanceReport {
    parent_span: TrackSpan,
    copied_parent_tracks: Vec<TrackSize>,
    reversed: bool,
    after_reversal: Vec<TrackSize>,
    start_mbp_removed: Vec<TrackSize>,
    end_mbp_removed: Vec<TrackSize>,
    gap_difference: f32,
    parent_gap: OracleGapReport,
    subgrid_gap: OracleGapReport,
    final_tracks: Vec<TrackSize>,
}
```

Gap reports should either carry `OracleGap::Normal | OracleGap::Length(f32)` plus resolved values, or carry `resolved_parent_gap`, `resolved_subgrid_gap`, and `subgrid_gap_was_normal`. The oracle must not silently treat `normal` as zero.

Standalone-axis track sizing:

- Use ordinary `tracks` oracle phases.
- Include explicit lower bounds if the standalone axis is affected by subgrid children.

### 4. Nested Subgrid Intrinsic Contributions

For intrinsic sizing, nested subgrid descendants must contribute to the ancestor grid tracks they ultimately occupy.

WebKit's useful lesson is the stack traversal:

```text
visit child
if child is subgrid in the queried axis:
  accumulate subgrid margin/border/padding into intrinsic-min edge tracks
  recurse into its children with accumulated edge adjustments
else:
  hand leaf item and resolved ancestor span to intrinsic sizing
```

The oracle should model the traversal over explicit descriptors:

The edge placeholder rule needs explicit per-track facts. The traversal input must include intrinsic-min eligibility for the ancestor tracks touched by a subgrid edge, for example:

```rust
SubgridTraversalInput {
    ancestor_track_intrinsic_min_eligibility: Vec<bool>,
    root_children: Vec<SubgridChild>,
}
```

If a test asserts edge placeholder behavior without providing these facts, the oracle should return `OracleGridError::MissingIntrinsicMinTrackFacts`.

```rust
SubgridNode {
    id: &'static str,
    span_in_parent: TrackSpan,
    axis: SubgridAxisKind,
    margins: AxisEdges,
    border: AxisEdges,
    padding: AxisEdges,
    gap: f32,
    children: Vec<SubgridChild>,
}
```

Leaf output:

```rust
SubgridLeafContribution {
    id: &'static str,
    ancestor_span: TrackSpan,
    accumulated_edge_adjustment: Vec<f32>,
    contribution: ItemContributionFacts,
}
```

The oracle should expose traversal reports so tests can assert that a failure is in span translation, edge adjustment, or contribution sizing.

### 5. Grid-Lanes Placement

Grid-lanes uses a separate report, inspired by WebKit's `GridMasonryLayout`.

Inputs:

- grid axis track count
- auto-flow direction used to derive the lane axis
- lane axis direction
- gap in the lane axis
- item order
- each item's grid-axis span, if definite
- each item's grid-axis auto span, if indefinite
- item lane-axis margin-box size as an explicit number
- flow tolerance: normal, fixed length, percentage, or infinite

Algorithm:

- Initialize one running position per grid-axis track.
- For a definite grid-axis item, use its resolved grid-axis span.
- For an indefinite grid-axis item:
  - If tolerance is infinite, place round-robin from the cursor and wrap when the item does not fit.
  - Otherwise, clamp the requested span length to the grid-axis track count.
  - Set `max_start = grid_axis_track_count + 1 - span_len`.
  - Shift the cursor to `0` when the cursor is greater than `max_start`.
  - Search candidate starts modulo `max_start`, not modulo the track count.
  - Choose the first candidate whose max running position is within tolerance of the absolute shortest candidate.
  - No finite-tolerance candidate may wrap across the grid-axis end.
- The item lane-axis offset is the previous max running position for its chosen span.
- After placement, update every covered running position to `previous + item_margin_box + lane_gap`.
- Content size is the max running position minus the final gap.
- Cursor advances to the end of the grid-axis span modulo grid-axis track count.

Report:

```rust
LanePlacementReport {
    item_offsets: Vec<LaneItemOffset>,
    running_positions_after_each_item: Vec<Vec<f32>>,
    content_size: f32,
    final_cursor: usize,
}
```

### 6. Grid-Lanes Intrinsic Track Sizing

Grid-lanes affects track sizing in the grid axis, not the lane axis.

WebKit's useful distinction:

- Definite grid-axis items contribute to their actual spans.
- Indefinite grid-axis items contribute as grouped possibilities because they may land in multiple tracks.
- Items inside subgrids of a grid-lanes container need special handling when their placement is indefinite.

First oracle increment:

- Support definite grid-axis items normally.
- Support direct indefinite grid-axis items by aggregating a maximum contribution by span length.
- Convert aggregated indefinite contributions into definite possibilities across content-sized tracks.
- Return unsupported for nested grid-lanes subgrid indefinite descendants until a fixture requires it.

Report:

```rust
LaneIntrinsicSizingReport {
    definite_items: Vec<GridItemWithSpan>,
    indefinite_groups: Vec<IndefiniteLaneContributionGroup>,
    converted_indefinite_items: Vec<GridItemWithSpan>,
    final_track_report: TrackSizingReport,
}
```

### 7. Alignment And Final Rectangles

Alignment remains phase-local:

- Track alignment after sizing stays in `alignment.rs`.
- Item self-alignment inside subgrid areas remains child layout/scenario vocabulary, with explicit item sizes and margins.
- Baseline facts are explicit inputs. The oracle may compose baseline offsets but may not infer text baselines.
- Only `scenario.rs` may compose placement, track sizing, lane placement, alignment, and final rectangles.
- Scenario helpers must consume explicit solved tracks, explicit item sizes, explicit margins, and explicit placement reports. They must not recursively solve child layout or call measurement hooks.

Subgrid item rectangles must report:

- inherited axis offsets from parent tracks
- standalone axis offsets from local tracks
- margin/border/padding offsets for the subgrid container
- item self-alignment offsets
- final rectangle

Grid-lanes item rectangles must report:

- grid-axis area from solved tracks
- lane-axis offset from `LanePlacementReport`
- item alignment inside the grid-axis area
- final rectangle

## Suggested Module Interfaces

The exact names can change during implementation, but the first implementation should keep these concepts visible:

```rust
pub enum GridAxis {
    Column,
    Row,
}

pub enum OracleWritingMode {
    HorizontalTb,
    VerticalLr,
    VerticalRl,
}

pub enum OracleDirection {
    Ltr,
    Rtl,
}

pub enum OracleGap {
    Normal,
    Length(f32),
}

pub struct OracleGapReport {
    pub specified: OracleGap,
    pub resolved: f32,
}

pub struct TrackSpan {
    pub start: usize,
    pub end: usize,
}

pub struct AxisEdges {
    pub start: f32,
    pub end: f32,
}

pub enum SubgridAxisKind {
    Inherited,
    Standalone,
}

pub enum LaneFlowTolerance {
    Normal { font_size: f32 },
    Fixed(f32),
    Percent(f32),
    Infinite,
}
```

No oracle type should contain production `NodeInput`, production `Tree`, or a closure that measures children.

## Coverage Strategy

Coverage should grow in this order.

### Subgrid Axis And Eligibility

- requested subgrid with parent grid is eligible
- requested subgrid with no parent grid is not eligible
- requested subgrid with independent formatting context is not eligible
- requested subgrid in parent grid-lanes axis is not eligible
- requested subgrid in the standalone axis of a grid-lanes parent remains eligible when the resolved parent axis is not lanes
- RTL parent versus LTR child produces a reversed inherited axis
- reversed inherited axis is computed from resolved parent and child flipped states, not from parent direction alone
- orthogonal writing modes map column to row and row to column once vertical mode support is enabled

### Subgrid Track Inheritance

- copy parent used track sizes for a direct column subgrid
- copy parent used track sizes for a direct row subgrid
- reverse copied tracks when direction is reversed
- remove start margin/border/padding from inherited start tracks
- remove end margin/border/padding from inherited end tracks
- subtract positive subgrid gap difference at internal edges
- add negative subgrid gap difference at internal edges
- resolve `normal` subgrid gap to the parent gap before difference math
- preserve zero-size tracks without producing negative sizes

### Nested Subgrid Intrinsic Contributions

- direct leaf contributes to ancestor span
- single nested subgrid translates leaf span to ancestor span
- nested subgrid edge margin/border/padding raises edge track lower bounds
- edge margin/border/padding only applies to edge tracks marked intrinsic-min eligible
- nested subgrid gap difference adds ancestor extra margin
- multiple nested subgrids accumulate edge adjustments
- missing intrinsic-min track facts return an explicit unsupported error
- unsupported line-name inheritance returns an explicit unsupported result

### Grid-Lanes Placement

- definite grid-axis item uses the declared span
- indefinite item chooses the shortest available span
- infinite tolerance uses round-robin placement
- fixed tolerance chooses first span within tolerance from cursor
- percentage tolerance resolves against explicit basis
- spanning item updates every covered running position
- finite-tolerance search uses `max_start = grid_axis_track_count + 1 - span_len` and does not wrap candidates across the grid-axis end
- content size excludes trailing lane gap
- cursor wraps by grid-axis track count

### Grid-Lanes With Subgrid

- subgrid request in lane axis is not eligible
- subgrid request in grid axis can inherit grid-axis tracks
- grid-lanes item inside ordinary subgrid uses the nearest grid-lanes ancestor only when explicitly described
- auto-placed subgrid in grid-lanes returns unsupported for line-name inheritance in the first increment

### Scenario Fixtures

Keep scenario fixtures small and high signal:

- ordinary grid unchanged by the new modules
- one direct subgrid with inherited columns and standalone rows
- one nested subgrid with margin/border/padding edge adjustments
- one grid-lanes container with three items and fixed tolerance
- one grid-lanes plus subgrid eligibility scenario

Browser parity fixtures under these paths should inform scenario selection, but the oracle should not duplicate the entire fixture matrix:

- `crates/surgeist/tests/layout_browser_parity/html/subgrid/`
- `crates/surgeist/tests/layout_browser_parity/html/grid-lanes/`
- `crates/surgeist/tests/layout_browser_parity/xml/grid-lanes/`

Grid-lanes fixtures should inform lane placement, intrinsic sizing, empty containers, overflow and scrollbar behavior, RTL, and min/max-content scenario selection.

## Error Policy

Unsupported cases must be explicit and typed:

```rust
OracleGridError::NamedLineInheritanceUnsupported
OracleGridError::BaselineInferenceUnsupported
OracleGridError::MissingIntrinsicMinTrackFacts
OracleGridError::NestedGridLanesSubgridIndefiniteUnsupported
OracleGridError::VerticalWritingModeUnsupported
```

Returning an unsupported error is preferable to silently producing approximate numbers. Tests for unsupported cases are part of correctness.

## Implementation Notes For Future Plans

- Start with `axis.rs` and subgrid eligibility before track inheritance.
- Add report structs before broad scenario composition.
- Keep WebKit references in tests as comments only when they clarify a rule; do not encode WebKit file paths into APIs.
- Commit at logical checkpoints:
  - axis vocabulary and eligibility
  - inherited track copying
  - nested intrinsic traversal
  - lanes placement
  - lanes intrinsic sizing
  - composed scenarios
- Run focused oracle tests after each checkpoint, then `cargo test -p surgeist --test layout_oracle`, then `cargo test -p surgeist`.

## Done Criteria

The spec is complete when:

- It describes how grid, subgrid, and grid-lanes share phase vocabulary.
- It names the oracle modules and responsibilities.
- It defines the WebKit-inspired rules Surgeist should adopt.
- It draws a firm line around non-goals and unsupported cases.
- A clean-context reviewer checks the spec for correctness, missing interactions, and hidden second-engine risk.
- Any accepted reviewer recommendations are incorporated.
