# Surgeist Subgrid And Grid-Lanes Oracle Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the oracle vocabulary and phase solvers described in `docs/superpowers/specs/2026-06-16-surgeist-subgrid-grid-lanes-oracle-design.md` for subgrid and grid-lanes without turning the oracle into a second production layout engine.

**Architecture:** Extend the existing test-only grid oracle with three focused modules: `axis.rs`, `subgrid.rs`, and `lanes.rs`. Each module consumes explicit typed facts, returns report structs with intermediate values, and avoids production tree traversal, style resolution, child measurement, or calls into `compute_grid`. `scenario.rs` remains the only composition layer.

**Tech Stack:** Rust test support under `crates/surgeist/tests/support/oracle/grid`, pure oracle tests in `crates/surgeist/tests/oracle.rs`, composed layout comparisons in `crates/surgeist/tests/layout_oracle.rs`, focused verification with `cargo test -p surgeist --test oracle`, `cargo test -p surgeist --test layout_oracle`, and final verification with `cargo test -p surgeist`.

---

## File Map

- Create `crates/surgeist/tests/support/oracle/grid/axis.rs`
  - Owns axis vocabulary, writing-mode facts, flow-aware mapping, and reversed-axis reports.

- Create `crates/surgeist/tests/support/oracle/grid/subgrid.rs`
  - Owns subgrid eligibility, inherited track copying, gap reports, nested subgrid traversal, and unsupported errors.

- Create `crates/surgeist/tests/support/oracle/grid/lanes.rs`
  - Owns grid-lanes axis derivation, lane placement, lane intrinsic sizing inputs/reports, and flow tolerance.

- Modify `crates/surgeist/tests/support/oracle/grid/mod.rs`
  - Exposes the new modules and intentional front-door types.

- Modify `crates/surgeist/tests/support/oracle/grid/scenario.rs`
  - Adds guardrails and small composition helpers that consume explicit phase reports.

- Modify `crates/surgeist/tests/oracle.rs`
  - Adds pure phase tests for axis mapping, subgrid eligibility, inherited track copying, nested intrinsic traversal, lanes placement, lanes intrinsic grouping, and small composed scenarios after the phase reports are independently tested.

---

## Guardrails For Every Task

- Do not use production `NodeInput`, production `Tree`, retained tree types, or `compute_grid` inside oracle solvers.
- Do not add child measurement callbacks to oracle modules.
- Do not parse CSS strings in oracle modules.
- Prefer explicit unsupported errors over approximate behavior.
- Keep reports visible and assert intermediate state in tests.
- Commit after each task with a short concrete message.

---

## Task 1: Add Shared Axis Vocabulary

**Files:**
- Create: `crates/surgeist/tests/support/oracle/grid/axis.rs`
- Modify: `crates/surgeist/tests/support/oracle/grid/mod.rs`
- Modify: `crates/surgeist/tests/oracle.rs`

- [ ] **Step 1: Add failing axis mapping tests**

Add these tests to `crates/surgeist/tests/oracle.rs` near the existing grid oracle tests:

```rust
#[test]
fn oracle_axis_mapping_preserves_parallel_horizontal_axes() {
    let report = support::oracle::grid::map_axis(support::oracle::grid::AxisMappingInput {
        queried_axis: support::oracle::grid::GridAxis::Column,
        parent_writing_mode: support::oracle::grid::OracleWritingMode::HorizontalTb,
        child_writing_mode: support::oracle::grid::OracleWritingMode::HorizontalTb,
        parent_direction: support::oracle::grid::OracleDirection::Ltr,
        child_direction: support::oracle::grid::OracleDirection::Ltr,
        parent_flipped_in_resolved_axis: false,
        child_flipped_in_resolved_axis: false,
    })
    .unwrap();

    assert_eq!(report.parent_axis, support::oracle::grid::GridAxis::Column);
    assert_eq!(report.child_axis, support::oracle::grid::GridAxis::Column);
    assert!(!report.reversed);
}

#[test]
fn oracle_axis_mapping_rejects_vertical_mapping_without_explicit_support() {
    let err = support::oracle::grid::map_axis(support::oracle::grid::AxisMappingInput {
        queried_axis: support::oracle::grid::GridAxis::Column,
        parent_writing_mode: support::oracle::grid::OracleWritingMode::HorizontalTb,
        child_writing_mode: support::oracle::grid::OracleWritingMode::VerticalRl,
        parent_direction: support::oracle::grid::OracleDirection::Ltr,
        child_direction: support::oracle::grid::OracleDirection::Ltr,
        parent_flipped_in_resolved_axis: false,
        child_flipped_in_resolved_axis: false,
    })
    .unwrap_err();

    assert_eq!(err, support::oracle::grid::AxisMappingError::VerticalWritingModeUnsupported);
}

#[test]
fn oracle_axis_mapping_reports_reversed_when_flipped_states_differ() {
    let report = support::oracle::grid::map_axis(support::oracle::grid::AxisMappingInput {
        queried_axis: support::oracle::grid::GridAxis::Row,
        parent_writing_mode: support::oracle::grid::OracleWritingMode::HorizontalTb,
        child_writing_mode: support::oracle::grid::OracleWritingMode::HorizontalTb,
        parent_direction: support::oracle::grid::OracleDirection::Rtl,
        child_direction: support::oracle::grid::OracleDirection::Ltr,
        parent_flipped_in_resolved_axis: true,
        child_flipped_in_resolved_axis: false,
    })
    .unwrap();

    assert_eq!(report.parent_axis, support::oracle::grid::GridAxis::Row);
    assert_eq!(report.child_axis, support::oracle::grid::GridAxis::Row);
    assert!(report.reversed);
}
```

- [ ] **Step 2: Run the focused failing tests**

Run:

```bash
cargo test -p surgeist --test oracle oracle_axis_mapping
```

Expected: compile failure because `axis.rs`, `AxisMappingInput`, `OracleWritingMode`, `OracleDirection`, and `map_axis` do not exist yet.

- [ ] **Step 3: Add `axis.rs`**

Create `crates/surgeist/tests/support/oracle/grid/axis.rs`:

```rust
use super::placement::GridAxis;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OracleWritingMode {
    HorizontalTb,
    VerticalLr,
    VerticalRl,
}

impl OracleWritingMode {
    #[must_use]
    pub const fn is_vertical(self) -> bool {
        matches!(self, Self::VerticalLr | Self::VerticalRl)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OracleDirection {
    Ltr,
    Rtl,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AxisMappingError {
    VerticalWritingModeUnsupported,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AxisMappingInput {
    pub queried_axis: GridAxis,
    pub parent_writing_mode: OracleWritingMode,
    pub child_writing_mode: OracleWritingMode,
    pub parent_direction: OracleDirection,
    pub child_direction: OracleDirection,
    pub parent_flipped_in_resolved_axis: bool,
    pub child_flipped_in_resolved_axis: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AxisMappingReport {
    pub queried_axis: GridAxis,
    pub parent_axis: GridAxis,
    pub child_axis: GridAxis,
    pub parent_writing_mode: OracleWritingMode,
    pub child_writing_mode: OracleWritingMode,
    pub parent_direction: OracleDirection,
    pub child_direction: OracleDirection,
    pub parent_flipped_in_resolved_axis: bool,
    pub child_flipped_in_resolved_axis: bool,
    pub reversed: bool,
}

#[must_use]
pub fn map_axis(input: AxisMappingInput) -> Result<AxisMappingReport, AxisMappingError> {
    if input.parent_writing_mode.is_vertical() || input.child_writing_mode.is_vertical() {
        return Err(AxisMappingError::VerticalWritingModeUnsupported);
    }

    Ok(AxisMappingReport {
        queried_axis: input.queried_axis,
        parent_axis: input.queried_axis,
        child_axis: input.queried_axis,
        parent_writing_mode: input.parent_writing_mode,
        child_writing_mode: input.child_writing_mode,
        parent_direction: input.parent_direction,
        child_direction: input.child_direction,
        parent_flipped_in_resolved_axis: input.parent_flipped_in_resolved_axis,
        child_flipped_in_resolved_axis: input.child_flipped_in_resolved_axis,
        reversed: input.parent_flipped_in_resolved_axis != input.child_flipped_in_resolved_axis,
    })
}

#[must_use]
pub const fn opposite_axis(axis: GridAxis) -> GridAxis {
    match axis {
        GridAxis::Column => GridAxis::Row,
        GridAxis::Row => GridAxis::Column,
    }
}
```

- [ ] **Step 4: Export the axis API**

Modify `crates/surgeist/tests/support/oracle/grid/mod.rs`:

```rust
pub mod alignment;
pub mod axis;
pub mod contributions;
pub mod placement;
pub mod scenario;
pub mod tracks;

#[allow(unused_imports)]
pub use axis::{
    AxisMappingError, AxisMappingInput, AxisMappingReport, OracleDirection, OracleWritingMode,
    map_axis, opposite_axis,
};
```

Preserve the existing `pub use` groups for alignment, contributions, placement, scenario, and tracks.

- [ ] **Step 5: Verify axis mapping**

Run:

```bash
cargo test -p surgeist --test oracle oracle_axis_mapping
```

Expected: all three axis mapping tests pass.

- [ ] **Step 6: Commit**

```bash
git add crates/surgeist/tests/support/oracle/grid/axis.rs crates/surgeist/tests/support/oracle/grid/mod.rs crates/surgeist/tests/oracle.rs
git commit -m "Add grid oracle axis mapping"
```

---

## Task 2: Add Subgrid Eligibility And Shared Error Types

**Files:**
- Create: `crates/surgeist/tests/support/oracle/grid/subgrid.rs`
- Modify: `crates/surgeist/tests/support/oracle/grid/mod.rs`
- Modify: `crates/surgeist/tests/oracle.rs`

- [ ] **Step 1: Add failing subgrid eligibility tests**

Add these tests to `crates/surgeist/tests/oracle.rs`:

```rust
#[test]
fn oracle_subgrid_eligibility_accepts_requested_axis_with_parent_grid() {
    let report = support::oracle::grid::subgrid_eligibility(
        support::oracle::grid::SubgridEligibilityInput {
            requested: true,
            has_parent_grid: true,
            independent_formatting_context: false,
            excluded_from_normal_layout: false,
            parent_is_lanes_in_resolved_axis: false,
        },
    );

    assert!(report.eligible);
    assert_eq!(report.reason, None);
}

#[test]
fn oracle_subgrid_eligibility_rejects_lanes_parent_in_resolved_axis() {
    let report = support::oracle::grid::subgrid_eligibility(
        support::oracle::grid::SubgridEligibilityInput {
            requested: true,
            has_parent_grid: true,
            independent_formatting_context: false,
            excluded_from_normal_layout: false,
            parent_is_lanes_in_resolved_axis: true,
        },
    );

    assert!(!report.eligible);
    assert_eq!(
        report.reason,
        Some(support::oracle::grid::SubgridIneligibleReason::ParentIsLanesInResolvedAxis)
    );
}

#[test]
fn oracle_subgrid_eligibility_reports_first_blocking_reason() {
    let report = support::oracle::grid::subgrid_eligibility(
        support::oracle::grid::SubgridEligibilityInput {
            requested: false,
            has_parent_grid: false,
            independent_formatting_context: true,
            excluded_from_normal_layout: true,
            parent_is_lanes_in_resolved_axis: true,
        },
    );

    assert!(!report.eligible);
    assert_eq!(
        report.reason,
        Some(support::oracle::grid::SubgridIneligibleReason::NotRequested)
    );
}

#[test]
fn oracle_subgrid_eligibility_reports_each_blocking_reason() {
    let cases = [
        (
            support::oracle::grid::SubgridEligibilityInput {
                requested: true,
                has_parent_grid: false,
                independent_formatting_context: false,
                excluded_from_normal_layout: false,
                parent_is_lanes_in_resolved_axis: false,
            },
            support::oracle::grid::SubgridIneligibleReason::NoParentGrid,
        ),
        (
            support::oracle::grid::SubgridEligibilityInput {
                requested: true,
                has_parent_grid: true,
                independent_formatting_context: true,
                excluded_from_normal_layout: false,
                parent_is_lanes_in_resolved_axis: false,
            },
            support::oracle::grid::SubgridIneligibleReason::IndependentFormattingContext,
        ),
        (
            support::oracle::grid::SubgridEligibilityInput {
                requested: true,
                has_parent_grid: true,
                independent_formatting_context: false,
                excluded_from_normal_layout: true,
                parent_is_lanes_in_resolved_axis: false,
            },
            support::oracle::grid::SubgridIneligibleReason::ExcludedFromNormalLayout,
        ),
    ];

    for (input, reason) in cases {
        assert_eq!(support::oracle::grid::subgrid_eligibility(input).reason, Some(reason));
    }
}

#[test]
fn oracle_subgrid_eligibility_allows_standalone_axis_when_resolved_parent_axis_is_not_lanes() {
    let report = support::oracle::grid::subgrid_eligibility(
        support::oracle::grid::SubgridEligibilityInput {
            requested: true,
            has_parent_grid: true,
            independent_formatting_context: false,
            excluded_from_normal_layout: false,
            parent_is_lanes_in_resolved_axis: false,
        },
    );

    assert!(report.eligible);
}
```

- [ ] **Step 2: Run focused failing tests**

Run:

```bash
cargo test -p surgeist --test oracle oracle_subgrid_eligibility
```

Expected: compile failure because `subgrid.rs` and eligibility types do not exist yet.

- [ ] **Step 3: Add `subgrid.rs` eligibility types**

Create `crates/surgeist/tests/support/oracle/grid/subgrid.rs`:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OracleGridError {
    NamedLineInheritanceUnsupported,
    BaselineInferenceUnsupported,
    MissingIntrinsicMinTrackFacts,
    NestedGridLanesSubgridIndefiniteUnsupported,
    StandaloneSubgridTraversalUnsupported,
    EmptyTrackList,
    SpanOutOfRange,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SubgridEligibilityInput {
    pub requested: bool,
    pub has_parent_grid: bool,
    pub independent_formatting_context: bool,
    pub excluded_from_normal_layout: bool,
    pub parent_is_lanes_in_resolved_axis: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubgridIneligibleReason {
    NotRequested,
    NoParentGrid,
    IndependentFormattingContext,
    ExcludedFromNormalLayout,
    ParentIsLanesInResolvedAxis,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SubgridEligibilityReport {
    pub eligible: bool,
    pub reason: Option<SubgridIneligibleReason>,
}

#[must_use]
pub fn subgrid_eligibility(input: SubgridEligibilityInput) -> SubgridEligibilityReport {
    let reason = if !input.requested {
        Some(SubgridIneligibleReason::NotRequested)
    } else if !input.has_parent_grid {
        Some(SubgridIneligibleReason::NoParentGrid)
    } else if input.independent_formatting_context {
        Some(SubgridIneligibleReason::IndependentFormattingContext)
    } else if input.excluded_from_normal_layout {
        Some(SubgridIneligibleReason::ExcludedFromNormalLayout)
    } else if input.parent_is_lanes_in_resolved_axis {
        Some(SubgridIneligibleReason::ParentIsLanesInResolvedAxis)
    } else {
        None
    };

    SubgridEligibilityReport {
        eligible: reason.is_none(),
        reason,
    }
}
```

- [ ] **Step 4: Export the subgrid API**

Modify `crates/surgeist/tests/support/oracle/grid/mod.rs`:

```rust
pub mod subgrid;

#[allow(unused_imports)]
pub use subgrid::{
    OracleGridError, SubgridEligibilityInput, SubgridEligibilityReport,
    SubgridIneligibleReason, subgrid_eligibility,
};
```

Keep all existing module exports.

- [ ] **Step 5: Verify subgrid eligibility**

Run:

```bash
cargo test -p surgeist --test oracle oracle_subgrid_eligibility
```

Expected: all three eligibility tests pass.

- [ ] **Step 6: Commit**

```bash
git add crates/surgeist/tests/support/oracle/grid/subgrid.rs crates/surgeist/tests/support/oracle/grid/mod.rs crates/surgeist/tests/oracle.rs
git commit -m "Add subgrid oracle eligibility"
```

---

## Task 3: Implement Subgrid Track Inheritance

**Files:**
- Modify: `crates/surgeist/tests/support/oracle/grid/subgrid.rs`
- Modify: `crates/surgeist/tests/support/oracle/grid/mod.rs`
- Modify: `crates/surgeist/tests/oracle.rs`

- [ ] **Step 1: Add failing inherited-track tests**

Add these tests to `crates/surgeist/tests/oracle.rs`:

```rust
#[test]
fn oracle_subgrid_copies_parent_tracks_for_span() {
    let report = support::oracle::grid::inherit_subgrid_tracks(
        support::oracle::grid::SubgridTrackInheritanceInput {
            parent_tracks: vec![40.0, 60.0, 90.0],
            parent_span: support::oracle::grid::TrackSpan::new(2, 4),
            reversed: false,
            start_mbp: 0.0,
            end_mbp: 0.0,
            parent_gap: support::oracle::grid::OracleGapReport::length(10.0),
            subgrid_gap: support::oracle::grid::OracleGapReport::length(10.0),
        },
    )
    .unwrap();

    assert_eq!(report.copied_parent_tracks, vec![60.0, 90.0]);
    assert_eq!(report.final_tracks, vec![60.0, 90.0]);
}

#[test]
fn oracle_subgrid_reverses_copied_tracks_before_mbp_removal() {
    let report = support::oracle::grid::inherit_subgrid_tracks(
        support::oracle::grid::SubgridTrackInheritanceInput {
            parent_tracks: vec![40.0, 60.0, 90.0],
            parent_span: support::oracle::grid::TrackSpan::new(1, 4),
            reversed: true,
            start_mbp: 10.0,
            end_mbp: 20.0,
            parent_gap: support::oracle::grid::OracleGapReport::length(10.0),
            subgrid_gap: support::oracle::grid::OracleGapReport::length(10.0),
        },
    )
    .unwrap();

    assert_eq!(report.after_reversal, vec![90.0, 60.0, 40.0]);
    assert_eq!(report.final_tracks, vec![80.0, 60.0, 20.0]);
}

#[test]
fn oracle_subgrid_resolves_normal_gap_to_parent_gap() {
    let report = support::oracle::grid::inherit_subgrid_tracks(
        support::oracle::grid::SubgridTrackInheritanceInput {
            parent_tracks: vec![50.0, 50.0],
            parent_span: support::oracle::grid::TrackSpan::new(1, 3),
            reversed: false,
            start_mbp: 0.0,
            end_mbp: 0.0,
            parent_gap: support::oracle::grid::OracleGapReport::length(20.0),
            subgrid_gap: support::oracle::grid::OracleGapReport::normal_resolved_to(20.0),
        },
    )
    .unwrap();

    assert_eq!(report.gap_difference, 0.0);
    assert_eq!(report.final_tracks, vec![50.0, 50.0]);
}

#[test]
fn oracle_subgrid_applies_gap_difference_to_internal_edges() {
    let report = support::oracle::grid::inherit_subgrid_tracks(
        support::oracle::grid::SubgridTrackInheritanceInput {
            parent_tracks: vec![50.0, 50.0, 50.0],
            parent_span: support::oracle::grid::TrackSpan::new(1, 4),
            reversed: false,
            start_mbp: 0.0,
            end_mbp: 0.0,
            parent_gap: support::oracle::grid::OracleGapReport::length(10.0),
            subgrid_gap: support::oracle::grid::OracleGapReport::length(20.0),
        },
    )
    .unwrap();

    assert_eq!(report.gap_difference, 5.0);
    assert_eq!(report.final_tracks, vec![45.0, 40.0, 45.0]);
}

#[test]
fn oracle_subgrid_adds_negative_gap_difference_to_internal_edges() {
    let report = support::oracle::grid::inherit_subgrid_tracks(
        support::oracle::grid::SubgridTrackInheritanceInput {
            parent_tracks: vec![40.0, 40.0],
            parent_span: support::oracle::grid::TrackSpan::new(1, 3),
            reversed: false,
            start_mbp: 0.0,
            end_mbp: 0.0,
            parent_gap: support::oracle::grid::OracleGapReport::length(20.0),
            subgrid_gap: support::oracle::grid::OracleGapReport::length(10.0),
        },
    )
    .unwrap();

    assert_eq!(report.gap_difference, -5.0);
    assert_eq!(report.final_tracks, vec![45.0, 45.0]);
}

#[test]
fn oracle_subgrid_mbp_removal_clamps_tracks_to_zero() {
    let report = support::oracle::grid::inherit_subgrid_tracks(
        support::oracle::grid::SubgridTrackInheritanceInput {
            parent_tracks: vec![5.0, 10.0],
            parent_span: support::oracle::grid::TrackSpan::new(1, 3),
            reversed: false,
            start_mbp: 20.0,
            end_mbp: 0.0,
            parent_gap: support::oracle::grid::OracleGapReport::length(0.0),
            subgrid_gap: support::oracle::grid::OracleGapReport::length(0.0),
        },
    )
    .unwrap();

    assert_eq!(report.final_tracks, vec![0.0, 0.0]);
}
```

- [ ] **Step 2: Run focused failing tests**

Run:

```bash
cargo test -p surgeist --test oracle oracle_subgrid_
```

Expected: existing eligibility tests pass, new inherited-track tests fail to compile.

- [ ] **Step 3: Add track span and gap reports**

Add to `subgrid.rs`:

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrackSpan {
    pub start: usize,
    pub end: usize,
}

impl TrackSpan {
    #[must_use]
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn checked_len(self) -> Result<usize, OracleGridError> {
        if self.start == 0 || self.end <= self.start {
            Err(OracleGridError::SpanOutOfRange)
        } else {
            Ok(self.end - self.start)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OracleGap {
    Normal,
    Length(f32),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OracleGapReport {
    pub specified: OracleGap,
    pub resolved: f32,
}

impl OracleGapReport {
    #[must_use]
    pub const fn length(value: f32) -> Self {
        Self {
            specified: OracleGap::Length(value),
            resolved: value,
        }
    }

    #[must_use]
    pub const fn normal_resolved_to(parent_gap: f32) -> Self {
        Self {
            specified: OracleGap::Normal,
            resolved: parent_gap,
        }
    }
}
```

- [ ] **Step 4: Add inherited-track input/report and solver**

Add to `subgrid.rs`:

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct SubgridTrackInheritanceInput {
    pub parent_tracks: Vec<f32>,
    pub parent_span: TrackSpan,
    pub reversed: bool,
    pub start_mbp: f32,
    pub end_mbp: f32,
    pub parent_gap: OracleGapReport,
    pub subgrid_gap: OracleGapReport,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SubgridTrackInheritanceReport {
    pub parent_span: TrackSpan,
    pub copied_parent_tracks: Vec<f32>,
    pub reversed: bool,
    pub after_reversal: Vec<f32>,
    pub start_mbp_removed: Vec<f32>,
    pub end_mbp_removed: Vec<f32>,
    pub gap_difference: f32,
    pub parent_gap: OracleGapReport,
    pub subgrid_gap: OracleGapReport,
    pub final_tracks: Vec<f32>,
}

pub fn inherit_subgrid_tracks(
    input: SubgridTrackInheritanceInput,
) -> Result<SubgridTrackInheritanceReport, OracleGridError> {
    if input.parent_tracks.is_empty() {
        return Err(OracleGridError::EmptyTrackList);
    }
    if input.parent_span.start == 0
        || input.parent_span.end <= input.parent_span.start
        || input.parent_span.end > input.parent_tracks.len() + 1
    {
        return Err(OracleGridError::SpanOutOfRange);
    }

    let start_index = input.parent_span.start - 1;
    let end_index = input.parent_span.end - 1;
    let copied_parent_tracks = input.parent_tracks[start_index..end_index].to_vec();

    let mut after_reversal = copied_parent_tracks.clone();
    if input.reversed {
        after_reversal.reverse();
    }

    let mut start_mbp_removed = after_reversal.clone();
    remove_from_tracks(&mut start_mbp_removed, input.start_mbp, true);

    let mut end_mbp_removed = start_mbp_removed.clone();
    remove_from_tracks(&mut end_mbp_removed, input.end_mbp, false);

    let gap_difference = (input.subgrid_gap.resolved - input.parent_gap.resolved) / 2.0;
    let mut final_tracks = end_mbp_removed.clone();
    let last_index = final_tracks.len().saturating_sub(1);
    for (index, track) in final_tracks.iter_mut().enumerate() {
        if index > 0 {
            *track -= gap_difference;
        }
        if index != last_index {
            *track -= gap_difference;
        }
        *track = track.max(0.0);
    }

    Ok(SubgridTrackInheritanceReport {
        parent_span: input.parent_span,
        copied_parent_tracks,
        reversed: input.reversed,
        after_reversal,
        start_mbp_removed,
        end_mbp_removed,
        gap_difference,
        parent_gap: input.parent_gap,
        subgrid_gap: input.subgrid_gap,
        final_tracks,
    })
}

fn remove_from_tracks(tracks: &mut [f32], mut amount: f32, forwards: bool) {
    let mut indices = (0..tracks.len()).collect::<Vec<_>>();
    if !forwards {
        indices.reverse();
    }
    for index in indices {
        if amount <= 0.0 {
            break;
        }
        let removed = tracks[index].min(amount);
        tracks[index] -= removed;
        amount -= removed;
    }
}
```

- [ ] **Step 5: Export inherited-track API**

Modify `grid/mod.rs` subgrid exports:

```rust
pub use subgrid::{
    OracleGap, OracleGapReport, OracleGridError, SubgridEligibilityInput,
    SubgridEligibilityReport, SubgridIneligibleReason, SubgridTrackInheritanceInput,
    SubgridTrackInheritanceReport, TrackSpan, inherit_subgrid_tracks, subgrid_eligibility,
};
```

- [ ] **Step 6: Verify inherited-track behavior**

Run:

```bash
cargo test -p surgeist --test oracle oracle_subgrid_
```

Expected: all subgrid eligibility and inherited-track tests pass.

- [ ] **Step 7: Commit**

```bash
git add crates/surgeist/tests/support/oracle/grid/subgrid.rs crates/surgeist/tests/support/oracle/grid/mod.rs crates/surgeist/tests/oracle.rs
git commit -m "Add subgrid track inheritance oracle"
```

---

## Task 4: Implement Nested Subgrid Intrinsic Traversal

**Files:**
- Modify: `crates/surgeist/tests/support/oracle/grid/subgrid.rs`
- Modify: `crates/surgeist/tests/support/oracle/grid/mod.rs`
- Modify: `crates/surgeist/tests/oracle.rs`

- [ ] **Step 1: Add failing traversal tests**

Add these tests to `crates/surgeist/tests/oracle.rs`:

```rust
#[test]
fn oracle_subgrid_traversal_reports_direct_leaf_contribution() {
    let leaf = support::oracle::grid::SubgridChild::Leaf(
        support::oracle::grid::SubgridLeaf {
            id: "leaf",
            span_in_parent: support::oracle::grid::TrackSpan::new(1, 2),
            contribution: ItemContributionFacts {
                area: GridArea::new(1, 1, 1, 1),
                min_content: 20.0,
                max_content: 40.0,
                preferred: ContributionSize::Auto,
                min_size: ContributionSize::Auto,
                max_size: ContributionSize::Infinite,
                margin_before: 0.0,
                margin_after: 0.0,
                automatic_minimum_applies: true,
            },
        },
    );

    let report = support::oracle::grid::traverse_subgrid_intrinsic(
        support::oracle::grid::SubgridTraversalInput {
            ancestor_track_intrinsic_min_eligibility: vec![true],
            root_children: vec![leaf],
        },
    )
    .unwrap();

    assert_eq!(report.leaves.len(), 1);
    assert_eq!(report.leaves[0].id, "leaf");
    assert_eq!(report.leaves[0].ancestor_span, support::oracle::grid::TrackSpan::new(1, 2));
}

#[test]
fn oracle_subgrid_traversal_accumulates_intrinsic_edge_mbp() {
    let subgrid = support::oracle::grid::SubgridChild::Subgrid(
        support::oracle::grid::SubgridNode {
            id: "sub",
            axis: support::oracle::grid::SubgridAxisKind::Inherited,
            span_in_parent: support::oracle::grid::TrackSpan::new(1, 3),
            margins: support::oracle::grid::AxisEdges { start: 3.0, end: 4.0 },
            border: support::oracle::grid::AxisEdges { start: 5.0, end: 6.0 },
            padding: support::oracle::grid::AxisEdges { start: 7.0, end: 8.0 },
            parent_gap: support::oracle::grid::OracleGapReport::length(10.0),
            subgrid_gap: support::oracle::grid::OracleGapReport::length(10.0),
            children: vec![support::oracle::grid::SubgridChild::Leaf(
                support::oracle::grid::SubgridLeaf {
                    id: "leaf",
                    span_in_parent: support::oracle::grid::TrackSpan::new(1, 2),
                    contribution: ItemContributionFacts {
                        area: GridArea::new(1, 1, 1, 1),
                        min_content: 10.0,
                        max_content: 10.0,
                        preferred: ContributionSize::Auto,
                        min_size: ContributionSize::Auto,
                        max_size: ContributionSize::Infinite,
                        margin_before: 0.0,
                        margin_after: 0.0,
                        automatic_minimum_applies: true,
                    },
                },
            )],
        },
    );

    let report = support::oracle::grid::traverse_subgrid_intrinsic(
        support::oracle::grid::SubgridTraversalInput {
            ancestor_track_intrinsic_min_eligibility: vec![true, true],
            root_children: vec![subgrid],
        },
    )
    .unwrap();

    assert_eq!(report.edge_lower_bounds, vec![15.0, 18.0]);
    assert_eq!(report.leaves[0].accumulated_edge_adjustment, vec![15.0, 18.0]);
}

#[test]
fn oracle_subgrid_traversal_translates_leaf_span_to_ancestor_span() {
    let subgrid = support::oracle::grid::SubgridChild::Subgrid(
        support::oracle::grid::SubgridNode {
            id: "sub",
            axis: support::oracle::grid::SubgridAxisKind::Inherited,
            span_in_parent: support::oracle::grid::TrackSpan::new(2, 4),
            margins: support::oracle::grid::AxisEdges { start: 0.0, end: 0.0 },
            border: support::oracle::grid::AxisEdges { start: 0.0, end: 0.0 },
            padding: support::oracle::grid::AxisEdges { start: 0.0, end: 0.0 },
            parent_gap: support::oracle::grid::OracleGapReport::length(10.0),
            subgrid_gap: support::oracle::grid::OracleGapReport::length(10.0),
            children: vec![support::oracle::grid::SubgridChild::Leaf(
                support::oracle::grid::SubgridLeaf {
                    id: "leaf",
                    span_in_parent: support::oracle::grid::TrackSpan::new(2, 3),
                    contribution: ItemContributionFacts {
                        area: GridArea::new(1, 1, 1, 1),
                        min_content: 10.0,
                        max_content: 10.0,
                        preferred: ContributionSize::Auto,
                        min_size: ContributionSize::Auto,
                        max_size: ContributionSize::Infinite,
                        margin_before: 0.0,
                        margin_after: 0.0,
                        automatic_minimum_applies: true,
                    },
                },
            )],
        },
    );

    let report = support::oracle::grid::traverse_subgrid_intrinsic(
        support::oracle::grid::SubgridTraversalInput {
            ancestor_track_intrinsic_min_eligibility: vec![true, true, true],
            root_children: vec![subgrid],
        },
    )
    .unwrap();

    assert_eq!(report.leaves[0].ancestor_span, support::oracle::grid::TrackSpan::new(3, 4));
}

#[test]
fn oracle_subgrid_traversal_accumulates_gap_difference_edges() {
    let subgrid = support::oracle::grid::SubgridChild::Subgrid(
        support::oracle::grid::SubgridNode {
            id: "sub",
            axis: support::oracle::grid::SubgridAxisKind::Inherited,
            span_in_parent: support::oracle::grid::TrackSpan::new(1, 3),
            margins: support::oracle::grid::AxisEdges { start: 0.0, end: 0.0 },
            border: support::oracle::grid::AxisEdges { start: 0.0, end: 0.0 },
            padding: support::oracle::grid::AxisEdges { start: 0.0, end: 0.0 },
            parent_gap: support::oracle::grid::OracleGapReport::length(10.0),
            subgrid_gap: support::oracle::grid::OracleGapReport::length(20.0),
            children: vec![support::oracle::grid::SubgridChild::Leaf(
                support::oracle::grid::SubgridLeaf {
                    id: "leaf",
                    span_in_parent: support::oracle::grid::TrackSpan::new(2, 3),
                    contribution: ItemContributionFacts {
                        area: GridArea::new(1, 1, 1, 1),
                        min_content: 10.0,
                        max_content: 10.0,
                        preferred: ContributionSize::Auto,
                        min_size: ContributionSize::Auto,
                        max_size: ContributionSize::Infinite,
                        margin_before: 0.0,
                        margin_after: 0.0,
                        automatic_minimum_applies: true,
                    },
                },
            )],
        },
    );

    let report = support::oracle::grid::traverse_subgrid_intrinsic(
        support::oracle::grid::SubgridTraversalInput {
            ancestor_track_intrinsic_min_eligibility: vec![true, true],
            root_children: vec![subgrid],
        },
    )
    .unwrap();

    assert_eq!(report.leaves[0].accumulated_gap_adjustment, vec![0.0, 5.0]);
}

#[test]
fn oracle_subgrid_traversal_skips_edge_mbp_for_non_intrinsic_min_tracks() {
    let subgrid = support::oracle::grid::SubgridChild::Subgrid(
        support::oracle::grid::SubgridNode {
            id: "sub",
            axis: support::oracle::grid::SubgridAxisKind::Inherited,
            span_in_parent: support::oracle::grid::TrackSpan::new(1, 3),
            margins: support::oracle::grid::AxisEdges { start: 10.0, end: 10.0 },
            border: support::oracle::grid::AxisEdges { start: 0.0, end: 0.0 },
            padding: support::oracle::grid::AxisEdges { start: 0.0, end: 0.0 },
            parent_gap: support::oracle::grid::OracleGapReport::length(0.0),
            subgrid_gap: support::oracle::grid::OracleGapReport::length(0.0),
            children: vec![],
        },
    );

    let report = support::oracle::grid::traverse_subgrid_intrinsic(
        support::oracle::grid::SubgridTraversalInput {
            ancestor_track_intrinsic_min_eligibility: vec![false, false],
            root_children: vec![subgrid],
        },
    )
    .unwrap();

    assert_eq!(report.edge_lower_bounds, vec![0.0, 0.0]);
}

#[test]
fn oracle_subgrid_traversal_requires_intrinsic_min_facts_for_edge_placeholders() {
    let subgrid = support::oracle::grid::SubgridChild::Subgrid(
        support::oracle::grid::SubgridNode {
            id: "sub",
            axis: support::oracle::grid::SubgridAxisKind::Inherited,
            span_in_parent: support::oracle::grid::TrackSpan::new(1, 2),
            margins: support::oracle::grid::AxisEdges { start: 1.0, end: 1.0 },
            border: support::oracle::grid::AxisEdges { start: 0.0, end: 0.0 },
            padding: support::oracle::grid::AxisEdges { start: 0.0, end: 0.0 },
            parent_gap: support::oracle::grid::OracleGapReport::length(0.0),
            subgrid_gap: support::oracle::grid::OracleGapReport::length(0.0),
            children: vec![],
        },
    );

    let err = support::oracle::grid::traverse_subgrid_intrinsic(
        support::oracle::grid::SubgridTraversalInput {
            ancestor_track_intrinsic_min_eligibility: vec![],
            root_children: vec![subgrid],
        },
    )
    .unwrap_err();

    assert_eq!(err, support::oracle::grid::OracleGridError::MissingIntrinsicMinTrackFacts);
}

#[test]
fn oracle_subgrid_traversal_reports_standalone_axis_unsupported() {
    let subgrid = support::oracle::grid::SubgridChild::Subgrid(
        support::oracle::grid::SubgridNode {
            id: "standalone",
            axis: support::oracle::grid::SubgridAxisKind::Standalone,
            span_in_parent: support::oracle::grid::TrackSpan::new(1, 2),
            margins: support::oracle::grid::AxisEdges { start: 0.0, end: 0.0 },
            border: support::oracle::grid::AxisEdges { start: 0.0, end: 0.0 },
            padding: support::oracle::grid::AxisEdges { start: 0.0, end: 0.0 },
            parent_gap: support::oracle::grid::OracleGapReport::length(0.0),
            subgrid_gap: support::oracle::grid::OracleGapReport::length(0.0),
            children: vec![],
        },
    );

    let err = support::oracle::grid::traverse_subgrid_intrinsic(
        support::oracle::grid::SubgridTraversalInput {
            ancestor_track_intrinsic_min_eligibility: vec![true],
            root_children: vec![subgrid],
        },
    )
    .unwrap_err();

    assert_eq!(err, support::oracle::grid::OracleGridError::StandaloneSubgridTraversalUnsupported);
}

#[test]
fn oracle_subgrid_traversal_rejects_invalid_leaf_span() {
    let leaf = support::oracle::grid::SubgridChild::Leaf(
        support::oracle::grid::SubgridLeaf {
            id: "bad-leaf",
            span_in_parent: support::oracle::grid::TrackSpan::new(2, 2),
            contribution: ItemContributionFacts {
                area: GridArea::new(1, 1, 1, 1),
                min_content: 10.0,
                max_content: 10.0,
                preferred: ContributionSize::Auto,
                min_size: ContributionSize::Auto,
                max_size: ContributionSize::Infinite,
                margin_before: 0.0,
                margin_after: 0.0,
                automatic_minimum_applies: true,
            },
        },
    );

    let err = support::oracle::grid::traverse_subgrid_intrinsic(
        support::oracle::grid::SubgridTraversalInput {
            ancestor_track_intrinsic_min_eligibility: vec![true],
            root_children: vec![leaf],
        },
    )
    .unwrap_err();

    assert_eq!(err, support::oracle::grid::OracleGridError::SpanOutOfRange);
}
```

- [ ] **Step 2: Run focused failing tests**

Run:

```bash
cargo test -p surgeist --test oracle oracle_subgrid_traversal
```

Expected: compile failure for traversal types.

- [ ] **Step 3: Add traversal data types**

Add to `subgrid.rs`:

```rust
use super::contributions::ItemContributionFacts;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AxisEdges {
    pub start: f32,
    pub end: f32,
}

impl AxisEdges {
    #[must_use]
    pub const fn sum(self) -> f32 {
        self.start + self.end
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SubgridTraversalInput {
    pub ancestor_track_intrinsic_min_eligibility: Vec<bool>,
    pub root_children: Vec<SubgridChild>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SubgridChild {
    Subgrid(SubgridNode),
    Leaf(SubgridLeaf),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubgridAxisKind {
    Inherited,
    Standalone,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SubgridNode {
    pub id: &'static str,
    pub axis: SubgridAxisKind,
    pub span_in_parent: TrackSpan,
    pub margins: AxisEdges,
    pub border: AxisEdges,
    pub padding: AxisEdges,
    pub parent_gap: OracleGapReport,
    pub subgrid_gap: OracleGapReport,
    pub children: Vec<SubgridChild>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SubgridLeaf {
    pub id: &'static str,
    pub span_in_parent: TrackSpan,
    pub contribution: ItemContributionFacts,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SubgridLeafContribution {
    pub id: &'static str,
    pub ancestor_span: TrackSpan,
    pub accumulated_edge_adjustment: Vec<f32>,
    pub accumulated_gap_adjustment: Vec<f32>,
    pub contribution: ItemContributionFacts,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SubgridTraversalReport {
    pub edge_lower_bounds: Vec<f32>,
    pub leaves: Vec<SubgridLeafContribution>,
}
```

- [ ] **Step 4: Implement traversal**

Add to `subgrid.rs`:

```rust
pub fn traverse_subgrid_intrinsic(
    input: SubgridTraversalInput,
) -> Result<SubgridTraversalReport, OracleGridError> {
    let mut edge_lower_bounds = vec![0.0; input.ancestor_track_intrinsic_min_eligibility.len()];
    let mut leaves = Vec::new();
    let mut stack = input
        .root_children
        .into_iter()
        .rev()
        .map(|child| {
            (
                child,
                TraversalContext {
                    ancestor_start_line: 1,
                    accumulated_edge_adjustment: vec![0.0; edge_lower_bounds.len()],
                    accumulated_gap_adjustment: vec![0.0; edge_lower_bounds.len()],
                },
            )
        })
        .collect::<Vec<_>>();

    while let Some((child, context)) = stack.pop() {
        match child {
            SubgridChild::Leaf(leaf) => {
                leaf.span_in_parent.checked_len()?;
                let ancestor_span = translate_span_to_ancestor(context.ancestor_start_line, leaf.span_in_parent);
                leaves.push(SubgridLeafContribution {
                    id: leaf.id,
                    ancestor_span,
                    accumulated_edge_adjustment: context.accumulated_edge_adjustment,
                    accumulated_gap_adjustment: context.accumulated_gap_adjustment,
                    contribution: leaf.contribution,
                });
            }
            SubgridChild::Subgrid(subgrid) => {
                apply_subgrid_edge_placeholders(
                    &input.ancestor_track_intrinsic_min_eligibility,
                    &mut edge_lower_bounds,
                    &mut stack,
                    subgrid,
                    context,
                )?;
            }
        }
    }

    Ok(SubgridTraversalReport {
        edge_lower_bounds,
        leaves,
    })
}

fn apply_subgrid_edge_placeholders(
    intrinsic_min: &[bool],
    edge_lower_bounds: &mut [f32],
    stack: &mut Vec<(SubgridChild, TraversalContext)>,
    subgrid: SubgridNode,
    mut context: TraversalContext,
) -> Result<(), OracleGridError> {
    if subgrid.axis == SubgridAxisKind::Standalone {
        return Err(OracleGridError::StandaloneSubgridTraversalUnsupported);
    }
    if subgrid.span_in_parent.start == 0 || subgrid.span_in_parent.end <= subgrid.span_in_parent.start
    {
        return Err(OracleGridError::SpanOutOfRange);
    }

    let start_index = subgrid.span_in_parent.start - 1;
    let end_index = subgrid.span_in_parent.end - 2;
    if end_index >= intrinsic_min.len()
        || end_index >= edge_lower_bounds.len()
        || context.accumulated_edge_adjustment.len() != edge_lower_bounds.len()
        || context.accumulated_gap_adjustment.len() != edge_lower_bounds.len()
    {
        return Err(OracleGridError::MissingIntrinsicMinTrackFacts);
    }

    let start_edge = subgrid.margins.start + subgrid.border.start + subgrid.padding.start;
    let end_edge = subgrid.margins.end + subgrid.border.end + subgrid.padding.end;

    if intrinsic_min[start_index] {
        context.accumulated_edge_adjustment[start_index] += start_edge;
        edge_lower_bounds[start_index] = edge_lower_bounds[start_index].max(context.accumulated_edge_adjustment[start_index]);
    }
    if intrinsic_min[end_index] {
        context.accumulated_edge_adjustment[end_index] += end_edge;
        edge_lower_bounds[end_index] = edge_lower_bounds[end_index].max(context.accumulated_edge_adjustment[end_index]);
    }

    let gap_difference = (subgrid.subgrid_gap.resolved - subgrid.parent_gap.resolved) / 2.0;
    if start_index > 0 {
        context.accumulated_gap_adjustment[start_index] += gap_difference;
    }
    if end_index < context.accumulated_gap_adjustment.len() {
        context.accumulated_gap_adjustment[end_index] += gap_difference;
    }

    let child_context = TraversalContext {
        ancestor_start_line: context.ancestor_start_line + subgrid.span_in_parent.start - 1,
        accumulated_edge_adjustment: context.accumulated_edge_adjustment,
        accumulated_gap_adjustment: context.accumulated_gap_adjustment,
    };

    for child in subgrid.children.into_iter().rev() {
        stack.push((child, child_context.clone()));
    }

    Ok(())
}

#[derive(Clone, Debug, PartialEq)]
struct TraversalContext {
    ancestor_start_line: usize,
    accumulated_edge_adjustment: Vec<f32>,
    accumulated_gap_adjustment: Vec<f32>,
}

fn translate_span_to_ancestor(ancestor_start_line: usize, local_span: TrackSpan) -> TrackSpan {
    TrackSpan::new(
        ancestor_start_line + local_span.start - 1,
        ancestor_start_line + local_span.end - 1,
    )
}
```

- [ ] **Step 5: Export traversal API**

Update the `subgrid` export group in `grid/mod.rs`:

```rust
pub use subgrid::{
    AxisEdges, OracleGap, OracleGapReport, OracleGridError, SubgridChild,
    SubgridEligibilityInput, SubgridEligibilityReport, SubgridIneligibleReason, SubgridLeaf,
    SubgridAxisKind, SubgridLeafContribution, SubgridNode, SubgridTrackInheritanceInput,
    SubgridTrackInheritanceReport, SubgridTraversalInput, SubgridTraversalReport, TrackSpan,
    inherit_subgrid_tracks, subgrid_eligibility, traverse_subgrid_intrinsic,
};
```

- [ ] **Step 6: Verify traversal**

Run:

```bash
cargo test -p surgeist --test oracle oracle_subgrid_traversal
```

Expected: all traversal tests pass.

- [ ] **Step 7: Commit**

```bash
git add crates/surgeist/tests/support/oracle/grid/subgrid.rs crates/surgeist/tests/support/oracle/grid/mod.rs crates/surgeist/tests/oracle.rs
git commit -m "Add nested subgrid oracle traversal"
```

---

## Task 5: Add Grid-Lanes Axis Derivation And Placement

**Files:**
- Create: `crates/surgeist/tests/support/oracle/grid/lanes.rs`
- Modify: `crates/surgeist/tests/support/oracle/grid/mod.rs`
- Modify: `crates/surgeist/tests/oracle.rs`

- [ ] **Step 1: Add failing lane placement tests**

Add these tests to `crates/surgeist/tests/oracle.rs`:

```rust
#[test]
fn oracle_lanes_row_auto_flow_makes_rows_the_lane_axis() {
    assert_eq!(
        support::oracle::grid::lane_axis(support::oracle::grid::LaneAutoFlow::Row),
        support::oracle::grid::GridAxis::Row
    );
    assert_eq!(
        support::oracle::grid::grid_axis_for_lanes(support::oracle::grid::LaneAutoFlow::Row),
        support::oracle::grid::GridAxis::Column
    );
}

#[test]
fn oracle_lanes_place_definite_and_indefinite_items_with_fixed_tolerance() {
    let report = support::oracle::grid::place_lanes(
        support::oracle::grid::LanePlacementInput {
            grid_axis_tracks: 3,
            auto_flow: support::oracle::grid::LaneAutoFlow::Row,
            lane_gap: 10.0,
            tolerance: support::oracle::grid::LaneFlowTolerance::Fixed(0.0),
            tolerance_basis: 0.0,
            items: vec![
                support::oracle::grid::LaneItemInput::definite("a", 1, 2, 40.0),
                support::oracle::grid::LaneItemInput::auto("b", 1, 20.0),
                support::oracle::grid::LaneItemInput::auto("c", 2, 30.0),
            ],
        },
    )
    .unwrap();

    assert_eq!(report.item_offsets[0].offset, 0.0);
    assert_eq!(report.item_offsets[1].offset, 0.0);
    assert_eq!(report.item_offsets[2].offset, 50.0);
    assert_eq!(report.content_size, 80.0);
}

#[test]
fn oracle_lanes_finite_search_does_not_wrap_candidate_span() {
    let report = support::oracle::grid::place_lanes(
        support::oracle::grid::LanePlacementInput {
            grid_axis_tracks: 3,
            auto_flow: support::oracle::grid::LaneAutoFlow::Row,
            lane_gap: 0.0,
            tolerance: support::oracle::grid::LaneFlowTolerance::Fixed(0.0),
            tolerance_basis: 0.0,
            items: vec![
                support::oracle::grid::LaneItemInput::auto("a", 2, 10.0),
                support::oracle::grid::LaneItemInput::auto("b", 2, 10.0),
            ],
        },
    )
    .unwrap();

    assert!(report.item_offsets.iter().all(|item| item.grid_axis_start + item.grid_axis_span <= 4));
}

#[test]
fn oracle_lanes_reject_definite_item_that_exceeds_grid_axis() {
    let err = support::oracle::grid::place_lanes(
        support::oracle::grid::LanePlacementInput {
            grid_axis_tracks: 3,
            auto_flow: support::oracle::grid::LaneAutoFlow::Row,
            lane_gap: 0.0,
            tolerance: support::oracle::grid::LaneFlowTolerance::Fixed(0.0),
            tolerance_basis: 0.0,
            items: vec![support::oracle::grid::LaneItemInput::definite("a", 3, 2, 10.0)],
        },
    )
    .unwrap_err();

    assert_eq!(err, support::oracle::grid::OracleGridError::SpanOutOfRange);
}

#[test]
fn oracle_lanes_infinite_tolerance_uses_round_robin_cursor() {
    let report = support::oracle::grid::place_lanes(
        support::oracle::grid::LanePlacementInput {
            grid_axis_tracks: 2,
            auto_flow: support::oracle::grid::LaneAutoFlow::Column,
            lane_gap: 0.0,
            tolerance: support::oracle::grid::LaneFlowTolerance::Infinite,
            tolerance_basis: 0.0,
            items: vec![
                support::oracle::grid::LaneItemInput::auto("a", 1, 10.0),
                support::oracle::grid::LaneItemInput::auto("b", 1, 10.0),
                support::oracle::grid::LaneItemInput::auto("c", 1, 10.0),
            ],
        },
    )
    .unwrap();

    assert_eq!(
        report
            .item_offsets
            .iter()
            .map(|item| item.grid_axis_start)
            .collect::<Vec<_>>(),
        vec![1, 2, 1]
    );
}

#[test]
fn oracle_lanes_percentage_tolerance_resolves_against_basis() {
    let report = support::oracle::grid::place_lanes(
        support::oracle::grid::LanePlacementInput {
            grid_axis_tracks: 2,
            auto_flow: support::oracle::grid::LaneAutoFlow::Row,
            lane_gap: 0.0,
            tolerance: support::oracle::grid::LaneFlowTolerance::Percent(0.25),
            tolerance_basis: 40.0,
            items: vec![
                support::oracle::grid::LaneItemInput::definite("a", 1, 1, 10.0),
                support::oracle::grid::LaneItemInput::auto("b", 1, 10.0),
            ],
        },
    )
    .unwrap();

    assert_eq!(report.item_offsets[1].grid_axis_start, 2);
}

#[test]
fn oracle_lanes_finite_tolerance_chooses_first_candidate_within_tolerance() {
    let report = support::oracle::grid::place_lanes(
        support::oracle::grid::LanePlacementInput {
            grid_axis_tracks: 3,
            auto_flow: support::oracle::grid::LaneAutoFlow::Row,
            lane_gap: 0.0,
            tolerance: support::oracle::grid::LaneFlowTolerance::Fixed(10.0),
            tolerance_basis: 0.0,
            items: vec![
                support::oracle::grid::LaneItemInput::definite("a", 1, 1, 10.0),
                support::oracle::grid::LaneItemInput::definite("b", 2, 1, 20.0),
                support::oracle::grid::LaneItemInput::auto("c", 1, 10.0),
            ],
        },
    )
    .unwrap();

    assert_eq!(report.item_offsets[2].grid_axis_start, 3);
}
```

- [ ] **Step 2: Run focused failing tests**

Run:

```bash
cargo test -p surgeist --test oracle oracle_lanes_
```

Expected: compile failure for `lanes.rs` types.

- [ ] **Step 3: Add lanes data types**

Create `crates/surgeist/tests/support/oracle/grid/lanes.rs`:

```rust
use super::axis::opposite_axis;
use super::placement::GridAxis;
use super::subgrid::OracleGridError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LaneAutoFlow {
    Row,
    Column,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LaneFlowTolerance {
    Normal { font_size: f32 },
    Fixed(f32),
    Percent(f32),
    Infinite,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LanePlacementInput {
    pub grid_axis_tracks: usize,
    pub auto_flow: LaneAutoFlow,
    pub lane_gap: f32,
    pub tolerance: LaneFlowTolerance,
    pub tolerance_basis: f32,
    pub items: Vec<LaneItemInput>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LaneItemInput {
    pub id: &'static str,
    pub grid_axis_span: usize,
    pub definite_grid_axis_start: Option<usize>,
    pub lane_axis_margin_box: f32,
}

impl LaneItemInput {
    #[must_use]
    pub const fn definite(
        id: &'static str,
        grid_axis_start: usize,
        grid_axis_span: usize,
        lane_axis_margin_box: f32,
    ) -> Self {
        Self {
            id,
            grid_axis_span,
            definite_grid_axis_start: Some(grid_axis_start),
            lane_axis_margin_box,
        }
    }

    #[must_use]
    pub const fn auto(id: &'static str, grid_axis_span: usize, lane_axis_margin_box: f32) -> Self {
        Self {
            id,
            grid_axis_span,
            definite_grid_axis_start: None,
            lane_axis_margin_box,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LaneItemOffset {
    pub id: &'static str,
    pub grid_axis_start: usize,
    pub grid_axis_span: usize,
    pub offset: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LanePlacementReport {
    pub lane_axis: GridAxis,
    pub grid_axis: GridAxis,
    pub item_offsets: Vec<LaneItemOffset>,
    pub running_positions_after_each_item: Vec<Vec<f32>>,
    pub content_size: f32,
    pub final_cursor: usize,
}

#[must_use]
pub const fn lane_axis(auto_flow: LaneAutoFlow) -> GridAxis {
    match auto_flow {
        LaneAutoFlow::Row => GridAxis::Row,
        LaneAutoFlow::Column => GridAxis::Column,
    }
}

#[must_use]
pub const fn grid_axis_for_lanes(auto_flow: LaneAutoFlow) -> GridAxis {
    opposite_axis(lane_axis(auto_flow))
}
```

- [ ] **Step 4: Implement lane placement**

Add to `lanes.rs`:

```rust
pub fn place_lanes(input: LanePlacementInput) -> Result<LanePlacementReport, OracleGridError> {
    if input.grid_axis_tracks == 0 {
        return Err(OracleGridError::EmptyTrackList);
    }

    let mut running = vec![0.0; input.grid_axis_tracks];
    let mut item_offsets = Vec::new();
    let mut running_positions_after_each_item = Vec::new();
    let mut cursor = 0usize;
    let tolerance = resolve_tolerance(input.tolerance, input.tolerance_basis);
    let mut content_size: f32 = 0.0;

    for item in input.items {
        let (start_zero, span) = match item.definite_grid_axis_start {
            Some(start_line) => {
                if start_line == 0 || item.grid_axis_span == 0 {
                    return Err(OracleGridError::SpanOutOfRange);
                }
                let start_zero = start_line - 1;
                if start_zero + item.grid_axis_span > input.grid_axis_tracks {
                    return Err(OracleGridError::SpanOutOfRange);
                }
                (start_zero, item.grid_axis_span)
            }
            None => {
                let span = item.grid_axis_span.clamp(1, input.grid_axis_tracks);
                let start_zero = if matches!(input.tolerance, LaneFlowTolerance::Infinite) {
                    infinite_candidate_start(cursor, span, input.grid_axis_tracks)
                } else {
                    finite_candidate_start(&running, cursor, span, tolerance)
                };
                (start_zero, span)
            }
        };
        if start_zero + span > input.grid_axis_tracks {
            return Err(OracleGridError::SpanOutOfRange);
        }

        let previous = running[start_zero..start_zero + span]
            .iter()
            .copied()
            .fold(0.0, f32::max);
        let new_position = previous + item.lane_axis_margin_box + input.lane_gap;
        content_size = content_size.max(new_position - input.lane_gap);
        for position in &mut running[start_zero..start_zero + span] {
            *position = new_position;
        }

        item_offsets.push(LaneItemOffset {
            id: item.id,
            grid_axis_start: start_zero + 1,
            grid_axis_span: span,
            offset: previous,
        });
        running_positions_after_each_item.push(running.clone());
        cursor = (start_zero + span) % input.grid_axis_tracks;
    }

    Ok(LanePlacementReport {
        lane_axis: lane_axis(input.auto_flow),
        grid_axis: grid_axis_for_lanes(input.auto_flow),
        item_offsets,
        running_positions_after_each_item,
        content_size,
        final_cursor: cursor,
    })
}

fn resolve_tolerance(tolerance: LaneFlowTolerance, basis: f32) -> f32 {
    match tolerance {
        LaneFlowTolerance::Normal { font_size } => font_size,
        LaneFlowTolerance::Fixed(value) => value,
        LaneFlowTolerance::Percent(factor) => factor * basis,
        LaneFlowTolerance::Infinite => f32::INFINITY,
    }
}

fn infinite_candidate_start(cursor: usize, span: usize, track_count: usize) -> usize {
    if cursor + span > track_count {
        0
    } else {
        cursor
    }
}

fn finite_candidate_start(running: &[f32], cursor: usize, span: usize, tolerance: f32) -> usize {
    let track_count = running.len();
    let max_start = track_count + 1 - span;
    let shifted_cursor = if cursor > max_start { 0 } else { cursor };
    let absolute_shortest = (0..max_start)
        .map(|start| max_running_position(running, start, span))
        .fold(f32::INFINITY, f32::min);

    for offset in 0..max_start {
        let start = (shifted_cursor + offset) % max_start;
        if max_running_position(running, start, span) <= absolute_shortest + tolerance {
            return start;
        }
    }

    0
}

fn max_running_position(running: &[f32], start: usize, span: usize) -> f32 {
    running[start..start + span]
        .iter()
        .copied()
        .fold(0.0, f32::max)
}
```

- [ ] **Step 5: Export lanes API**

Modify `grid/mod.rs`:

```rust
pub mod lanes;

#[allow(unused_imports)]
pub use lanes::{
    LaneAutoFlow, LaneFlowTolerance, LaneItemInput, LaneItemOffset, LanePlacementInput,
    LanePlacementReport, grid_axis_for_lanes, lane_axis, place_lanes,
};
```

- [ ] **Step 6: Verify lane placement**

Run:

```bash
cargo test -p surgeist --test oracle oracle_lanes_
```

Expected: all lane axis and placement tests pass.

- [ ] **Step 7: Commit**

```bash
git add crates/surgeist/tests/support/oracle/grid/lanes.rs crates/surgeist/tests/support/oracle/grid/mod.rs crates/surgeist/tests/oracle.rs
git commit -m "Add grid-lanes oracle placement"
```

---

## Task 6: Add Grid-Lanes Intrinsic Sizing Group Reports

**Files:**
- Modify: `crates/surgeist/tests/support/oracle/grid/lanes.rs`
- Modify: `crates/surgeist/tests/support/oracle/grid/mod.rs`
- Modify: `crates/surgeist/tests/oracle.rs`

- [ ] **Step 1: Add failing intrinsic grouping tests**

Add these tests to `crates/surgeist/tests/oracle.rs`:

```rust
#[test]
fn oracle_lanes_intrinsic_keeps_definite_items_by_span() {
    let report = support::oracle::grid::lane_intrinsic_sizing(
        support::oracle::grid::LaneIntrinsicSizingInput {
            axis: support::oracle::grid::GridAxis::Column,
            available: Some(200.0),
            gap: 10.0,
            tracks: vec![GridTrack::auto(), GridTrack::auto()],
            content_sized_tracks: vec![0, 1],
            items: vec![support::oracle::grid::LaneIntrinsicItem::definite(
                "a",
                support::oracle::grid::TrackSpan::new(1, 2),
                ItemContributionFacts {
                    area: GridArea::new(1, 1, 1, 1),
                    min_content: 20.0,
                    max_content: 50.0,
                    preferred: ContributionSize::Auto,
                    min_size: ContributionSize::Auto,
                    max_size: ContributionSize::Infinite,
                    margin_before: 0.0,
                    margin_after: 0.0,
                    automatic_minimum_applies: true,
                },
            )],
        },
    )
    .unwrap();

    assert_eq!(report.definite_items.len(), 1);
    assert!(report.indefinite_groups.is_empty());
    assert_eq!(report.definite_items[0].contribution.area, GridArea::new(1, 1, 1, 1));
}

#[test]
fn oracle_lanes_intrinsic_rewrites_definite_item_area_from_span() {
    let report = support::oracle::grid::lane_intrinsic_sizing(
        support::oracle::grid::LaneIntrinsicSizingInput {
            axis: support::oracle::grid::GridAxis::Column,
            available: Some(200.0),
            gap: 10.0,
            tracks: vec![GridTrack::auto(), GridTrack::auto()],
            content_sized_tracks: vec![0, 1],
            items: vec![support::oracle::grid::LaneIntrinsicItem::definite(
                "a",
                support::oracle::grid::TrackSpan::new(2, 3),
                ItemContributionFacts {
                    area: GridArea::new(1, 1, 1, 1),
                    min_content: 20.0,
                    max_content: 50.0,
                    preferred: ContributionSize::Auto,
                    min_size: ContributionSize::Auto,
                    max_size: ContributionSize::Infinite,
                    margin_before: 0.0,
                    margin_after: 0.0,
                    automatic_minimum_applies: true,
                },
            )],
        },
    )
    .unwrap();

    assert_eq!(report.definite_items[0].contribution.area, GridArea::new(2, 1, 1, 1));
}

#[test]
fn oracle_lanes_intrinsic_rewrites_row_axis_areas_from_spans() {
    let report = support::oracle::grid::lane_intrinsic_sizing(
        support::oracle::grid::LaneIntrinsicSizingInput {
            axis: support::oracle::grid::GridAxis::Row,
            available: Some(200.0),
            gap: 10.0,
            tracks: vec![GridTrack::auto(), GridTrack::auto()],
            content_sized_tracks: vec![0, 1],
            items: vec![support::oracle::grid::LaneIntrinsicItem::definite(
                "a",
                support::oracle::grid::TrackSpan::new(2, 3),
                ItemContributionFacts {
                    area: GridArea::new(1, 1, 1, 1),
                    min_content: 20.0,
                    max_content: 50.0,
                    preferred: ContributionSize::Auto,
                    min_size: ContributionSize::Auto,
                    max_size: ContributionSize::Infinite,
                    margin_before: 0.0,
                    margin_after: 0.0,
                    automatic_minimum_applies: true,
                },
            )],
        },
    )
    .unwrap();

    assert_eq!(report.definite_items[0].contribution.area, GridArea::new(1, 2, 1, 1));
}

#[test]
fn oracle_lanes_intrinsic_groups_indefinite_items_by_span_length() {
    let facts = ItemContributionFacts {
        area: GridArea::new(1, 1, 1, 1),
        min_content: 20.0,
        max_content: 50.0,
        preferred: ContributionSize::Auto,
        min_size: ContributionSize::Auto,
        max_size: ContributionSize::Infinite,
        margin_before: 0.0,
        margin_after: 0.0,
        automatic_minimum_applies: true,
    };
    let report = support::oracle::grid::lane_intrinsic_sizing(
        support::oracle::grid::LaneIntrinsicSizingInput {
            axis: support::oracle::grid::GridAxis::Column,
            available: Some(300.0),
            gap: 10.0,
            tracks: vec![GridTrack::auto(), GridTrack::auto(), GridTrack::auto()],
            content_sized_tracks: vec![0, 1, 2],
            items: vec![
                support::oracle::grid::LaneIntrinsicItem::indefinite("a", 2, facts),
                support::oracle::grid::LaneIntrinsicItem::indefinite("b", 2, ItemContributionFacts {
                    min_content: 30.0,
                    max_content: 60.0,
                    ..facts
                }),
            ],
        },
    )
    .unwrap();

    assert_eq!(report.indefinite_groups.len(), 1);
    assert_eq!(report.indefinite_groups[0].span, 2);
    assert_eq!(report.indefinite_groups[0].max_min_content, 30.0);
    assert_eq!(report.indefinite_groups[0].max_max_content, 60.0);
    assert_eq!(report.converted_indefinite_items.len(), 2);
    assert_eq!(report.final_track_report.final_tracks.len(), 3);
}

#[test]
fn oracle_lanes_intrinsic_reports_nested_indefinite_subgrid_unsupported() {
    let facts = ItemContributionFacts {
        area: GridArea::new(1, 1, 1, 1),
        min_content: 20.0,
        max_content: 50.0,
        preferred: ContributionSize::Auto,
        min_size: ContributionSize::Auto,
        max_size: ContributionSize::Infinite,
        margin_before: 0.0,
        margin_after: 0.0,
        automatic_minimum_applies: true,
    };
    let err = support::oracle::grid::lane_intrinsic_sizing(
        support::oracle::grid::LaneIntrinsicSizingInput {
            axis: support::oracle::grid::GridAxis::Column,
            available: Some(300.0),
            gap: 10.0,
            tracks: vec![GridTrack::auto(), GridTrack::auto(), GridTrack::auto()],
            content_sized_tracks: vec![0, 1, 2],
            items: vec![support::oracle::grid::LaneIntrinsicItem::nested_indefinite_subgrid(
                "subgrid-child",
                2,
                facts,
            )],
        },
    )
    .unwrap_err();

    assert_eq!(
        err,
        support::oracle::grid::OracleGridError::NestedGridLanesSubgridIndefiniteUnsupported
    );
}

#[test]
fn oracle_lanes_intrinsic_rejects_invalid_definite_span() {
    let facts = ItemContributionFacts {
        area: GridArea::new(1, 1, 1, 1),
        min_content: 20.0,
        max_content: 50.0,
        preferred: ContributionSize::Auto,
        min_size: ContributionSize::Auto,
        max_size: ContributionSize::Infinite,
        margin_before: 0.0,
        margin_after: 0.0,
        automatic_minimum_applies: true,
    };
    let err = support::oracle::grid::lane_intrinsic_sizing(
        support::oracle::grid::LaneIntrinsicSizingInput {
            axis: support::oracle::grid::GridAxis::Column,
            available: Some(300.0),
            gap: 10.0,
            tracks: vec![GridTrack::auto()],
            content_sized_tracks: vec![0],
            items: vec![support::oracle::grid::LaneIntrinsicItem::definite(
                "bad",
                support::oracle::grid::TrackSpan::new(2, 2),
                facts,
            )],
        },
    )
    .unwrap_err();

    assert_eq!(err, support::oracle::grid::OracleGridError::SpanOutOfRange);
}
```

- [ ] **Step 2: Run focused failing tests**

Run:

```bash
cargo test -p surgeist --test oracle oracle_lanes_intrinsic
```

Expected: compile failure for intrinsic sizing types.

- [ ] **Step 3: Add intrinsic sizing report types**

Add to `lanes.rs`:

```rust
use super::contributions::ItemContributionFacts;
use super::tracks::{GridTrack, TrackSizingReport, TrackSizingSlice};
use super::subgrid::TrackSpan;

#[derive(Clone, Debug, PartialEq)]
pub struct LaneIntrinsicSizingInput {
    pub axis: GridAxis,
    pub available: Option<f32>,
    pub gap: f32,
    pub tracks: Vec<GridTrack>,
    pub content_sized_tracks: Vec<usize>,
    pub items: Vec<LaneIntrinsicItem>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LaneIntrinsicItem {
    pub id: &'static str,
    pub span: usize,
    pub definite_span: Option<TrackSpan>,
    pub contribution: ItemContributionFacts,
    pub nested_indefinite_subgrid: bool,
}

impl LaneIntrinsicItem {
    #[must_use]
    pub const fn definite(
        id: &'static str,
        span: TrackSpan,
        contribution: ItemContributionFacts,
    ) -> Self {
        Self {
            id,
            span: 0,
            definite_span: Some(span),
            contribution,
            nested_indefinite_subgrid: false,
        }
    }

    #[must_use]
    pub const fn indefinite(
        id: &'static str,
        span: usize,
        contribution: ItemContributionFacts,
    ) -> Self {
        Self {
            id,
            span,
            definite_span: None,
            contribution,
            nested_indefinite_subgrid: false,
        }
    }

    #[must_use]
    pub const fn nested_indefinite_subgrid(
        id: &'static str,
        span: usize,
        contribution: ItemContributionFacts,
    ) -> Self {
        Self {
            id,
            span,
            definite_span: None,
            contribution,
            nested_indefinite_subgrid: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DefiniteLaneIntrinsicItem {
    pub id: &'static str,
    pub span: TrackSpan,
    pub contribution: ItemContributionFacts,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IndefiniteLaneContributionGroup {
    pub span: usize,
    pub max_min_content: f32,
    pub max_max_content: f32,
    pub item_ids: Vec<&'static str>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LaneIntrinsicSizingReport {
    pub definite_items: Vec<DefiniteLaneIntrinsicItem>,
    pub indefinite_groups: Vec<IndefiniteLaneContributionGroup>,
    pub converted_indefinite_items: Vec<DefiniteLaneIntrinsicItem>,
    pub final_track_report: TrackSizingReport,
}
```

- [ ] **Step 4: Implement grouping**

Add to `lanes.rs`:

```rust
pub fn lane_intrinsic_sizing(
    input: LaneIntrinsicSizingInput,
) -> Result<LaneIntrinsicSizingReport, OracleGridError> {
    if input.content_sized_tracks.is_empty() {
        return Err(OracleGridError::EmptyTrackList);
    }
    if input.tracks.is_empty() {
        return Err(OracleGridError::EmptyTrackList);
    }

    let mut definite_items = Vec::new();
    let mut indefinite_groups: Vec<IndefiniteLaneContributionGroup> = Vec::new();

    for item in &input.items {
        if item.nested_indefinite_subgrid {
            return Err(OracleGridError::NestedGridLanesSubgridIndefiniteUnsupported);
        }
        if let Some(span) = item.definite_span {
            span.checked_len()?;
            let contribution = contribution_with_span_area(input.axis, span, item.contribution);
            definite_items.push(DefiniteLaneIntrinsicItem {
                id: item.id,
                span,
                contribution,
            });
            continue;
        }

        let span = item.span.max(1);
        let contributions = item.contribution.contributions();
        if let Some(group) = indefinite_groups.iter_mut().find(|group| group.span == span) {
            group.max_min_content = group.max_min_content.max(contributions.min_content);
            group.max_max_content = group.max_max_content.max(contributions.max_content);
            group.item_ids.push(item.id);
        } else {
            indefinite_groups.push(IndefiniteLaneContributionGroup {
                span,
                max_min_content: contributions.min_content,
                max_max_content: contributions.max_content,
                item_ids: vec![item.id],
            });
        }
    }

    let mut converted_indefinite_items = Vec::new();
    for group in &indefinite_groups {
        for start_index in candidate_content_starts(&input.content_sized_tracks, input.tracks.len(), group.span) {
            let span = TrackSpan::new(start_index + 1, start_index + 1 + group.span);
            let contribution = contribution_with_span_area(
                input.axis,
                span,
                ItemContributionFacts {
                    area: super::placement::GridArea::new(1, 1, 1, 1),
                    min_content: group.max_min_content,
                    max_content: group.max_max_content,
                    preferred: super::contributions::ContributionSize::Auto,
                    min_size: super::contributions::ContributionSize::Auto,
                    max_size: super::contributions::ContributionSize::Infinite,
                    margin_before: 0.0,
                    margin_after: 0.0,
                    automatic_minimum_applies: true,
                },
            );
            converted_indefinite_items.push(DefiniteLaneIntrinsicItem {
                id: "indefinite-group",
                span,
                contribution,
            });
        }
    }

    let mut track_slice = match (input.axis, input.available) {
        (GridAxis::Column, Some(available)) => TrackSizingSlice::definite_columns(available, input.gap),
        (GridAxis::Row, Some(available)) => TrackSizingSlice::definite_rows(available, input.gap),
        (GridAxis::Column, None) => TrackSizingSlice::indefinite_columns(input.gap),
        (GridAxis::Row, None) => TrackSizingSlice::indefinite_rows(input.gap),
    };
    for track in input.tracks {
        track_slice = track_slice.track(track);
    }
    for item in definite_items.iter().chain(converted_indefinite_items.iter()) {
        track_slice = track_slice.item(item.contribution);
    }
    let final_track_report = track_slice
        .try_solve()
        .map_err(|_| OracleGridError::SpanOutOfRange)?;

    Ok(LaneIntrinsicSizingReport {
        definite_items,
        indefinite_groups,
        converted_indefinite_items,
        final_track_report,
    })
}

fn candidate_content_starts(content_sized_tracks: &[usize], track_count: usize, span: usize) -> Vec<usize> {
    let span = span.max(1).min(track_count);
    content_sized_tracks
        .iter()
        .copied()
        .filter(|start| start + span <= track_count)
        .collect()
}

fn contribution_with_span_area(
    axis: GridAxis,
    span: TrackSpan,
    mut contribution: ItemContributionFacts,
) -> ItemContributionFacts {
    let span_len = span.checked_len().expect("span already validated");
    contribution.area = match axis {
        GridAxis::Column => super::placement::GridArea::new(span.start, 1, span_len, 1),
        GridAxis::Row => super::placement::GridArea::new(1, span.start, 1, span_len),
    };
    contribution
}
```

- [ ] **Step 5: Export intrinsic sizing API**

Update `grid/mod.rs` lanes exports:

```rust
pub use lanes::{
    DefiniteLaneIntrinsicItem, IndefiniteLaneContributionGroup, LaneAutoFlow,
    LaneFlowTolerance, LaneIntrinsicItem, LaneIntrinsicSizingInput, LaneIntrinsicSizingReport,
    LaneItemInput, LaneItemOffset, LanePlacementInput, LanePlacementReport, grid_axis_for_lanes,
    lane_axis, lane_intrinsic_sizing, place_lanes,
};
```

- [ ] **Step 6: Verify intrinsic grouping**

Run:

```bash
cargo test -p surgeist --test oracle oracle_lanes_intrinsic
```

Expected: all lane intrinsic grouping tests pass.

- [ ] **Step 7: Commit**

```bash
git add crates/surgeist/tests/support/oracle/grid/lanes.rs crates/surgeist/tests/support/oracle/grid/mod.rs crates/surgeist/tests/oracle.rs
git commit -m "Add grid-lanes intrinsic oracle reports"
```

---

## Task 7: Add Scenario Composition Guardrails

**Files:**
- Modify: `crates/surgeist/tests/support/oracle/grid/scenario.rs`
- Modify: `crates/surgeist/tests/support/oracle/grid/mod.rs`
- Modify: `crates/surgeist/tests/oracle.rs`

- [ ] **Step 1: Add failing scenario guardrail tests**

Add these tests to `crates/surgeist/tests/oracle.rs`:

```rust
#[test]
fn oracle_scenario_composes_subgrid_rect_from_explicit_tracks_and_offsets() {
    let report = support::oracle::grid::compose_subgrid_item_rect(
        support::oracle::grid::SubgridItemRectInput {
            inherited_axis: support::oracle::grid::GridAxis::Column,
            inherited_axis_offset: 20.0,
            standalone_axis_offset: 5.0,
            inherited_axis_size: 80.0,
            standalone_axis_size: 30.0,
            container_mbp_offset: support::oracle::grid::AxisEdges { start: 3.0, end: 0.0 },
            item_inline_offset: 7.0,
            item_block_offset: 11.0,
        },
    );

    assert_eq!(report.inherited_axis_offset, 30.0);
    assert_eq!(report.standalone_axis_offset, 16.0);
    assert_eq!(report.rect, support::oracle::grid::GridItemRect::new(30.0, 16.0, 80.0, 30.0));
}

#[test]
fn oracle_scenario_composes_lane_rect_from_lane_offset_and_grid_axis_area() {
    let rect = support::oracle::grid::compose_lane_item_rect(
        support::oracle::grid::LaneItemRectInput {
            grid_axis_start: 12.0,
            grid_axis_size: 50.0,
            lane_axis_offset: 27.0,
            lane_axis_size: 40.0,
            grid_axis_is_column: true,
        },
    );

    assert_eq!(rect, support::oracle::grid::GridItemRect::new(12.0, 27.0, 50.0, 40.0));
}
```

- [ ] **Step 2: Run focused failing tests**

Run:

```bash
cargo test -p surgeist --test oracle oracle_scenario_composes
```

Expected: compile failure for composition input types.

- [ ] **Step 3: Add explicit rectangle composition inputs**

Add to `scenario.rs`:

```rust
use super::subgrid::AxisEdges;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SubgridItemRectInput {
    pub inherited_axis: super::placement::GridAxis,
    pub inherited_axis_offset: f32,
    pub standalone_axis_offset: f32,
    pub inherited_axis_size: f32,
    pub standalone_axis_size: f32,
    pub container_mbp_offset: AxisEdges,
    pub item_inline_offset: f32,
    pub item_block_offset: f32,
}

#[must_use]
pub fn compose_subgrid_item_rect(input: SubgridItemRectInput) -> SubgridItemRectReport {
    let inherited_axis_offset =
        input.inherited_axis_offset + input.container_mbp_offset.start + input.item_inline_offset;
    let standalone_axis_offset = input.standalone_axis_offset + input.item_block_offset;
    let rect = match input.inherited_axis {
        super::placement::GridAxis::Column => GridItemRect::new(
            inherited_axis_offset,
            standalone_axis_offset,
            input.inherited_axis_size,
            input.standalone_axis_size,
        ),
        super::placement::GridAxis::Row => GridItemRect::new(
            standalone_axis_offset,
            inherited_axis_offset,
            input.standalone_axis_size,
            input.inherited_axis_size,
        ),
    };

    SubgridItemRectReport {
        inherited_axis_offset,
        standalone_axis_offset,
        rect,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SubgridItemRectReport {
    pub inherited_axis_offset: f32,
    pub standalone_axis_offset: f32,
    pub rect: GridItemRect,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LaneItemRectInput {
    pub grid_axis_start: f32,
    pub grid_axis_size: f32,
    pub lane_axis_offset: f32,
    pub lane_axis_size: f32,
    pub grid_axis_is_column: bool,
}

#[must_use]
pub fn compose_lane_item_rect(input: LaneItemRectInput) -> GridItemRect {
    if input.grid_axis_is_column {
        GridItemRect::new(
            input.grid_axis_start,
            input.lane_axis_offset,
            input.grid_axis_size,
            input.lane_axis_size,
        )
    } else {
        GridItemRect::new(
            input.lane_axis_offset,
            input.grid_axis_start,
            input.lane_axis_size,
            input.grid_axis_size,
        )
    }
}
```

- [ ] **Step 4: Export scenario helpers**

Update `grid/mod.rs` scenario exports:

```rust
pub use scenario::{
    GridItemRect, GridScenarioReport, LaneItemRectInput, SubgridItemRectInput,
    SubgridItemRectReport,
    compose_grid_scenario, compose_lane_item_rect, compose_subgrid_item_rect,
};
```

- [ ] **Step 5: Add module documentation guardrail**

At the top of `scenario.rs`, replace the module doc with:

```rust
//! Curated grid oracle scenarios composed from tested phase outputs.
//!
//! This module may combine explicit reports into final rectangles. It must not
//! traverse production trees, measure children, parse styles, or call production
//! layout algorithms.
```

- [ ] **Step 6: Verify scenario helpers**

Run:

```bash
cargo test -p surgeist --test oracle oracle_scenario_composes
```

Expected: both scenario composition tests pass.

- [ ] **Step 7: Commit**

```bash
git add crates/surgeist/tests/support/oracle/grid/scenario.rs crates/surgeist/tests/support/oracle/grid/mod.rs crates/surgeist/tests/oracle.rs
git commit -m "Guard grid oracle scenario composition"
```

---

## Task 8: Add Small Pure Oracle Composition Scenarios

**Files:**
- Modify: `crates/surgeist/tests/oracle.rs`

- [ ] **Step 1: Add pure composition scenarios without requiring production subgrid/grid-lanes support**

Append these pure oracle tests to `crates/surgeist/tests/oracle.rs`. They document high-signal composed shapes without pretending to compare production layout:

```rust
#[test]
fn oracle_direct_subgrid_inherited_columns_shape() {
    let inherited = support::oracle::grid::inherit_subgrid_tracks(
        support::oracle::grid::SubgridTrackInheritanceInput {
            parent_tracks: vec![80.0, 120.0],
            parent_span: support::oracle::grid::TrackSpan::new(1, 3),
            reversed: false,
            start_mbp: 0.0,
            end_mbp: 0.0,
            parent_gap: support::oracle::grid::OracleGapReport::length(10.0),
            subgrid_gap: support::oracle::grid::OracleGapReport::normal_resolved_to(10.0),
        },
    )
    .unwrap();

    assert_eq!(inherited.final_tracks, vec![80.0, 120.0]);
}

#[test]
fn oracle_grid_lanes_three_item_shape() {
    let report = support::oracle::grid::place_lanes(
        support::oracle::grid::LanePlacementInput {
            grid_axis_tracks: 2,
            auto_flow: support::oracle::grid::LaneAutoFlow::Row,
            lane_gap: 5.0,
            tolerance: support::oracle::grid::LaneFlowTolerance::Fixed(0.0),
            tolerance_basis: 0.0,
            items: vec![
                support::oracle::grid::LaneItemInput::auto("a", 1, 20.0),
                support::oracle::grid::LaneItemInput::auto("b", 1, 30.0),
                support::oracle::grid::LaneItemInput::auto("c", 2, 10.0),
            ],
        },
    )
    .unwrap();

    assert_eq!(report.item_offsets.len(), 3);
    assert_eq!(report.content_size, 45.0);
}
```

- [ ] **Step 2: Run pure oracle tests**

Run:

```bash
cargo test -p surgeist --test oracle oracle_direct_subgrid
cargo test -p surgeist --test oracle oracle_grid_lanes_three_item_shape
```

Expected: both pure oracle composition tests pass.

- [ ] **Step 3: Commit**

```bash
git add crates/surgeist/tests/oracle.rs
git commit -m "Add subgrid lanes oracle scenarios"
```

---

## Task 9: Full Verification And Cleanup

**Files:**
- Modify only files already touched by previous tasks if verification exposes issues.

- [ ] **Step 1: Run formatting check**

Run:

```bash
cargo fmt --check
```

Expected: pass.

- [ ] **Step 2: Run pure oracle tests**

Run:

```bash
cargo test -p surgeist --test oracle
```

Expected: pass.

- [ ] **Step 3: Run layout oracle tests**

Run:

```bash
cargo test -p surgeist --test layout_oracle
```

Expected: pass.

- [ ] **Step 4: Run Surgeist package tests**

Run:

```bash
cargo test -p surgeist
```

Expected: pass.

- [ ] **Step 5: Run Surgeist clippy**

Run:

```bash
cargo clippy -p surgeist --all-targets --all-features -- -D warnings
```

Expected: pass.

- [ ] **Step 6: Commit final cleanup if needed**

If formatting or lint fixes changed files, commit them:

```bash
git add crates/surgeist/tests/support/oracle/grid crates/surgeist/tests/oracle.rs crates/surgeist/tests/layout_oracle.rs
git commit -m "Clean up subgrid lanes oracle"
```

Skip this commit if no files changed.

---

## Reviewer Checklist

Before implementation starts, a clean-context reviewer should verify:

- The plan implements every required concept from `docs/superpowers/specs/2026-06-16-surgeist-subgrid-grid-lanes-oracle-design.md`.
- Each task has a bounded write set and a logical commit point.
- The plan avoids production layout calls and hidden measurement hooks.
- Grid-lanes finite placement uses the WebKit-inspired `max_start = grid_axis_track_count + 1 - span_len` candidate range.
- Subgrid edge placeholders require explicit intrinsic-min track facts.
- `normal` subgrid gaps resolve explicitly before gap-difference math.
- Pure composition scenarios stay in `oracle.rs`; production layout comparisons should be added later only when production subgrid/grid-lanes behavior exists.
