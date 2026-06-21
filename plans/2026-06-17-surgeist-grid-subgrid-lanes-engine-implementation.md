# Surgeist Grid, Subgrid, And Grid-Lanes Engine Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement production Surgeist layout-engine support for grid-lanes and subgrid behavior, using the completed grid/subgrid/grid-lanes oracle as the executable reference while keeping oracle code test-only.

**Architecture:** Extend the existing `crates/surgeist/src/layout/grid` pipeline in phases: value plumbing, engine phase extraction, subgrid axis/track inheritance, nested subgrid intrinsic contribution traversal, grid-lanes placement/sizing, then composed layout output. Production code may mirror oracle vocabulary, but must live in `src/layout/grid` and must not import `crates/surgeist/tests/support/oracle`. Layout-oracle tests compare production results against oracle reports at the boundary.

**Tech Stack:** Rust under `crates/surgeist`, focused layout tests in `crates/surgeist/tests/layout_oracle.rs` and `crates/surgeist/tests/layout/grid.rs`, pure oracle tests in `crates/surgeist/tests/oracle.rs`, browser parity fixtures under `crates/surgeist/tests/layout_browser_parity`, verification with `cargo test -p surgeist --test oracle`, `cargo test -p surgeist --test layout_oracle`, `cargo test -p surgeist`, and `cargo clippy -p surgeist --all-targets --all-features -- -D warnings`.

---

## Source References

- Oracle spec: `docs/superpowers/specs/2026-06-16-surgeist-subgrid-grid-lanes-oracle-design.md`
- Oracle implementation plan: `docs/superpowers/plans/2026-06-16-surgeist-subgrid-grid-lanes-oracle-implementation.md`
- Production grid engine:
  - `crates/surgeist/src/layout/grid/mod.rs`
  - `crates/surgeist/src/layout/grid/tracks.rs`
  - `crates/surgeist/src/layout/grid/placement.rs`
  - `crates/surgeist/src/layout/grid/child.rs`
  - `crates/surgeist/src/layout/grid/alignment.rs`
- Production layout values:
  - `crates/surgeist/src/layout/value.rs`
  - `crates/surgeist/src/layout/node_input.rs`
  - `crates/surgeist/src/style/value.rs`
  - `crates/surgeist/src/style/adapters/layout.rs`
- Oracle modules:
  - `crates/surgeist/tests/support/oracle/grid/axis.rs`
  - `crates/surgeist/tests/support/oracle/grid/subgrid.rs`
  - `crates/surgeist/tests/support/oracle/grid/lanes.rs`
  - `crates/surgeist/tests/support/oracle/grid/tracks.rs`
  - `crates/surgeist/tests/support/oracle/grid/placement.rs`
  - `crates/surgeist/tests/support/oracle/grid/scenario.rs`
- Production/oracle comparison harness:
  - `crates/surgeist/tests/support/grid_layout_comparison.rs`
  - `crates/surgeist/tests/support/oracle_tree.rs`
  - `crates/surgeist/tests/layout_oracle.rs`

---

## Required Boundaries

- [ ] Do not import test oracle modules from production code.
- [ ] Keep `compute_grid` as the public production entry point.
- [ ] Add production phase helpers under `crates/surgeist/src/layout/grid`, not in broader layout modules unless they become shared outside grid.
- [ ] Preserve existing ordinary-grid behavior while introducing subgrid and grid-lanes.
- [ ] Add unsupported states as explicit production fallbacks or no-op behavior before adding approximations.
- [ ] Keep line-name inheritance, full vertical writing-mode subgrid mapping, nested grid-lanes indefinite subgrid sizing, and baseline inference outside the first implementation unless a task below explicitly says otherwise.
- [ ] Commit after each logical phase with short concrete messages.
- [ ] Run `git status --short --branch` before staging each commit.
- [ ] Use narrow `git add` path lists. Do not stage whole directories unless every file in that directory belongs to the current task.
- [ ] Run `git diff --check` before each commit.

---

## Implementation Overview

The current production grid engine is largely monolithic in `compute_grid`, with reusable phase helpers in `tracks.rs`, `placement.rs`, and `child.rs`. The implementation should avoid a large rewrite. First introduce small production data types and phase wrappers that make the current algorithm inspectable. Then add subgrid and grid-lanes in places that match the oracle:

1. Value plumbing: represent `Display::GridLanes`, `TrackComponent::Subgrid`, and flow tolerance at the layout layer.
2. Baseline-preserving refactor: extract grid setup and phase reports without changing behavior.
3. Test harness expansion: teach layout-oracle comparison helpers to express nested grids, subgrids, grid-lanes roots, margins, padding, and lane reports.
4. Subgrid axis and eligibility: determine whether each child is a subgrid in column and/or row axes.
5. Subgrid track inheritance: copy used parent tracks into child subgrid axes, including reversal, margin/border/padding consumption, and gap differences.
6. Subgrid intrinsic traversal: make parent intrinsic sizing see eligible nested subgrid leaf contributions.
7. Grid-lanes placement: add a separate placement path for lane containers.
8. Grid-lanes sizing and child layout: convert lane reports into production child rectangles and content size.
9. Browser parity and cleanup: add fixtures for cases the oracle now covers; keep unsupported features documented in tests.

---

## Task 1: Confirm Baseline And Add Failing Value Plumbing Tests

**Files:**
- Modify `crates/surgeist/src/layout/node_input.rs`
- Modify `crates/surgeist/src/layout/value.rs`
- Modify `crates/surgeist/src/layout/compute.rs` or the display dispatcher that calls `compute_grid`
- Modify `crates/surgeist/src/style/value.rs`
- Modify `crates/surgeist/src/style/property.rs`
- Modify `crates/surgeist/src/style/declaration.rs`
- Modify `crates/surgeist/src/style/adapters/layout.rs`
- Modify `crates/surgeist/src/css/mod.rs`
- Modify `crates/surgeist/tests/support/oracle_tree.rs`
- Modify `crates/surgeist/tests/style.rs` or add focused tests where current adapter tests live
- Modify `crates/surgeist/tests/css.rs`
- Modify `crates/surgeist/tests/layout/grid.rs`

- [ ] Run the current focused baseline:

```bash
cargo test -p surgeist --test oracle
cargo test -p surgeist --test layout_oracle
cargo test -p surgeist --test layout -- grid
```

Expected: all pass before engine work begins.

- [ ] Add failing tests proving style can lower `Display::GridLanes` and `Display::InlineGridLanes` to layout-layer display values.

Expected layout API shape:

```rust
pub enum Display {
    Block,
    Flex,
    Grid,
    GridLanes,
    None,
}
```

Inline layout display should lower to the same layout mode as the block-level equivalent, matching the current layout engine's block/inline simplification.

- [ ] Update every layout display dispatcher for the new variant before committing Task 1. Until Task 8 adds lane-specific branching, `Display::GridLanes` must dispatch to `compute_grid` with ordinary-grid behavior. This temporary fallback prevents exhaustive match failures and gives Task 3 eligibility tests a runnable parent display mode.

Required dispatcher updates include:
  - `crates/surgeist/tests/support/oracle_tree.rs`
  - any production dispatcher that matches `layout::Display`
  - any layout smoke harness with exhaustive display matching

- [ ] Add failing tests proving style `GridTrackComponent::Subgrid(SubgridTrack)` is preserved in layout `TrackComponent`.

Expected layout API shape:

```rust
pub enum TrackComponent {
    Track(TrackSizing),
    Repeat(TrackRepetition),
    Subgrid(SubgridTrack),
}

pub struct SubgridTrack {
    pub line_names: Vec<Vec<String>>,
}
```

- [ ] Refactor track-list expansion so `Subgrid` does not fall through generic track expansion. Add a layout-internal axis template shape:

```rust
enum GridAxisTemplate {
    Tracks(Vec<TrackSizing>),
    Subgrid(SubgridTrack),
}
```

Task 1 only needs the value to survive lowering and dispatch safely. Before Task 5 and Task 6 consume inherited tracks, an ineligible or unsupported `Subgrid` axis must resolve to an explicit empty explicit-track list that can still grow implicit auto tracks. Do not silently convert `Subgrid` to `auto`, and do not make `reserved_track_space`, `tracks_need_available_basis`, or auto-repeat helpers treat it as an ordinary track component.

- [ ] Add a real style and layout value for grid flow tolerance in this task:

```rust
pub enum GridFlowTolerance {
    Normal,
    Length(Length),
    Percent(f32),
    Infinite,
}
```

This is a style-exposed property, not a test-only layout knob. Add:
  - `style::Value::GridFlowTolerance`
  - `style::Property::GridFlowTolerance`
  - `Declarations::grid_flow_tolerance`
  - metadata/default/validation in `style/property.rs`
  - lowering into `layout::NodeInput`
  - CSS parsing for `grid-flow-tolerance`

`Normal` must lower with the resolved `font-size` fact so production can match the oracle's `LaneFlowTolerance::Normal { font_size }`. Add an explicit layout value such as `layout::GridFlowTolerance::Normal { font_size: Scalar }` if that is the cleanest route.

- [ ] Implement the layout value plumbing and adapter lowering.

- [ ] Ensure ordinary `Display::Grid` and non-subgrid track lists still lower exactly as before.

- [ ] Run:

```bash
cargo test -p surgeist --test style
cargo test -p surgeist --test css
cargo test -p surgeist --test layout -- grid
cargo fmt --check
```

- [ ] Commit:

```bash
git status --short --branch
git diff --check
git add crates/surgeist/src/layout/node_input.rs crates/surgeist/src/layout/value.rs crates/surgeist/src/style/value.rs crates/surgeist/src/style/property.rs crates/surgeist/src/style/declaration.rs crates/surgeist/src/style/adapters/layout.rs crates/surgeist/src/css/mod.rs crates/surgeist/tests/support/oracle_tree.rs crates/surgeist/tests/style.rs crates/surgeist/tests/css.rs crates/surgeist/tests/layout/grid.rs
git commit -m "Plumb grid-lanes and subgrid layout values"
```

---

## Task 2: Extract Production Grid Phase Inputs Without Behavior Change

**Files:**
- Modify `crates/surgeist/src/layout/grid/mod.rs`
- Modify `crates/surgeist/src/layout/grid/tracks.rs`
- Modify `crates/surgeist/src/layout/grid/placement.rs`
- Modify `crates/surgeist/src/layout/grid/child.rs`
- Modify `crates/surgeist/src/layout/grid/tests.rs`

- [ ] Add a production-only phase context type near `compute_grid`:

```rust
struct GridContainerContext {
    gap: Size,
    column_basis: Option<Scalar>,
    row_basis: Option<Scalar>,
    explicit_columns: usize,
    explicit_rows: usize,
    leading_columns: usize,
    leading_rows: usize,
    lines: GridLines,
}
```

This type may evolve during implementation, but it should initially be a pure extraction of values `compute_grid` already computes.

- [ ] Extract track-list setup from `compute_grid` into a helper with no behavior change:

```rust
fn initialize_grid_tracks<Tree>(
    tree: &Tree,
    node: <Tree as Traverse>::Node,
    style: &NodeInput,
    constants: &Constants,
) -> InitializedGridTracks
where
    Tree: Compute,
```

The helper should compute expanded columns/rows, leading implicit tracks, explicit counts, and `GridLines`.

- [ ] Extract intrinsic sizing orchestration into a helper without changing the existing algorithm:

```rust
fn resolve_grid_track_sizes<Tree>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: GridTrackResolutionInput<'_>,
) -> GridTrackResolution
where
    Tree: Compute,
```

This is a mechanical refactor. Do not add subgrid or lanes behavior in this task.

- [ ] Extract child layout orchestration only enough to make it reusable by grid-lanes later. Keep `layout_grid_children` in `child.rs`.

- [ ] Add unit tests for any newly public or `pub(super)` helper only when a test can assert behavior more directly than the existing integration tests.

- [ ] Run:

```bash
cargo test -p surgeist --test layout_oracle
cargo test -p surgeist --test layout -- grid
cargo test -p surgeist
```

Expected: no behavior change.

- [ ] Commit:

```bash
git status --short --branch
git diff --check
git add crates/surgeist/src/layout/grid/mod.rs crates/surgeist/src/layout/grid/tracks.rs crates/surgeist/src/layout/grid/placement.rs crates/surgeist/src/layout/grid/child.rs crates/surgeist/src/layout/grid/tests.rs
git commit -m "Extract grid layout phases"
```

---

## Task 3: Extend Layout-Oracle Comparison Harness

**Files:**
- Modify `crates/surgeist/tests/support/grid_layout_comparison.rs`
- Modify `crates/surgeist/tests/support/oracle_tree.rs`
- Modify `crates/surgeist/tests/layout_oracle.rs`

- [ ] Extend `GridLayoutComparison` so later tasks can build more than a flat grid root.

Required capabilities:
  - root display can be `Display::Grid` or `Display::GridLanes`
  - children can themselves have `Display::Grid` or `Display::GridLanes`
  - child track templates can include `TrackComponent::Subgrid`
  - children can specify margin, padding, border, direction, writing mode, overflow, and position
  - expected layout assertions can target nested descendants, not only direct children
  - lane tests can compare lane reports and final child rectangles without duplicating oracle calculations by hand

- [ ] Keep the existing simple builder API working for current grid layout-oracle tests.

- [ ] Add one harness-only regression test that builds a nested ordinary grid with margins and asserts the expected descendant output. This proves the harness extension works before subgrid behavior is added.

- [ ] Add one harness-only regression test that builds a `Display::GridLanes` root and confirms the temporary Task 1 fallback dispatches through `compute_grid` until lane-specific branching is added.

- [ ] Run:

```bash
cargo test -p surgeist --test layout_oracle
cargo test -p surgeist --test layout -- grid
```

- [ ] Commit:

```bash
git status --short --branch
git diff --check
git add crates/surgeist/tests/support/grid_layout_comparison.rs crates/surgeist/tests/support/oracle_tree.rs crates/surgeist/tests/layout_oracle.rs
git commit -m "Extend grid layout oracle harness"
```

---

## Task 4: Add Production Axis Mapping And Subgrid Eligibility

**Files:**
- Add `crates/surgeist/src/layout/grid/axis.rs`
- Add `crates/surgeist/src/layout/grid/subgrid.rs`
- Modify `crates/surgeist/src/layout/grid/mod.rs`
- Modify `crates/surgeist/src/layout/grid/tracks.rs`
- Modify `crates/surgeist/tests/support/grid_layout_comparison.rs`
- Modify `crates/surgeist/tests/layout/grid.rs`
- Modify `crates/surgeist/tests/layout_oracle.rs`

- [ ] Add production axis mapping helpers inspired by oracle `axis.rs`.

Use production `WritingMode`, `Direction`, and grid axes:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum GridAxisKind {
    Column,
    Row,
}

struct GridAxisMapping {
    parent_axis: GridAxisKind,
    child_axis: GridAxisKind,
    reversed: bool,
}
```

If `GridAxisKind` already exists in `placement.rs`, move it to `axis.rs` or re-export it internally so placement and subgrid share one type.

- [ ] Match the oracle's first-increment support:
  - horizontal-tb parent and child are supported
  - RTL reversal is represented with `reversed`
  - vertical writing modes return an explicit unsupported mapping result

- [ ] Add production subgrid eligibility helpers:

```rust
struct SubgridEligibility {
    eligible: bool,
    reason: Option<SubgridIneligibleReason>,
}
```

The predicate must match the oracle:

1. track declaration requests subgrid
2. child has a parent grid
3. child display is a grid container display supported by this engine phase
4. child is not an independent formatting context
5. child is not excluded from normal layout
6. parent is not grid-lanes in the resolved parent axis

For this first implementation, a child can be subgrid-eligible only when its production `display` is `Display::Grid`, or `Display::GridLanes` in an axis explicitly supported by the current grid-lanes fallback/implementation. A block or flex child that happens to carry subgrid track values must remain an ordinary block/flex child.

- [ ] Treat `position: absolute`, `display: none`, and independent formatting behavior as ineligible, not as malformed input.

- [ ] Add failing layout tests for:
  - ordinary child with subgrid track declaration is ignored when there is no parent grid
  - block child with subgrid track declaration is not routed through the grid engine
  - flex child with subgrid track declaration is not routed through the grid engine
  - absolute child with subgrid declaration does not participate as subgrid
  - child of a grid-lanes container is not subgrid in the lane axis

- [ ] Implement eligibility and wire it into grid setup as a report used by later tasks. It should not change layout output yet except for tests that inspect direct helper behavior.

- [ ] Run:

```bash
cargo test -p surgeist --test oracle oracle_subgrid_eligibility
cargo test -p surgeist --test layout -- grid
cargo test -p surgeist --test layout_oracle
```

- [ ] Commit:

```bash
git status --short --branch
git diff --check
git add crates/surgeist/src/layout/grid/axis.rs crates/surgeist/src/layout/grid/subgrid.rs crates/surgeist/src/layout/grid/mod.rs crates/surgeist/src/layout/grid/tracks.rs crates/surgeist/tests/support/grid_layout_comparison.rs crates/surgeist/tests/layout/grid.rs crates/surgeist/tests/layout_oracle.rs
git commit -m "Add grid subgrid eligibility"
```

---

## Task 5: Implement Subgrid Track Inheritance For Used Tracks

**Files:**
- Modify `crates/surgeist/src/layout/grid/subgrid.rs`
- Modify `crates/surgeist/src/layout/grid/tracks.rs`
- Modify `crates/surgeist/src/layout/grid/child.rs`
- Modify `crates/surgeist/src/layout/grid/tests.rs`

- [ ] Add a production equivalent of oracle `inherit_subgrid_tracks`.

Expected production input:

```rust
struct SubgridTrackInheritanceInput<'a> {
    parent_tracks: &'a [Scalar],
    parent_span: GridTrackSpan,
    reversed: bool,
    start_mbp: Scalar,
    end_mbp: Scalar,
    parent_gap: Scalar,
    subgrid_gap: ResolvedSubgridGap,
}
```

- [ ] Define `ResolvedSubgridGap` so `normal` can be represented before resolving:

```rust
enum ResolvedSubgridGap {
    Normal,
    Length(Scalar),
}
```

`Normal` resolves to the parent gap before gap-difference math.

- [ ] Copy parent used track sizes over the placed parent span.

- [ ] Reverse copied tracks when the axis mapping says `reversed`.

- [ ] Subtract subgrid margin/border/padding from start and end inherited tracks using the oracle's consuming rule.

- [ ] Apply internal gap-difference adjustment with half the difference on each adjacent track edge.

- [ ] Add direct production helper tests in `crates/surgeist/src/layout/grid/tests.rs` for:
  - simple inherited columns
  - reversed inherited columns
  - start/end margin-border-padding consumption
  - subgrid gap `normal`
  - explicit subgrid gap differing from parent gap

These tests should assert production helper reports directly. Do not require full production child layout in this task; full child-rect integration starts in Task 6.

- [ ] Keep standalone axis behavior ordinary. A grid item can be subgridded in columns while its rows are independently sized, and vice versa.

- [ ] Run:

```bash
cargo test -p surgeist --test oracle oracle_subgrid_track
cargo test -p surgeist layout::grid
cargo test -p surgeist --test layout -- grid
```

- [ ] Commit:

```bash
git status --short --branch
git diff --check
git add crates/surgeist/src/layout/grid/subgrid.rs crates/surgeist/src/layout/grid/tracks.rs crates/surgeist/src/layout/grid/child.rs crates/surgeist/src/layout/grid/tests.rs
git commit -m "Implement subgrid track inheritance"
```

---

## Task 6: Thread Subgrid Child Layout Through Parent Used Tracks

**Files:**
- Modify `crates/surgeist/src/layout/grid/mod.rs`
- Modify `crates/surgeist/src/layout/grid/child.rs`
- Modify `crates/surgeist/src/layout/grid/placement.rs`
- Modify `crates/surgeist/src/layout/grid/tracks.rs`
- Modify `crates/surgeist/tests/support/grid_layout_comparison.rs`
- Modify `crates/surgeist/tests/layout_oracle.rs`

- [ ] When laying out a child grid item that is eligible as subgrid in one or both axes, pass inherited track sizes into the child's grid layout instead of letting the child expand its own template in that axis.

- [ ] Use a grid-local recursive context route for this first implementation. Do not extend `ComputeInput`, do not add transient inherited tracks to `NodeInput`, and do not alter cache keys in this task.

- [ ] Split `compute_grid` into the existing public entry point plus an internal context-aware helper:

```rust
pub fn compute_grid<Tree>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: ComputeInput,
) -> ComputeOutput
where
    Tree: Compute,
{
    compute_grid_with_context(tree, node, input, GridParentContext::none())
}

struct GridParentContext {
    columns: Option<InheritedGridAxis>,
    rows: Option<InheritedGridAxis>,
}
```

When a grid child is eligible as a subgrid, call `compute_grid_with_context` directly for that child. Non-subgrid children continue through `tree.compute_child`.

- [ ] Document in code that this intentionally bypasses generic layout caching for context-sensitive subgrid calls. A later cache design can add context keys if profiling proves it matters.

- [ ] Ensure subgrid child placement resolves against subgrid-local line indexes while final child rectangles are expressed in parent content coordinates.

- [ ] Add tests for:
  - full production child rects match `compose_subgrid_item_rect`
  - subgrid child items resolve against local lines
  - subgrid itself still respects parent grid placement
  - child alignment and auto margins still use inherited area size
  - absolute children in the subgrid keep existing production static-position behavior

The absolute-child test is a production smoke test, not an oracle-parity requirement; the oracle does not define absolute descendants inside subgrid.

- [ ] Run:

```bash
cargo test -p surgeist --test layout_oracle subgrid
cargo test -p surgeist --test layout -- grid
cargo test -p surgeist
```

- [ ] Commit:

```bash
git status --short --branch
git diff --check
git add crates/surgeist/src/layout/grid/mod.rs crates/surgeist/src/layout/grid/child.rs crates/surgeist/src/layout/grid/placement.rs crates/surgeist/src/layout/grid/tracks.rs crates/surgeist/tests/support/grid_layout_comparison.rs crates/surgeist/tests/layout_oracle.rs
git commit -m "Lay out children on inherited subgrid tracks"
```

---

## Task 7: Implement Nested Subgrid Intrinsic Contribution Traversal

**Files:**
- Modify `crates/surgeist/src/layout/grid/subgrid.rs`
- Modify `crates/surgeist/src/layout/grid/tracks.rs`
- Modify `crates/surgeist/src/layout/grid/mod.rs`
- Modify `crates/surgeist/src/layout/grid/tests.rs`
- Modify `crates/surgeist/tests/layout_oracle.rs`

- [ ] Add a production traversal corresponding to oracle `collect_subgrid_leaf_contributions`.

The traversal should:
  - walk explicit eligible subgrid descendants in the queried inherited axis
  - translate descendant local spans into ancestor track spans
  - respect reversed nested subgrid traversal
  - add subgrid margin/border/padding contributions
  - carry per-leaf `accumulated_edge_adjustment`
  - carry per-leaf `accumulated_gap_adjustment`
  - compute `edge_lower_bounds` for parent tracks touched by subgrid edges
  - reject or skip standalone-axis traversal using an explicit unsupported path

- [ ] Model intrinsic-min track eligibility facts explicitly. When production cannot know whether an edge track is intrinsic-min eligible, return an explicit helper-report error equivalent to oracle `MissingIntrinsicMinTrackFacts` instead of guessing.

This diagnostic is a production helper/report result for tests and internal control flow, not part of `ComputeOutput`. Do not try to encode unsupported traversal states into layout output.

- [ ] When integrating traversal into production sizing, preserve these oracle report concepts even if the production struct names differ:
  - translated leaf span
  - reversed traversal state at each subgrid level
  - accumulated margin/border/padding edge adjustment
  - accumulated gap adjustment
  - edge lower bounds for non-intrinsic edge tracks
  - missing intrinsic-min facts

- [ ] Integrate traversal into `intrinsic_track_sizes` before distributing spanning contributions.

- [ ] Preserve existing direct child measurement for ordinary grid items.

- [ ] Add direct production helper tests in `crates/surgeist/src/layout/grid/tests.rs` for:
  - edge lower bounds on non-intrinsic edge tracks
  - missing intrinsic-min facts returning the explicit helper-report error
  - accumulated edge adjustment in a nested translated span
  - accumulated gap adjustment through nested subgrids

- [ ] Add layout-oracle tests for:
  - nested inherited subgrid leaf contribution grows parent auto track
  - reversed nested inherited subgrid maps contribution to the mirrored track
  - nested margin/border/padding increases the contribution
  - gap-difference adjustment accumulates through nested subgrids
  - translated nested edge adjustments land on the correct ancestor tracks
  - standalone subgrid traversal remains unsupported in this phase

- [ ] Keep baseline inference unsupported for subgrid intrinsic traversal. Baseline behavior for ordinary grid remains unchanged.

- [ ] Run:

```bash
cargo test -p surgeist --test oracle subgrid_traversal
cargo test -p surgeist --test layout_oracle subgrid
cargo test -p surgeist --test layout -- grid
```

- [ ] Commit:

```bash
git status --short --branch
git diff --check
git add crates/surgeist/src/layout/grid/subgrid.rs crates/surgeist/src/layout/grid/tracks.rs crates/surgeist/src/layout/grid/mod.rs crates/surgeist/tests/support/grid_layout_comparison.rs crates/surgeist/tests/layout_oracle.rs
git commit -m "Add nested subgrid intrinsic traversal"
```

---

## Task 8: Add Grid-Lanes Placement As A Separate Path

**Files:**
- Add `crates/surgeist/src/layout/grid/lanes.rs`
- Modify `crates/surgeist/src/layout/grid/mod.rs`
- Modify `crates/surgeist/src/layout/grid/placement.rs`
- Modify `crates/surgeist/tests/layout_oracle.rs`
- Modify `crates/surgeist/tests/layout/grid.rs`

- [ ] Add production lane-axis helpers matching oracle `lanes.rs`:

```rust
fn lane_axis(auto_flow: GridAutoFlow) -> GridAxisKind;
fn grid_axis_for_lanes(auto_flow: GridAutoFlow) -> GridAxisKind;
```

Row auto-flow means rows are the lane axis. Column auto-flow means columns are the lane axis.

- [ ] Implement a production `place_lanes` equivalent that accepts production child facts rather than oracle facts:

```rust
struct LanePlacementInput<Item> {
    grid_axis_tracks: usize,
    auto_flow: GridAutoFlow,
    lane_gap: Scalar,
    tolerance: GridFlowTolerance,
    tolerance_basis: Scalar,
    items: Vec<LaneItem<Item>>,
}
```

- [ ] Preserve the oracle rules:
  - definite grid-axis start is honored
  - auto grid-axis items use running lane positions
  - auto grid-axis span is clamped to `1..=grid_axis_tracks`
  - finite tolerance picks the first candidate within shortest-position plus tolerance
  - finite candidate search computes `max_start = grid_axis_tracks + 1 - span`
  - finite candidate search resets the cursor to zero when it is outside `max_start`
  - finite candidate search wraps only modulo `max_start`
  - finite candidate search never allows a candidate span to wrap across the grid-axis end
  - infinite tolerance uses cursor order
  - content size excludes the trailing lane gap

- [ ] Do not modify ordinary `resolve_grid_child_areas` to handle lane placement. Grid-lanes must call a separate placement path from `compute_grid`.

- [ ] Add layout-oracle tests comparing production to oracle `place_lanes` for:
  - row auto-flow
  - column auto-flow
  - definite grid-axis item
  - auto span clamping
  - finite tolerance
  - finite search does not wrap a candidate span across the grid-axis end
  - infinite tolerance

- [ ] Run:

```bash
cargo test -p surgeist --test oracle lanes
cargo test -p surgeist --test layout_oracle lanes
cargo test -p surgeist --test layout -- grid
```

- [ ] Commit:

```bash
git status --short --branch
git diff --check
git add crates/surgeist/src/layout/grid/lanes.rs crates/surgeist/src/layout/grid/mod.rs crates/surgeist/src/layout/grid/placement.rs crates/surgeist/tests/support/grid_layout_comparison.rs crates/surgeist/tests/layout_oracle.rs crates/surgeist/tests/layout/grid.rs
git commit -m "Add grid-lanes placement"
```

---

## Task 9: Implement Grid-Lanes Track Sizing And Child Rects

**Files:**
- Modify `crates/surgeist/src/layout/grid/lanes.rs`
- Modify `crates/surgeist/src/layout/grid/tracks.rs`
- Modify `crates/surgeist/src/layout/grid/child.rs`
- Modify `crates/surgeist/src/layout/grid/mod.rs`
- Modify `crates/surgeist/tests/support/grid_layout_comparison.rs`
- Modify `crates/surgeist/tests/layout_oracle.rs`

- [ ] Use ordinary grid track sizing for the grid axis of a grid-lanes container.

- [ ] Implement lane intrinsic sizing with the same observable stages as oracle `resolve_lane_intrinsic_sizing`:
  - identify content-sized grid-axis tracks
  - include definite items only when their definite span overlaps content-sized tracks
  - skip definite items outside content-sized tracks
  - clamp oversized indefinite spans to the grid-axis track count
  - group indefinite items by clamped span length
  - keep max min-content, max-content, and min-size facts per span group
  - convert each span group across every valid candidate start
  - project contributions into disjoint content-sized spans
  - preserve min-content vs max-content track behavior

- [ ] Size the lane axis from placed item margin boxes and lane running positions.

- [ ] Convert lane placement reports into `GridArea`-like child rectangles for `layout_grid_children`, or add a lane-specific child layout helper if the synthetic lane axis would make `GridArea` misleading.

- [ ] Ensure child measurement uses:
  - grid-axis span size from resolved grid-axis tracks
  - lane-axis available size from the item's own measured margin box, not from a normal row/column track

- [ ] Add intrinsic sizing tests for:
  - lane content size contributes to container size
  - content-sized grid-axis tracks collect lane item contributions
  - definite items outside content-sized tracks do not contribute
  - grouped indefinite items use max min-content, max-content, and min-size facts
  - grouped indefinite items are converted across every valid candidate start
  - disjoint content-sized spans receive projected sizing contributions
  - oversized indefinite spans are clamped
  - min-content and max-content track behavior differ where the oracle says they differ
  - definite lane-axis container size still lays out children at computed offsets
  - percent tolerance resolves against the chosen tolerance basis

- [ ] Keep nested grid-lanes indefinite subgrid sizing unsupported. Add an explicit test matching oracle `NestedGridLanesSubgridIndefiniteUnsupported` behavior.

- [ ] Run:

```bash
cargo test -p surgeist --test oracle lanes_intrinsic
cargo test -p surgeist --test layout_oracle lanes
cargo test -p surgeist --test layout -- grid
```

- [ ] Commit:

```bash
git status --short --branch
git diff --check
git add crates/surgeist/src/layout/grid/lanes.rs crates/surgeist/src/layout/grid/tracks.rs crates/surgeist/src/layout/grid/child.rs crates/surgeist/src/layout/grid/mod.rs crates/surgeist/tests/support/grid_layout_comparison.rs crates/surgeist/tests/layout_oracle.rs
git commit -m "Size and lay out grid-lanes"
```

---

## Task 10: Reconcile Subgrid And Grid-Lanes Interactions

**Files:**
- Modify `crates/surgeist/src/layout/grid/subgrid.rs`
- Modify `crates/surgeist/src/layout/grid/lanes.rs`
- Modify `crates/surgeist/src/layout/grid/mod.rs`
- Modify `crates/surgeist/tests/support/grid_layout_comparison.rs`
- Modify `crates/surgeist/tests/layout_oracle.rs`

- [ ] Enforce the oracle rule that a child is not a subgrid in a resolved axis where the parent is grid-lanes.

- [ ] Allow subgrid behavior in the non-lane grid axis when eligible and supported.

- [ ] Add tests for:
  - grid-lanes parent, child requests subgrid in lane axis: ineligible
  - grid-lanes parent, child requests subgrid in grid axis: eligible if parent axis mapping permits
  - ordinary grid parent, child requests subgrid in both axes: both axes eligible
  - nested subgrid inside lanes reports unsupported only for the unsupported indefinite path, not for all nesting

- [ ] Run:

```bash
cargo test -p surgeist --test layout_oracle "subgrid|lanes"
cargo test -p surgeist --test layout -- grid
```

- [ ] Commit:

```bash
git status --short --branch
git diff --check
git add crates/surgeist/src/layout/grid/subgrid.rs crates/surgeist/src/layout/grid/lanes.rs crates/surgeist/src/layout/grid/mod.rs crates/surgeist/tests/support/grid_layout_comparison.rs crates/surgeist/tests/layout_oracle.rs
git commit -m "Reconcile subgrid and grid-lanes axes"
```

---

## Task 11: Add Browser Parity Fixtures For Supported Cases

**Files:**
- Add focused subgrid fixtures under `crates/surgeist/tests/layout_browser_parity/html/subgrid`
- Add focused grid-lanes XML fixtures under `crates/surgeist/tests/layout_browser_parity/xml/grid-lanes` when generated browser data is stable enough
- Modify generated fixture manifests if the parity harness requires it
- Modify `crates/surgeist/tests/layout_browser_parity/README.md` only if commands or support matrix changed

- [ ] Add small browser fixtures for supported subgrid cases:
  - inherited columns
  - inherited rows
  - reversed direction
  - gap normal/inheritance

- [ ] Add small browser fixtures for supported grid-lanes cases only if the browser fixture generator can express the same behavior. If browser support or syntax is not stable enough, add a documented ignored test or README note instead of baking in unstable expectations.

- [ ] Do not add broad generated fixture sweeps until focused cases pass.

- [ ] Regenerate XML/manifests for only the new or changed fixtures. Do not regenerate unrelated parity domains. If the generator rewrites existing files, inspect `git diff --name-only` and keep only files required by the new focused fixtures.

- [ ] Run the narrow parity generation/check command used by this repo for subgrid and grid-lanes fixtures. If no narrow command exists, run:

```bash
cargo test -p surgeist --test layout_browser_parity -- subgrid
cargo test -p surgeist --test layout_browser_parity -- grid_lanes
```

- [ ] Commit:

```bash
git status --short --branch
git diff --check
git commit -m "Add subgrid and grid-lanes parity fixtures"
```

Before the commit, stage exact new or changed fixture files from `git status`; do not stage whole parity directories.

---

## Task 12: Final Verification And Cleanup

**Files:**
- Any touched implementation or test files

- [ ] Search for temporary names, debug output, and accidental oracle imports:

```bash
rg -n "dbg!|println!|todo!|unimplemented!|tests/support/oracle|support::oracle" crates/surgeist/src crates/surgeist/tests
```

Expected: no production oracle imports; any remaining `todo!` or `unimplemented!` is intentional and isolated from supported paths.

- [ ] Run formatting:

```bash
cargo fmt --check
```

- [ ] Run focused checks:

```bash
cargo test -p surgeist --test oracle
cargo test -p surgeist --test layout_oracle
cargo test -p surgeist --test layout -- grid
```

- [ ] Run full Surgeist checks:

```bash
cargo test -p surgeist
cargo clippy -p surgeist --all-targets --all-features -- -D warnings
```

- [ ] Inspect diff:

```bash
git status --short --branch
git diff --stat
git diff --check
```

- [ ] If final cleanup produced changes, stage the exact cleanup files reported by `git status`. Do not run a broad `git add` for final cleanup.

- [ ] Commit final cleanup if needed:

```bash
git status --short --branch
git diff --check
git commit -m "Finalize grid subgrid lanes engine"
```

---

## Acceptance Criteria

- [ ] `Display::GridLanes` and subgrid track declarations survive style-to-layout lowering.
- [ ] Ordinary grid layout remains unchanged by the phase extraction.
- [ ] Subgrid eligibility matches the oracle predicate.
- [ ] Subgrid inherited used-track sizing matches oracle track inheritance for supported axes.
- [ ] Nested inherited subgrid intrinsic contribution traversal matches oracle traversal for supported cases.
- [ ] Grid-lanes placement is implemented as a separate path from ordinary grid auto-placement.
- [ ] Grid-lanes flow tolerance is modeled and tested.
- [ ] Subgrid and grid-lanes interaction rules match the oracle.
- [ ] Unsupported browser features have explicit tests or explicit unsupported paths, not silent approximations.
- [ ] The final implementation passes:

```bash
cargo fmt --check
cargo test -p surgeist --test oracle
cargo test -p surgeist --test layout_oracle
cargo test -p surgeist --test layout -- grid
cargo test -p surgeist
cargo clippy -p surgeist --all-targets --all-features -- -D warnings
```

---

## Clean Review Record

This plan must not be marked complete until clean-context reviewers have checked it for correctness and completeness against the oracle spec, oracle implementation, and current production layout engine.

- [ ] Reviewer A: oracle/spec correctness
- [ ] Reviewer B: production architecture/test completeness
- [ ] Accepted recommendations implemented
