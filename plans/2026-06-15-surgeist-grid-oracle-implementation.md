# Surgeist Grid Oracle Implementation Plan

> **For agentic workers:** Use a task-by-task workflow with red-green-refactor checkpoints. Keep this plan local under `docs/superpowers/`; that folder is intentionally ignored by Git.

**Goal:** Implement the CSS Grid oracle described in `docs/superpowers/specs/2026-06-15-surgeist-grid-oracle-design.md` so Surgeist has a reliable independent base before subgrid and grid-lanes.

**Architecture:** Build phase-specific oracle solvers, not a second production layout engine. Each phase uses explicit inputs and returns inspectable reports. Scenario composition is thin and only combines already-tested phase outputs.

**Tech Stack:** Rust integration tests under `crates/surgeist/tests`, test-only support modules under `crates/surgeist/tests/support/oracle`, existing `cargo test -p surgeist --test oracle`, and selected comparisons from `crates/surgeist/tests/layout.rs`.

---

## File Map

- `crates/surgeist/tests/support/oracle/grid.rs`
  - Temporary compatibility file during the first split only.
  - Remove once callers import from phase modules.

- `crates/surgeist/tests/support/oracle/grid/mod.rs`
  - Public test-only grid oracle module.
  - Re-exports phase modules intentionally.

- `crates/surgeist/tests/support/oracle/grid/placement.rs`
  - Numeric base-grid placement declarations, resolved areas, occupied map, cursor state, implicit track growth.

- `crates/surgeist/tests/support/oracle/grid/tracks.rs`
  - Track definitions, sizing inputs, track states, track sizing reports, fixed/percent/flex/minmax/fit-content sizing.

- `crates/surgeist/tests/support/oracle/grid/contributions.rs`
  - Explicit item facts and contribution arithmetic. No text measurement, style resolution, or tree traversal.

- `crates/surgeist/tests/support/oracle/grid/alignment.rs`
  - Offset/gap distribution after track sizing, including safe overflow fallback.

- `crates/surgeist/tests/support/oracle/grid/scenario.rs`
  - Curated phase composition. No production layout calls.

- `crates/surgeist/tests/oracle.rs`
  - Independent oracle phase tests.

- `crates/surgeist/tests/layout.rs`
  - Selected layout-vs-oracle comparisons after oracle phases are complete.

---

## Task 1: Split Grid Oracle Modules Without Behavior Changes

**Files:**
- Create: `crates/surgeist/tests/support/oracle/grid/mod.rs`
- Create: `crates/surgeist/tests/support/oracle/grid/placement.rs`
- Create: `crates/surgeist/tests/support/oracle/grid/tracks.rs`
- Create: `crates/surgeist/tests/support/oracle/grid/contributions.rs`
- Create: `crates/surgeist/tests/support/oracle/grid/alignment.rs`
- Create: `crates/surgeist/tests/support/oracle/grid/scenario.rs`
- Delete: `crates/surgeist/tests/support/oracle/grid.rs`
- Modify: `crates/surgeist/tests/support/oracle/mod.rs`
- Modify: `crates/surgeist/tests/oracle.rs`
- Modify: `crates/surgeist/tests/layout.rs`

- [ ] **Step 1: Move current track sizing helpers**

Move `Track`, `DefiniteTracks`, `SolvedTracks`, `TrackArea`, and `EqualShareIntrinsicTracks` from `grid.rs` into `grid/tracks.rs`.

- [ ] **Step 2: Move current placement helpers**

Move `Flow`, `GridArea`, `AutoPlacer`, and `areas_overlap` from `grid.rs` into `grid/placement.rs`.

- [ ] **Step 3: Move current alignment helper**

Move `TrackAlignment` and `align_tracks` from `grid.rs` into `grid/alignment.rs`.

- [ ] **Step 4: Create empty phase modules**

Create `contributions.rs` and `scenario.rs` with module docs and no behavior yet:

```rust
//! Explicit grid item contribution facts and contribution arithmetic.

//! Curated grid oracle scenarios composed from tested phase outputs.
```

- [ ] **Step 5: Re-export phase APIs**

In `grid/mod.rs`, expose only intentional front-door APIs:

```rust
pub mod alignment;
pub mod contributions;
pub mod placement;
pub mod scenario;
pub mod tracks;

pub use alignment::{TrackAlignment, align_tracks};
pub use placement::{AutoPlacer, Flow, GridArea};
pub use tracks::{DefiniteTracks, EqualShareIntrinsicTracks, SolvedTracks, Track, TrackArea};
```

- [ ] **Step 6: Point oracle module at the new folder**

In `crates/surgeist/tests/support/oracle/mod.rs`:

```rust
pub mod grid;
```

- [ ] **Step 7: Verify no behavior changed**

Run:

```bash
cargo test -p surgeist --test oracle
cargo test -p surgeist
```

Expected: all tests pass.

- [ ] **Step 8: Commit checkpoint**

```bash
git add crates/surgeist/tests/support/oracle crates/surgeist/tests/oracle.rs crates/surgeist/tests/layout.rs
git commit -m "Split grid oracle phases"
```

---

## Task 2: Complete Numeric Base Placement Oracle

**Files:**
- Modify: `crates/surgeist/tests/support/oracle/grid/placement.rs`
- Modify: `crates/surgeist/tests/oracle.rs`
- Later comparison: `crates/surgeist/tests/layout.rs`

- [ ] **Step 1: Add failing tests for placement declarations**

Add tests in `oracle.rs` for:
- definite start/end lines
- start line plus span
- span plus end line
- auto plus span
- `auto / auto` defaults
- implicit tracks after explicit grid

Use test names like:

```rust
fn grid_placement_resolves_start_and_end_lines() {}
fn grid_placement_resolves_start_line_plus_span() {}
fn grid_placement_resolves_span_plus_end_line() {}
fn grid_placement_defaults_auto_auto_to_one_track_span() {}
fn grid_placement_extends_implicit_tracks_after_explicit_grid() {}
```

Expected red result: compile failure or unsupported placement variant.

- [ ] **Step 2: Implement oracle-native placement declarations**

Add types in `placement.rs`:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LinePlacement {
    Auto,
    Line(isize),
    Span(usize),
    LineSpan { start: isize, span: usize },
    SpanLine { span: usize, end: isize },
    Lines { start: isize, end: isize },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ItemPlacement {
    pub column: LinePlacement,
    pub row: LinePlacement,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AxisPlacement {
    pub start_line: isize,
    pub end_line: isize,
    pub span: usize,
}
```

- [ ] **Step 3: Implement definite axis resolution**

Add a resolver that handles numeric base-grid line/span cases and returns an unsupported error for named-line concepts:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlacementError {
    ZeroSpan,
    EndBeforeStart,
    UnresolvedAuto,
    NamedLinesUnsupported,
}
```

- [ ] **Step 4: Run focused placement tests**

Run:

```bash
cargo test -p surgeist --test oracle grid_placement
```

Expected: all new placement tests pass.

- [ ] **Step 5: Add auto-placement report**

Add:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlacementReport {
    pub areas: Vec<GridArea>,
    pub occupied: Vec<GridArea>,
    pub implicit_columns_before: usize,
    pub implicit_columns_after: usize,
    pub implicit_rows_before: usize,
    pub implicit_rows_after: usize,
    pub cursor: PlacementCursor,
}
```

- [ ] **Step 6: Add dense and column-flow matrix tests**

Extend existing auto-placement tests to assert full reports, not just returned areas.

- [ ] **Step 7: Compare selected layout placement tests against oracle**

Convert only cases where the oracle is now complete enough:
- `grid_definite_column_line_places_item_in_explicit_track`
- `grid_definite_column_line_span_resolves_from_start_line`
- `grid_definite_column_span_line_resolves_to_end_line`
- `grid_column_span_auto_places_across_multiple_free_tracks`
- `grid_dense_auto_flow_backfills_earlier_free_cells`

- [ ] **Step 8: Verify and commit**

Run:

```bash
cargo test -p surgeist --test oracle
cargo test -p surgeist --test layout grid_definite_column_line_places_item_in_explicit_track
cargo test -p surgeist
```

Commit:

```bash
git add crates/surgeist/tests/support/oracle/grid/placement.rs crates/surgeist/tests/oracle.rs crates/surgeist/tests/layout.rs
git commit -m "Complete numeric grid placement oracle"
```

---

## Task 3: Track Sizing Reports For Initialization And Definite Tracks

**Files:**
- Modify: `crates/surgeist/tests/support/oracle/grid/tracks.rs`
- Modify: `crates/surgeist/tests/oracle.rs`
- Later comparison: `crates/surgeist/tests/layout.rs`

- [ ] **Step 1: Add failing tests for track report phases**

Add tests that assert initialized bases and growth limits for fixed, percent, flex, auto, min-content, max-content, fit-content, and minmax tracks.

Use names like:

```rust
fn grid_track_report_initializes_fixed_percent_and_flex_tracks() {}
fn grid_track_report_initializes_auto_and_intrinsic_keywords() {}
fn grid_track_report_initializes_minmax_growth_limits() {}
```

- [ ] **Step 2: Add explicit track model**

Add:

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TrackMin {
    Fixed(f32),
    Percent(f32),
    Auto,
    MinContent,
    MaxContent,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TrackMax {
    Fixed(f32),
    Percent(f32),
    Flex(f32),
    Auto,
    MinContent,
    MaxContent,
    FitContent(f32),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GridTrack {
    pub min: TrackMin,
    pub max: TrackMax,
}
```

- [ ] **Step 3: Add report structs**

Add:

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct TrackState {
    pub tracks: Vec<TrackSize>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrackSize {
    pub base: f32,
    pub growth_limit: GrowthLimit,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GrowthLimit {
    Definite(f32),
    Infinite,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TrackSizingReport {
    pub initialized: TrackState,
    pub after_intrinsic_minimums: TrackState,
    pub after_content_based_minimums: TrackState,
    pub after_spanning_items: TrackState,
    pub after_maximize_tracks: TrackState,
    pub flex_fraction: Option<f32>,
    pub after_flexing: TrackState,
    pub after_stretch: TrackState,
    pub final_tracks: Vec<SolvedTrack>,
}
```

- [ ] **Step 4: Implement initialization**

Implement only initialization and definite track finalization. Unsupported intrinsic phases should copy the previous state until implemented, but the report fields must exist.

- [ ] **Step 5: Verify tests**

Run:

```bash
cargo test -p surgeist --test oracle grid_track_report
```

- [ ] **Step 6: Commit checkpoint**

```bash
git add crates/surgeist/tests/support/oracle/grid/tracks.rs crates/surgeist/tests/oracle.rs
git commit -m "Add grid track sizing reports"
```

---

## Task 4: Contributions Phase

**Files:**
- Modify: `crates/surgeist/tests/support/oracle/grid/contributions.rs`
- Modify: `crates/surgeist/tests/support/oracle/grid/tracks.rs`
- Modify: `crates/surgeist/tests/oracle.rs`

- [ ] **Step 1: Add failing contribution tests**

Add tests for:
- minimum contribution
- min-content contribution
- max-content contribution
- preferred definite size
- min/max clamps
- margins
- automatic-minimum eligibility as explicit input

- [ ] **Step 2: Add item fact structs**

Add:

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ItemContributionFacts {
    pub area: GridArea,
    pub min_content: f32,
    pub max_content: f32,
    pub preferred: ContributionSize,
    pub min_size: ContributionSize,
    pub max_size: ContributionSize,
    pub margin_before: f32,
    pub margin_after: f32,
    pub automatic_minimum_applies: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ContributionSize {
    Auto,
    Definite(f32),
    Infinite,
}
```

- [ ] **Step 3: Add contribution output**

Add:

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ItemContributions {
    pub minimum: f32,
    pub min_content: f32,
    pub max_content: f32,
    pub limited_min_content: f32,
    pub limited_max_content: f32,
}
```

- [ ] **Step 4: Implement arithmetic only from supplied facts**

Do not derive automatic minimum eligibility, aspect-ratio transfer, replaced sizes, or text measurement.

- [ ] **Step 5: Verify and commit**

Run:

```bash
cargo test -p surgeist --test oracle contribution
cargo test -p surgeist --test oracle
```

Commit:

```bash
git add crates/surgeist/tests/support/oracle/grid/contributions.rs crates/surgeist/tests/support/oracle/grid/tracks.rs crates/surgeist/tests/oracle.rs
git commit -m "Add grid contribution oracle"
```

---

## Task 5: Intrinsic Track Growth And Spanning Items

**Files:**
- Modify: `crates/surgeist/tests/support/oracle/grid/tracks.rs`
- Modify: `crates/surgeist/tests/support/oracle/grid/contributions.rs`
- Modify: `crates/surgeist/tests/oracle.rs`
- Later comparison: `crates/surgeist/tests/layout.rs`

- [ ] **Step 1: Add failing tests for single-span intrinsic growth**

Assert report states after intrinsic minimums and content-based minimums.

- [ ] **Step 2: Add failing tests for spanning growth**

Cover:
- homogeneous auto tracks
- min-content plus auto tracks
- fit-content caps
- percent reservation behavior when represented by explicit facts
- growth limit clamping

- [ ] **Step 3: Implement single-span growth**

Use item contributions to grow base sizes and limits for items spanning one track.

- [ ] **Step 4: Implement spanning distribution**

Handle mixed track categories in explicit, named helper functions. Each function should update the report state separately so tests can assert the phase boundary.

- [ ] **Step 5: Compare selected layout tests**

Convert selected tests where the oracle phase is complete:
- `grid_auto_track_uses_single_item_intrinsic_contribution`
- `grid_spanning_item_distributes_intrinsic_contribution_across_auto_tracks`
- `grid_spanning_item_grows_auto_track_after_min_content_track`
- `grid_clipped_spanning_item_distributes_across_min_content_and_auto_tracks`

- [ ] **Step 6: Verify and commit**

Run:

```bash
cargo test -p surgeist --test oracle grid_intrinsic
cargo test -p surgeist --test layout grid_spanning_item
cargo test -p surgeist
```

Commit:

```bash
git add crates/surgeist/tests/support/oracle/grid/tracks.rs crates/surgeist/tests/support/oracle/grid/contributions.rs crates/surgeist/tests/oracle.rs crates/surgeist/tests/layout.rs
git commit -m "Add intrinsic grid track oracle"
```

---

## Task 6: Maximize, Flex, And Stretch Track Sizing

**Files:**
- Modify: `crates/surgeist/tests/support/oracle/grid/tracks.rs`
- Modify: `crates/surgeist/tests/oracle.rs`
- Later comparison: `crates/surgeist/tests/layout.rs`

- [ ] **Step 1: Add failing tests for maximize-tracks growth**

Assert `after_maximize_tracks` grows eligible tracks from free space.

- [ ] **Step 2: Add failing tests for flex fraction**

Cover:
- fixed plus flex tracks
- multiple flex factors
- zero leftover
- indefinite available space behavior when represented explicitly

- [ ] **Step 3: Add failing tests for stretch-auto-track growth**

Assert stretch affects auto track sizes in `after_stretch`, not alignment offsets.

- [ ] **Step 4: Implement maximize-tracks**

Distribute free space according to the modeled track eligibility. Unsupported mixed cases should return explicit unsupported errors.

- [ ] **Step 5: Implement flex fraction resolution**

Return `flex_fraction` in the report and use it for final flex track sizes.

- [ ] **Step 6: Implement stretch-auto-track growth**

Grow auto tracks when alignment is stretch and free space remains after track sizing.

- [ ] **Step 7: Compare selected layout tests**

Convert selected tests:
- `grid_fraction_tracks_share_leftover_space_after_fixed_tracks_and_gaps`
- `grid_fraction_tracks_use_available_space_when_container_size_is_auto`
- `grid_fraction_tracks_clamp_available_space_to_min_size`
- `grid_stretch_distributes_free_space_to_auto_tracks`

- [ ] **Step 8: Verify and commit**

Run:

```bash
cargo test -p surgeist --test oracle grid_flex
cargo test -p surgeist --test oracle grid_stretch
cargo test -p surgeist
```

Commit:

```bash
git add crates/surgeist/tests/support/oracle/grid/tracks.rs crates/surgeist/tests/oracle.rs crates/surgeist/tests/layout.rs
git commit -m "Add flex and stretch grid track oracle"
```

---

## Task 7: Alignment Reports

**Files:**
- Modify: `crates/surgeist/tests/support/oracle/grid/alignment.rs`
- Modify: `crates/surgeist/tests/oracle.rs`
- Later comparison: `crates/surgeist/tests/layout.rs`

- [ ] **Step 1: Add failing tests for alignment report**

Cover:
- start
- end
- center
- space-between
- space-around
- space-evenly
- safe overflow fallback
- row axis
- column axis

- [ ] **Step 2: Replace tuple-like helper with report**

Add:

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct AlignmentReport {
    pub leading_offset: f32,
    pub distributed_gap: f32,
    pub offsets: Vec<f32>,
    pub safe_fallback_used: bool,
}
```

- [ ] **Step 3: Keep stretch out of alignment**

Do not add stretch here. Stretch stays in `tracks.rs`.

- [ ] **Step 4: Compare selected layout tests**

Convert selected tests:
- `grid_justify_content_center_offsets_tracks_inside_inner_width`
- `grid_align_content_center_offsets_tracks_inside_inner_height`
- `grid_justify_content_space_between_distributes_free_width_between_tracks`
- `grid_justify_content_space_around_and_evenly_distribute_free_width`
- `grid_safe_align_content_falls_back_to_start_when_tracks_overflow`

- [ ] **Step 5: Verify and commit**

Run:

```bash
cargo test -p surgeist --test oracle grid_alignment
cargo test -p surgeist
```

Commit:

```bash
git add crates/surgeist/tests/support/oracle/grid/alignment.rs crates/surgeist/tests/oracle.rs crates/surgeist/tests/layout.rs
git commit -m "Add grid alignment oracle reports"
```

---

## Task 8: Curated Scenario Composition

**Files:**
- Modify: `crates/surgeist/tests/support/oracle/grid/scenario.rs`
- Modify: `crates/surgeist/tests/oracle.rs`
- Modify: `crates/surgeist/tests/layout.rs`

- [ ] **Step 1: Add failing scenario tests**

Add scenarios that compose phases but keep inputs explicit:
- fixed + flex tracks with definite placement
- auto-placement plus explicit tracks
- intrinsic contribution plus alignment
- spanning item plus flex finalization

- [ ] **Step 2: Add scenario report**

Add:

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct GridScenarioReport {
    pub placement: PlacementReport,
    pub columns: TrackSizingReport,
    pub rows: TrackSizingReport,
    pub column_alignment: AlignmentReport,
    pub row_alignment: AlignmentReport,
    pub item_rects: Vec<GridItemRect>,
}
```

- [ ] **Step 3: Implement thin composition**

Scenario code may call oracle placement, contribution, track, and alignment solvers. It may not call production layout or infer style/tree facts.

- [ ] **Step 4: Compare selected layout tests**

Use scenario reports for a few high-signal tests. Do not replace every grid test yet.

- [ ] **Step 5: Verify and commit**

Run:

```bash
cargo test -p surgeist --test oracle scenario
cargo test -p surgeist
```

Commit:

```bash
git add crates/surgeist/tests/support/oracle/grid/scenario.rs crates/surgeist/tests/oracle.rs crates/surgeist/tests/layout.rs
git commit -m "Add curated grid oracle scenarios"
```

---

## Task 9: Clean-Context Review And Completion Audit

**Files:**
- Read-only unless review finds issues.

- [ ] **Step 1: Run full verification**

Run:

```bash
cargo fmt
cargo test -p surgeist --test oracle
cargo test -p surgeist
```

- [ ] **Step 2: Audit against spec completion criteria**

For each completion criterion in the spec, record the evidence:
- oracle test names
- report fields
- layout comparison tests
- unsupported-case tests

- [ ] **Step 3: Dispatch clean-context review**

Ask a clean-context reviewer to inspect:
- the spec
- oracle phase modules
- oracle tests
- layout comparisons

The reviewer should look for overclaiming names, hidden production-layout dependency, and missing required phase outputs.

- [ ] **Step 4: Fix review findings**

Each accepted finding gets its own failing test first, then implementation, then focused verification.

- [ ] **Step 5: Final verification**

Run:

```bash
cargo fmt
cargo test -p surgeist --test oracle
cargo test -p surgeist
```

- [ ] **Step 6: Commit final checkpoint**

```bash
git add crates/surgeist/tests/support/oracle crates/surgeist/tests/oracle.rs crates/surgeist/tests/layout.rs
git commit -m "Complete grid oracle"
```

Only after this task passes should the active goal be considered complete.

---

## Plan Self-Review

Spec coverage:
- Phase split is covered by Task 1.
- Numeric base placement is covered by Task 2.
- Track sizing reports are covered by Tasks 3, 5, and 6.
- Contribution facts are covered by Task 4.
- Alignment reports are covered by Task 7.
- Scenario composition is covered by Task 8.
- Clean-context review and completion audit are covered by Task 9.

Known sequencing constraint:
- Layout comparisons happen only after the relevant oracle phase is independently green.
- Unsupported cases should fail explicitly instead of guessing.
- Every behavior change starts with a failing oracle test.
