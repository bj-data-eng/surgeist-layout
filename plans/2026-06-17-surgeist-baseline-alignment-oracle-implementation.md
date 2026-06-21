# Surgeist Baseline Alignment Oracle Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement a test-only oracle for first/last baseline alignment across grid, subgrid, and grid-lanes before production engine work begins.

**Architecture:** Add a focused `baseline.rs` module under `crates/surgeist/tests/support/oracle/grid` and keep it phase-local: inputs are explicit baseline facts, spans, margins, gaps, and eligibility flags; outputs are reports with major/minor groups, offsets, shims, and propagated subgrid baselines. WebKit is the primary behavioral reference for participation, subgrid routing, and grid-lanes/masonry fallback; Blink is a secondary reference for compact baseline offset and accumulator formulas where they match WebKit's behavior.

**Tech Stack:** Rust test support under `crates/surgeist/tests/support/oracle/grid`, pure oracle tests in `crates/surgeist/tests/oracle.rs`, composed oracle/production comparison tests in `crates/surgeist/tests/layout_oracle.rs`, verification with `cargo test -p surgeist --test oracle`, `cargo test -p surgeist --test layout_oracle`, `cargo test -p surgeist`, and `cargo fmt --check`.

---

## Source References

- Existing oracle spec: `docs/superpowers/specs/2026-06-16-surgeist-subgrid-grid-lanes-oracle-design.md`
- Existing oracle plan: `docs/superpowers/plans/2026-06-16-surgeist-subgrid-grid-lanes-oracle-implementation.md`
- Engine baseline plan that this oracle plan must precede: `docs/superpowers/plans/2026-06-17-surgeist-baseline-alignment-engine-implementation.md`
- Existing oracle modules:
  - `crates/surgeist/tests/support/oracle/grid/mod.rs`
  - `crates/surgeist/tests/support/oracle/grid/alignment.rs`
  - `crates/surgeist/tests/support/oracle/grid/contributions.rs`
  - `crates/surgeist/tests/support/oracle/grid/placement.rs`
  - `crates/surgeist/tests/support/oracle/grid/scenario.rs`
  - `crates/surgeist/tests/support/oracle/grid/subgrid.rs`
  - `crates/surgeist/tests/support/oracle/grid/lanes.rs`
  - `crates/surgeist/tests/support/oracle/grid/tracks.rs`
- WebKit references:
  - `tmp/WebKit/Source/WebCore/rendering/GridBaselineAlignment.h`
  - `tmp/WebKit/Source/WebCore/rendering/GridTrackSizingAlgorithm.h`
  - `tmp/WebKit/Source/WebCore/rendering/GridTrackSizingAlgorithm.cpp`
  - `tmp/WebKit/Source/WebCore/rendering/RenderGrid.cpp`
  - `tmp/WebKit/Source/WebCore/rendering/RenderBlock.cpp`
- Blink references:
  - `tmp/chromium-inline/third_party/blink/renderer/core/layout/grid/grid_baseline_accumulator.h`
  - `tmp/chromium-inline/third_party/blink/renderer/core/layout/grid/grid_layout_utils.cc`
  - `tmp/chromium-inline/third_party/blink/renderer/core/layout/grid/grid_item.{cc,h}`
  - `tmp/chromium-inline/third_party/blink/renderer/core/layout/grid/grid_layout_algorithm.cc`

---

## Guardrails

- [ ] Do not import production layout code into oracle modules.
- [ ] Do not parse CSS or browser fixture XML inside oracle modules.
- [ ] Do not measure text, lay out children, traverse production trees, or call `compute_grid`.
- [ ] Do not implement WebKit's mutable baseline caches. The oracle should consume explicit facts and return reports.
- [ ] Do not hide inline formatting context gaps by treating them as successful grid oracle cases.
- [ ] Do not replace existing grid/subgrid/grid-lanes oracle modules; add baseline phases that compose with them.
- [ ] Commit after each logical task using short concrete messages.

---

## Baseline Oracle Model

The oracle should answer:

```text
Given explicit grid item baseline facts and explicit grid/subgrid/lane phase facts, what baseline groups, offsets, shims, and published baselines should this phase produce?
```

It should not answer:

```text
Given a styled tree, where are every element's browser-compatible baselines?
```

Coordinate convention:

- `first_baseline` is a border-box distance from the item's block-start edge.
- `last_baseline` is a border-box distance from the item's block-start edge.
- A major baseline contribution is `block_start_margin + first_baseline`.
- A minor baseline contribution is `block_end_margin + (border_box_block_size - last_baseline)`.
- Major groups store distances from alignment-context start.
- Minor groups store distances from alignment-context end.
- Spanning item computations use the whole spanned alignment area, including internal gaps.

WebKit behavior to model explicitly:

- Out-of-flow items do not participate in baseline alignment.
- Items with auto margins in the baseline axis do not participate.
- Synthesized baselines must fall back when they would create an intrinsic track sizing dependency.
- Subgridded-axis baseline offsets route through the ancestor grid.
- Grid-lanes/masonry baseline offsets are not applied in the lane axis; container baselines may still be synthesized from final geometry.

---

## Task 1: Add Baseline Oracle Module And Vocabulary

**Files:**
- Create `crates/surgeist/tests/support/oracle/grid/baseline.rs`
- Modify `crates/surgeist/tests/support/oracle/grid/mod.rs`
- Modify `crates/surgeist/tests/oracle.rs`

- [ ] Add failing tests for baseline coordinate facts.

Add to `crates/surgeist/tests/oracle.rs`:

```rust
#[test]
fn oracle_baseline_geometry_uses_margin_box_contributions() {
    let geometry = support::oracle::grid::BaselineGeometry::from_item(
        support::oracle::grid::BaselineItemFacts {
            id: "item",
            area: support::oracle::grid::GridArea::new(1, 1, 1, 1),
            block_size: 30.0,
            margin_before: 3.0,
            margin_after: 5.0,
            first_baseline: Some(8.0),
            last_baseline: Some(24.0),
            synthesized_first: false,
            synthesized_last: false,
            alignment: support::oracle::grid::BaselineAlignment::First,
            out_of_flow: false,
            baseline_axis_auto_margins: false,
            spans_intrinsic_track: false,
            baseline_requires_unavailable_subgrid_layout: false,
        },
        40.0,
    )
    .unwrap();

    assert_eq!(geometry.margin_box_size, 38.0);
    assert_eq!(geometry.major_baseline, 11.0);
    assert_eq!(geometry.minor_baseline, 11.0);
}
```

- [ ] Run the failing test.

```bash
cargo test -p surgeist --test oracle oracle_baseline_geometry_uses_margin_box_contributions
```

Expected: compile failure because `baseline.rs` and the baseline types do not exist.

- [ ] Create `baseline.rs` with the base vocabulary.

Expected initial code:

```rust
use super::placement::GridArea;
use super::subgrid::{OracleGapReport, OracleGridError, TrackSpan};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BaselineAlignment {
    None,
    First,
    Last,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BaselineGroupKind {
    Major,
    Minor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BaselineFallback {
    Start,
    End,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BaselineItemFacts {
    pub id: &'static str,
    pub area: GridArea,
    pub block_size: f32,
    pub margin_before: f32,
    pub margin_after: f32,
    pub first_baseline: Option<f32>,
    pub last_baseline: Option<f32>,
    pub synthesized_first: bool,
    pub synthesized_last: bool,
    pub alignment: BaselineAlignment,
    pub out_of_flow: bool,
    pub baseline_axis_auto_margins: bool,
    pub spans_intrinsic_track: bool,
    pub baseline_requires_unavailable_subgrid_layout: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BaselineGeometry {
    pub available_span_size: f32,
    pub margin_box_size: f32,
    pub major_baseline: f32,
    pub minor_baseline: f32,
}

impl BaselineGeometry {
    pub fn from_item(
        item: BaselineItemFacts,
        available_span_size: f32,
    ) -> Result<Self, OracleGridError> {
        let first = item.first_baseline.unwrap_or(item.block_size);
        let last = item.last_baseline.unwrap_or(0.0);
        if first < 0.0 || last < 0.0 || first > item.block_size || last > item.block_size {
            return Err(OracleGridError::BaselineInferenceUnsupported);
        }
        Ok(Self {
            available_span_size,
            margin_box_size: item.margin_before + item.block_size + item.margin_after,
            major_baseline: item.margin_before + first,
            minor_baseline: item.margin_after + item.block_size - last,
        })
    }
}
```

- [ ] Export the module from `mod.rs`.

Expected changes:

```rust
pub mod baseline;

#[allow(unused_imports)]
pub use baseline::{
    BaselineAlignment, BaselineFallback, BaselineGeometry, BaselineGroupKind, BaselineItemFacts,
};
```

- [ ] Run:

```bash
cargo test -p surgeist --test oracle oracle_baseline_geometry_uses_margin_box_contributions
cargo fmt --check
```

- [ ] Commit:

```bash
git status --short --branch
git diff --check
git add crates/surgeist/tests/support/oracle/grid/baseline.rs crates/surgeist/tests/support/oracle/grid/mod.rs crates/surgeist/tests/oracle.rs
git commit -m "Add baseline oracle vocabulary"
```

---

## Task 2: Model WebKit Participation And Fallback Rules

**Files:**
- Modify `crates/surgeist/tests/support/oracle/grid/baseline.rs`
- Modify `crates/surgeist/tests/support/oracle/grid/mod.rs`
- Modify `crates/surgeist/tests/oracle.rs`

- [ ] Add failing participation tests.

Add this helper near the participation tests in `crates/surgeist/tests/oracle.rs`:

```rust
fn oracle_baseline_test_item(
    id: &'static str,
    alignment: support::oracle::grid::BaselineAlignment,
) -> support::oracle::grid::BaselineItemFacts {
    support::oracle::grid::BaselineItemFacts {
        id,
        area: support::oracle::grid::GridArea::new(1, 1, 1, 1),
        block_size: 20.0,
        margin_before: 0.0,
        margin_after: 0.0,
        first_baseline: Some(8.0),
        last_baseline: Some(16.0),
        synthesized_first: false,
        synthesized_last: false,
        alignment,
        out_of_flow: false,
        baseline_axis_auto_margins: false,
        spans_intrinsic_track: false,
        baseline_requires_unavailable_subgrid_layout: false,
    }
}
```

```rust
#[test]
fn oracle_baseline_participation_rejects_out_of_flow_items() {
    let mut item = oracle_baseline_test_item("abspos", support::oracle::grid::BaselineAlignment::First);
    item.out_of_flow = true;
    let report = support::oracle::grid::baseline_participation(item);

    assert_eq!(report.participates, false);
    assert_eq!(report.fallback, Some(support::oracle::grid::BaselineFallback::Start));
}

#[test]
fn oracle_baseline_participation_rejects_auto_margins() {
    let mut item = oracle_baseline_test_item("auto-margin", support::oracle::grid::BaselineAlignment::Last);
    item.baseline_axis_auto_margins = true;
    let report = support::oracle::grid::baseline_participation(item);

    assert_eq!(report.participates, false);
    assert_eq!(report.fallback, Some(support::oracle::grid::BaselineFallback::End));
}

#[test]
fn oracle_baseline_participation_falls_back_for_synthesized_intrinsic_cycles() {
    let mut item = oracle_baseline_test_item("synth", support::oracle::grid::BaselineAlignment::First);
    item.first_baseline = None;
    item.synthesized_first = true;
    item.spans_intrinsic_track = true;
    let report = support::oracle::grid::baseline_participation(item);

    assert_eq!(report.participates, false);
    assert_eq!(report.fallback, Some(support::oracle::grid::BaselineFallback::Start));
}

#[test]
fn oracle_baseline_participation_falls_back_for_unavailable_subgrid_layout() {
    let mut item = oracle_baseline_test_item("subgrid-synth", support::oracle::grid::BaselineAlignment::First);
    item.first_baseline = None;
    item.synthesized_first = true;
    item.baseline_requires_unavailable_subgrid_layout = true;
    let report = support::oracle::grid::baseline_participation(item);

    assert_eq!(report.participates, false);
    assert_eq!(report.fallback, Some(support::oracle::grid::BaselineFallback::Start));
}
```

- [ ] Run:

```bash
cargo test -p surgeist --test oracle oracle_baseline_participation_rejects_out_of_flow_items
cargo test -p surgeist --test oracle oracle_baseline_participation_rejects_auto_margins
cargo test -p surgeist --test oracle oracle_baseline_participation_falls_back_for_synthesized_intrinsic_cycles
cargo test -p surgeist --test oracle oracle_baseline_participation_falls_back_for_unavailable_subgrid_layout
```

Expected: fail because `baseline_participation` and `BaselineParticipationReport` do not exist.

- [ ] Implement participation reports.

Expected code:

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BaselineParticipationReport {
    pub id: &'static str,
    pub alignment: BaselineAlignment,
    pub participates: bool,
    pub group: Option<BaselineGroupKind>,
    pub fallback: Option<BaselineFallback>,
    pub used_synthesized_baseline: bool,
}

#[must_use]
pub fn baseline_participation(item: BaselineItemFacts) -> BaselineParticipationReport {
    let (group, fallback) = match item.alignment {
        BaselineAlignment::None => (None, None),
        BaselineAlignment::First => (Some(BaselineGroupKind::Major), Some(BaselineFallback::Start)),
        BaselineAlignment::Last => (Some(BaselineGroupKind::Minor), Some(BaselineFallback::End)),
    };
    let used_synthesized_baseline = match item.alignment {
        BaselineAlignment::First => item.first_baseline.is_none() || item.synthesized_first,
        BaselineAlignment::Last => item.last_baseline.is_none() || item.synthesized_last,
        BaselineAlignment::None => false,
    };
    let cycle_fallback = used_synthesized_baseline
        && (item.spans_intrinsic_track || item.baseline_requires_unavailable_subgrid_layout);
    let participates =
        group.is_some() && !item.out_of_flow && !item.baseline_axis_auto_margins && !cycle_fallback;

    BaselineParticipationReport {
        id: item.id,
        alignment: item.alignment,
        participates,
        group: participates.then_some(group.expect("baseline group exists")),
        fallback: (!participates).then_some(fallback).flatten(),
        used_synthesized_baseline,
    }
}
```

- [ ] Run:

```bash
cargo test -p surgeist --test oracle oracle_baseline_participation
cargo fmt --check
```

- [ ] Commit:

```bash
git status --short --branch
git diff --check
git add crates/surgeist/tests/support/oracle/grid/baseline.rs crates/surgeist/tests/support/oracle/grid/mod.rs crates/surgeist/tests/oracle.rs
git commit -m "Model baseline participation"
```

---

## Task 3: Compute Major And Minor Baseline Groups

**Files:**
- Modify `crates/surgeist/tests/support/oracle/grid/baseline.rs`
- Modify `crates/surgeist/tests/support/oracle/grid/mod.rs`
- Modify `crates/surgeist/tests/oracle.rs`

- [ ] Add failing group tests.

Add this helper near the group tests:

```rust
fn oracle_baseline_item(
    id: &'static str,
    row_start: usize,
    row_span: usize,
    alignment: support::oracle::grid::BaselineAlignment,
    block_size: f32,
    margin_before: f32,
    margin_after: f32,
    first_baseline: Option<f32>,
    last_baseline: Option<f32>,
) -> support::oracle::grid::BaselineItemFacts {
    support::oracle::grid::BaselineItemFacts {
        id,
        area: support::oracle::grid::GridArea::new(row_start, 1, row_span, 1),
        block_size,
        margin_before,
        margin_after,
        first_baseline,
        last_baseline,
        synthesized_first: first_baseline.is_none(),
        synthesized_last: last_baseline.is_none(),
        alignment,
        out_of_flow: false,
        baseline_axis_auto_margins: false,
        spans_intrinsic_track: false,
        baseline_requires_unavailable_subgrid_layout: false,
    }
}
```

```rust
#[test]
fn oracle_baseline_groups_collect_major_group_on_start_track() {
    let report = support::oracle::grid::baseline_groups(
        support::oracle::grid::BaselineGroupInput {
            track_count: 3,
            track_sizes: vec![30.0, 40.0, 50.0],
            gap: 5.0,
            items: vec![
                oracle_baseline_item("a", 1, 1, support::oracle::grid::BaselineAlignment::First, 20.0, 3.0, 2.0, Some(8.0), Some(16.0)),
                oracle_baseline_item("b", 1, 1, support::oracle::grid::BaselineAlignment::First, 24.0, 1.0, 1.0, Some(12.0), Some(18.0)),
            ],
        },
    )
    .unwrap();

    assert_eq!(report.major[0], Some(13.0));
    assert_eq!(report.minor, vec![None, None, None]);
}

#[test]
fn oracle_baseline_groups_collect_minor_group_on_end_track_for_spanning_item() {
    let report = support::oracle::grid::baseline_groups(
        support::oracle::grid::BaselineGroupInput {
            track_count: 3,
            track_sizes: vec![30.0, 40.0, 50.0],
            gap: 5.0,
            items: vec![
                oracle_baseline_item("span", 1, 2, support::oracle::grid::BaselineAlignment::Last, 30.0, 2.0, 4.0, Some(8.0), Some(22.0)),
            ],
        },
    )
    .unwrap();

    assert_eq!(report.minor[1], Some(12.0));
}
```

- [ ] Run:

```bash
cargo test -p surgeist --test oracle oracle_baseline_groups_collect_major_group_on_start_track
cargo test -p surgeist --test oracle oracle_baseline_groups_collect_minor_group_on_end_track_for_spanning_item
```

Expected: fail because `BaselineGroupInput`, `BaselineGroupReport`, and `baseline_groups` do not exist.

- [ ] Implement grouping.

Expected types:

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct BaselineGroupInput {
    pub track_count: usize,
    pub track_sizes: Vec<f32>,
    pub gap: f32,
    pub items: Vec<BaselineItemFacts>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BaselineGroupReport {
    pub major: Vec<Option<f32>>,
    pub minor: Vec<Option<f32>>,
    pub participation: Vec<BaselineParticipationReport>,
}
```

Rules:

- Validate `track_count > 0`.
- Validate `track_sizes.len() == track_count`.
- Validate every item row span is in range.
- Major groups use the item's start-most track index.
- Minor groups use the item's end-most track index.
- Group value is the maximum baseline contribution for that group.
- Nonparticipants appear in `participation` but do not change `major` or `minor`.

- [ ] Run:

```bash
cargo test -p surgeist --test oracle oracle_baseline_groups
cargo fmt --check
```

- [ ] Commit:

```bash
git status --short --branch
git diff --check
git add crates/surgeist/tests/support/oracle/grid/baseline.rs crates/surgeist/tests/support/oracle/grid/mod.rs crates/surgeist/tests/oracle.rs
git commit -m "Add baseline oracle groups"
```

---

## Task 4: Compute Baseline Offsets And Intrinsic Shims

**Files:**
- Modify `crates/surgeist/tests/support/oracle/grid/baseline.rs`
- Modify `crates/surgeist/tests/support/oracle/grid/mod.rs`
- Modify `crates/surgeist/tests/oracle.rs`

- [ ] Add failing offset tests matching WebKit behavior and Blink's formula.

```rust
#[test]
fn oracle_baseline_offset_uses_whole_spanned_area_for_major_group() {
    let offset = support::oracle::grid::baseline_offset(
        support::oracle::grid::BaselineGroupKind::Major,
        20.0,
        support::oracle::grid::BaselineGeometry {
            available_span_size: 75.0,
            margin_box_size: 38.0,
            major_baseline: 11.0,
            minor_baseline: 11.0,
        },
    );

    assert_eq!(offset, 9.0);
}

#[test]
fn oracle_baseline_offset_uses_whole_spanned_area_for_minor_group() {
    let offset = support::oracle::grid::baseline_offset(
        support::oracle::grid::BaselineGroupKind::Minor,
        12.0,
        support::oracle::grid::BaselineGeometry {
            available_span_size: 75.0,
            margin_box_size: 38.0,
            major_baseline: 11.0,
            minor_baseline: 9.0,
        },
    );

    assert_eq!(offset, 34.0);
}
```

- [ ] Add failing shim tests.

```rust
#[test]
fn oracle_baseline_shim_grows_before_for_major_group() {
    let shim = support::oracle::grid::baseline_intrinsic_shim(
        support::oracle::grid::BaselineGroupKind::Major,
        20.0,
        support::oracle::grid::BaselineGeometry {
            available_span_size: 30.0,
            margin_box_size: 26.0,
            major_baseline: 11.0,
            minor_baseline: 8.0,
        },
    );

    assert_eq!(shim.before, 9.0);
    assert_eq!(shim.after, 0.0);
}

#[test]
fn oracle_baseline_shim_grows_after_for_minor_group() {
    let shim = support::oracle::grid::baseline_intrinsic_shim(
        support::oracle::grid::BaselineGroupKind::Minor,
        12.0,
        support::oracle::grid::BaselineGeometry {
            available_span_size: 30.0,
            margin_box_size: 26.0,
            major_baseline: 11.0,
            minor_baseline: 7.0,
        },
    );

    assert_eq!(shim.before, 0.0);
    assert_eq!(shim.after, 5.0);
}
```

- [ ] Run:

```bash
cargo test -p surgeist --test oracle oracle_baseline_offset
cargo test -p surgeist --test oracle oracle_baseline_shim
```

Expected: fail because helpers do not exist.

- [ ] Implement offset and shim helpers.

Expected code:

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BaselineShim {
    pub before: f32,
    pub after: f32,
}

#[must_use]
pub fn baseline_offset(
    group: BaselineGroupKind,
    shared_baseline: f32,
    geometry: BaselineGeometry,
) -> f32 {
    match group {
        BaselineGroupKind::Major => shared_baseline - geometry.major_baseline,
        BaselineGroupKind::Minor => {
            let baseline_delta = shared_baseline - geometry.minor_baseline;
            geometry.available_span_size - baseline_delta - geometry.margin_box_size
        }
    }
}

#[must_use]
pub fn baseline_intrinsic_shim(
    group: BaselineGroupKind,
    shared_baseline: f32,
    geometry: BaselineGeometry,
) -> BaselineShim {
    match group {
        BaselineGroupKind::Major => BaselineShim {
            before: (shared_baseline - geometry.major_baseline).max(0.0),
            after: 0.0,
        },
        BaselineGroupKind::Minor => BaselineShim {
            before: 0.0,
            after: (shared_baseline - geometry.minor_baseline).max(0.0),
        },
    }
}
```

- [ ] Run:

```bash
cargo test -p surgeist --test oracle oracle_baseline_offset
cargo test -p surgeist --test oracle oracle_baseline_shim
cargo fmt --check
```

- [ ] Commit:

```bash
git status --short --branch
git diff --check
git add crates/surgeist/tests/support/oracle/grid/baseline.rs crates/surgeist/tests/support/oracle/grid/mod.rs crates/surgeist/tests/oracle.rs
git commit -m "Add baseline oracle offsets"
```

---

## Task 5: Add Container Baseline Accumulator

**Files:**
- Modify `crates/surgeist/tests/support/oracle/grid/baseline.rs`
- Modify `crates/surgeist/tests/support/oracle/grid/mod.rs`
- Modify `crates/surgeist/tests/oracle.rs`

- [ ] Add failing container baseline tests.

```rust
#[test]
fn oracle_container_baselines_prefer_major_and_minor_groups() {
    let report = support::oracle::grid::container_baselines(
        support::oracle::grid::ContainerBaselineInput {
            track_offsets: vec![0.0, 40.0],
            track_sizes: vec![30.0, 30.0],
            groups: support::oracle::grid::BaselineGroupReport {
                major: vec![Some(14.0), None],
                minor: vec![None, Some(6.0)],
                participation: Vec::new(),
            },
            fallback_items: vec![
                support::oracle::grid::ContainerBaselineFallbackItem {
                    id: "first",
                    area: support::oracle::grid::GridArea::new(1, 1, 1, 1),
                    block_offset: 0.0,
                    first_baseline: 8.0,
                    last_baseline: 20.0,
                },
                support::oracle::grid::ContainerBaselineFallbackItem {
                    id: "last",
                    area: support::oracle::grid::GridArea::new(2, 1, 1, 1),
                    block_offset: 40.0,
                    first_baseline: 10.0,
                    last_baseline: 24.0,
                },
            ],
        },
    )
    .unwrap();

    assert_eq!(report.first, Some(14.0));
    assert_eq!(report.last, Some(64.0));
}

#[test]
fn oracle_container_baselines_use_minor_group_for_first_when_major_missing() {
    let report = support::oracle::grid::container_baselines(
        support::oracle::grid::ContainerBaselineInput {
            track_offsets: vec![0.0],
            track_sizes: vec![30.0],
            groups: support::oracle::grid::BaselineGroupReport {
                major: vec![None],
                minor: vec![Some(6.0)],
                participation: Vec::new(),
            },
            fallback_items: Vec::new(),
        },
    )
    .unwrap();

    assert_eq!(report.first, Some(24.0));
    assert_eq!(report.last, Some(24.0));
}

#[test]
fn oracle_container_baselines_use_major_group_for_last_when_minor_missing() {
    let report = support::oracle::grid::container_baselines(
        support::oracle::grid::ContainerBaselineInput {
            track_offsets: vec![40.0],
            track_sizes: vec![30.0],
            groups: support::oracle::grid::BaselineGroupReport {
                major: vec![Some(12.0)],
                minor: vec![None],
                participation: Vec::new(),
            },
            fallback_items: Vec::new(),
        },
    )
    .unwrap();

    assert_eq!(report.first, Some(52.0));
    assert_eq!(report.last, Some(52.0));
}

#[test]
fn oracle_container_baselines_fallback_by_grid_order_and_synthesis() {
    let report = support::oracle::grid::container_baselines(
        support::oracle::grid::ContainerBaselineInput {
            track_offsets: vec![0.0, 40.0],
            track_sizes: vec![30.0, 30.0],
            groups: support::oracle::grid::BaselineGroupReport {
                major: vec![None, None],
                minor: vec![None, None],
                participation: Vec::new(),
            },
            fallback_items: vec![
                support::oracle::grid::ContainerBaselineFallbackItem {
                    id: "row-2-col-1",
                    area: support::oracle::grid::GridArea::new(2, 1, 1, 1),
                    block_offset: 40.0,
                    first_baseline: 70.0,
                    last_baseline: 40.0,
                },
                support::oracle::grid::ContainerBaselineFallbackItem {
                    id: "row-1-col-2-synth-first",
                    area: support::oracle::grid::GridArea::new(1, 2, 1, 1),
                    block_offset: 0.0,
                    first_baseline: 30.0,
                    last_baseline: 6.0,
                },
                support::oracle::grid::ContainerBaselineFallbackItem {
                    id: "row-1-col-1",
                    area: support::oracle::grid::GridArea::new(1, 1, 1, 1),
                    block_offset: 0.0,
                    first_baseline: 8.0,
                    last_baseline: 22.0,
                },
            ],
        },
    )
    .unwrap();

    assert_eq!(report.first, Some(8.0));
    assert_eq!(report.last, Some(40.0));
}
```

- [ ] Run:

```bash
cargo test -p surgeist --test oracle oracle_container_baselines_prefer_major_and_minor_groups
cargo test -p surgeist --test oracle oracle_container_baselines_use_minor_group_for_first_when_major_missing
cargo test -p surgeist --test oracle oracle_container_baselines_use_major_group_for_last_when_minor_missing
cargo test -p surgeist --test oracle oracle_container_baselines_fallback_by_grid_order_and_synthesis
```

Expected: fail because container baseline accumulator types do not exist.

- [ ] Implement accumulator.

Expected behavior:

- Determine the first occupied row from major groups, minor groups, and fallback items. Determine the last occupied row from the same three sources.
- First baseline priority:
  - Use the first occupied row's major group when present: `track_offsets[row] + major`.
  - Otherwise use the first occupied row's minor group when present: `track_offsets[row] + track_sizes[row] - minor`.
  - Otherwise use the first fallback item in grid order, sorting by row then column, and read its `first_baseline` value.
- Last baseline priority:
  - Use the last occupied row's minor group when present: `track_offsets[row] + track_sizes[row] - minor`.
  - Otherwise use the last occupied row's major group when present: `track_offsets[row] + major`.
  - Otherwise use the last fallback item in reverse grid order, sorting by row then column, and read its `last_baseline` value.
- `ContainerBaselineFallbackItem::first_baseline` and `ContainerBaselineFallbackItem::last_baseline` are explicit final baseline coordinates. Tests may pass synthesized fallback coordinates, but the accumulator must not synthesize them internally.
- Empty inputs return `None` for both baselines.

- [ ] Run:

```bash
cargo test -p surgeist --test oracle oracle_container_baselines
cargo fmt --check
```

- [ ] Commit:

```bash
git status --short --branch
git diff --check
git add crates/surgeist/tests/support/oracle/grid/baseline.rs crates/surgeist/tests/support/oracle/grid/mod.rs crates/surgeist/tests/oracle.rs
git commit -m "Add container baseline oracle"
```

---

## Task 6: Add Subgrid Baseline Inheritance Oracle

**Files:**
- Modify `crates/surgeist/tests/support/oracle/grid/baseline.rs`
- Modify `crates/surgeist/tests/support/oracle/grid/subgrid.rs`
- Modify `crates/surgeist/tests/support/oracle/grid/mod.rs`
- Modify `crates/surgeist/tests/oracle.rs`

- [ ] Add failing inheritance tests.

```rust
#[test]
fn oracle_subgrid_baselines_slice_parent_groups_for_span() {
    let report = support::oracle::grid::inherit_subgrid_baselines(
        support::oracle::grid::SubgridBaselineInheritanceInput {
            parent_span: support::oracle::grid::TrackSpan::new(2, 4),
            reversed: false,
            parent_gap: support::oracle::grid::OracleGapReport::normal_resolved_to(10.0),
            subgrid_gap: support::oracle::grid::OracleGapReport::length(10.0),
            start_mbp: 0.0,
            end_mbp: 0.0,
            parent_major: vec![Some(4.0), Some(8.0), None, Some(6.0)],
            parent_minor: vec![None, Some(5.0), Some(7.0), None],
        },
    )
    .unwrap();

    assert_eq!(report.sliced_major, vec![Some(8.0), None]);
    assert_eq!(report.sliced_minor, vec![Some(5.0), Some(7.0)]);
    assert_eq!(report.after_reversal_major, vec![Some(8.0), None]);
    assert_eq!(report.after_reversal_minor, vec![Some(5.0), Some(7.0)]);
    assert_eq!(report.after_mbp_major, vec![Some(8.0), None]);
    assert_eq!(report.after_mbp_minor, vec![Some(5.0), Some(7.0)]);
    assert_eq!(report.final_major, vec![Some(8.0), None]);
    assert_eq!(report.final_minor, vec![Some(5.0), Some(7.0)]);
}

#[test]
fn oracle_subgrid_baselines_reverse_and_adjust_edges() {
    let report = support::oracle::grid::inherit_subgrid_baselines(
        support::oracle::grid::SubgridBaselineInheritanceInput {
            parent_span: support::oracle::grid::TrackSpan::new(1, 3),
            reversed: true,
            parent_gap: support::oracle::grid::OracleGapReport::normal_resolved_to(10.0),
            subgrid_gap: support::oracle::grid::OracleGapReport::length(20.0),
            start_mbp: 3.0,
            end_mbp: 5.0,
            parent_major: vec![Some(10.0), Some(14.0)],
            parent_minor: vec![Some(4.0), Some(8.0)],
        },
    )
    .unwrap();

    assert_eq!(report.final_major.len(), 2);
    assert_eq!(report.final_minor.len(), 2);
    assert_eq!(report.reversed, true);
    assert_eq!(report.start_mbp, 3.0);
    assert_eq!(report.end_mbp, 5.0);
    assert_eq!(report.gap_difference, 5.0);
    assert_eq!(report.sliced_major, vec![Some(10.0), Some(14.0)]);
    assert_eq!(report.sliced_minor, vec![Some(4.0), Some(8.0)]);
    assert_eq!(report.after_reversal_major, vec![Some(14.0), Some(10.0)]);
    assert_eq!(report.after_reversal_minor, vec![Some(8.0), Some(4.0)]);
    assert_eq!(report.after_mbp_major, vec![Some(17.0), Some(10.0)]);
    assert_eq!(report.after_mbp_minor, vec![Some(8.0), Some(9.0)]);
    assert_eq!(report.final_major, vec![Some(12.0), Some(5.0)]);
    assert_eq!(report.final_minor, vec![Some(3.0), Some(4.0)]);
}
```

- [ ] Run:

```bash
cargo test -p surgeist --test oracle oracle_subgrid_baselines_slice_parent_groups_for_span
cargo test -p surgeist --test oracle oracle_subgrid_baselines_reverse_and_adjust_edges
```

Expected: fail because the subgrid baseline inheritance API does not exist.

- [ ] Implement inheritance reports.

Expected types:

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct SubgridBaselineInheritanceInput {
    pub parent_span: TrackSpan,
    pub reversed: bool,
    pub parent_gap: OracleGapReport,
    pub subgrid_gap: OracleGapReport,
    pub start_mbp: f32,
    pub end_mbp: f32,
    pub parent_major: Vec<Option<f32>>,
    pub parent_minor: Vec<Option<f32>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SubgridBaselineInheritanceReport {
    pub parent_span: TrackSpan,
    pub reversed: bool,
    pub start_mbp: f32,
    pub end_mbp: f32,
    pub parent_gap: OracleGapReport,
    pub subgrid_gap: OracleGapReport,
    pub gap_difference: f32,
    pub sliced_major: Vec<Option<f32>>,
    pub sliced_minor: Vec<Option<f32>>,
    pub after_reversal_major: Vec<Option<f32>>,
    pub after_reversal_minor: Vec<Option<f32>>,
    pub after_mbp_major: Vec<Option<f32>>,
    pub after_mbp_minor: Vec<Option<f32>>,
    pub final_major: Vec<Option<f32>>,
    pub final_minor: Vec<Option<f32>>,
}
```

Rules:

- Slice parent groups using the 1-based `TrackSpan`.
- Reverse order when `reversed` is true.
- Preserve `None` entries.
- Resolve gaps through the existing `OracleGapReport` type so `normal` remains visible to tests and has an explicit used value.
- Store `gap_difference = (subgrid_gap.resolved - parent_gap.resolved) / 2.0`.
- Apply start MBP only to the first major baseline coordinate when it exists.
- Apply end MBP only to the last minor baseline coordinate when it exists.
- Apply positive gap difference by subtracting `gap_difference` from inherited coordinates adjacent to the subgrid's internal gap. For the two-track case in the test, subtract it from both final major coordinates and both final minor coordinates after MBP adjustment.
- Return all stages in the report: sliced groups, after reversal, after MBP, and final gap-adjusted groups.
- Do not synthesize inherited baselines.

- [ ] Export the API from `mod.rs`.

- [ ] Run:

```bash
cargo test -p surgeist --test oracle oracle_subgrid_baselines
cargo fmt --check
```

- [ ] Commit:

```bash
git status --short --branch
git diff --check
git add crates/surgeist/tests/support/oracle/grid/baseline.rs crates/surgeist/tests/support/oracle/grid/subgrid.rs crates/surgeist/tests/support/oracle/grid/mod.rs crates/surgeist/tests/oracle.rs
git commit -m "Add subgrid baseline inheritance oracle"
```

---

## Task 7: Add Subgrid Descendant Baseline Publication Oracle

**Files:**
- Modify `crates/surgeist/tests/support/oracle/grid/baseline.rs`
- Modify `crates/surgeist/tests/support/oracle/grid/subgrid.rs`
- Modify `crates/surgeist/tests/support/oracle/grid/mod.rs`
- Modify `crates/surgeist/tests/oracle.rs`

- [ ] Add failing publication tests.

```rust
#[test]
fn oracle_subgrid_publishes_descendant_baseline_to_ancestor_track() {
    let report = support::oracle::grid::publish_subgrid_baseline(
        support::oracle::grid::SubgridBaselinePublicationInput {
            subgrid_span_in_parent: support::oracle::grid::TrackSpan::new(2, 4),
            subgrid_offset_in_parent: 40.0,
            reversed: false,
            descendant_local_track: 1,
            descendant_track_offset_in_subgrid: 20.0,
            descendant_group: support::oracle::grid::BaselineGroupKind::Major,
            descendant_baseline_in_track: 12.0,
            inherited_axis_offset: 3.0,
            synthesized_cycle_fallback: false,
        },
    )
    .unwrap();

    assert_eq!(report.ancestor_track, 2);
    assert_eq!(report.group, support::oracle::grid::BaselineGroupKind::Major);
    assert_eq!(report.baseline, 75.0);
}

#[test]
fn oracle_subgrid_publishes_reversed_descendant_baseline_to_ancestor_track() {
    let report = support::oracle::grid::publish_subgrid_baseline(
        support::oracle::grid::SubgridBaselinePublicationInput {
            subgrid_span_in_parent: support::oracle::grid::TrackSpan::new(2, 5),
            subgrid_offset_in_parent: 40.0,
            reversed: true,
            descendant_local_track: 1,
            descendant_track_offset_in_subgrid: 20.0,
            descendant_group: support::oracle::grid::BaselineGroupKind::Minor,
            descendant_baseline_in_track: 12.0,
            inherited_axis_offset: 3.0,
            synthesized_cycle_fallback: false,
        },
    )
    .unwrap();

    assert_eq!(report.ancestor_track, 4);
    assert_eq!(report.group, support::oracle::grid::BaselineGroupKind::Minor);
    assert_eq!(report.baseline, 75.0);
}

#[test]
fn oracle_subgrid_does_not_publish_synthesized_cycle_fallback() {
    let report = support::oracle::grid::publish_subgrid_baseline(
        support::oracle::grid::SubgridBaselinePublicationInput {
            subgrid_span_in_parent: support::oracle::grid::TrackSpan::new(2, 4),
            subgrid_offset_in_parent: 40.0,
            reversed: false,
            descendant_local_track: 1,
            descendant_track_offset_in_subgrid: 20.0,
            descendant_group: support::oracle::grid::BaselineGroupKind::Major,
            descendant_baseline_in_track: 12.0,
            inherited_axis_offset: 3.0,
            synthesized_cycle_fallback: true,
        },
    )
    .unwrap();

    assert_eq!(report.published, false);
}
```

- [ ] Run:

```bash
cargo test -p surgeist --test oracle oracle_subgrid_publishes_descendant_baseline_to_ancestor_track
cargo test -p surgeist --test oracle oracle_subgrid_publishes_reversed_descendant_baseline_to_ancestor_track
cargo test -p surgeist --test oracle oracle_subgrid_does_not_publish_synthesized_cycle_fallback
```

Expected: fail because publication API does not exist.

- [ ] Implement publication.

Expected types:

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct SubgridBaselinePublicationInput {
    pub subgrid_span_in_parent: TrackSpan,
    pub subgrid_offset_in_parent: f32,
    pub reversed: bool,
    pub descendant_local_track: usize,
    pub descendant_track_offset_in_subgrid: f32,
    pub descendant_group: BaselineGroupKind,
    pub descendant_baseline_in_track: f32,
    pub inherited_axis_offset: f32,
    pub synthesized_cycle_fallback: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SubgridBaselinePublicationReport {
    pub published: bool,
    pub ancestor_track: usize,
    pub group: BaselineGroupKind,
    pub baseline: f32,
}
```

Expected behavior:

- Translate local track index into ancestor track index using the subgrid parent span.
- Reverse local index when the subgrid axis is reversed. For a span `TrackSpan::new(2, 5)`, local track `1` maps to ancestor track `4`.
- Treat `descendant_track_offset_in_subgrid` as a distance from subgrid content start to the descendant local track start.
- Treat `descendant_baseline_in_track` as a distance from descendant local track start to the descendant baseline.
- Publish `subgrid_offset_in_parent + inherited_axis_offset + descendant_track_offset_in_subgrid + descendant_baseline_in_track`.
- Return `published = false` when `synthesized_cycle_fallback` is true.
- Validate local track index against the parent span length.

- [ ] Run:

```bash
cargo test -p surgeist --test oracle oracle_subgrid_publish
cargo fmt --check
```

- [ ] Commit:

```bash
git status --short --branch
git diff --check
git add crates/surgeist/tests/support/oracle/grid/baseline.rs crates/surgeist/tests/support/oracle/grid/subgrid.rs crates/surgeist/tests/support/oracle/grid/mod.rs crates/surgeist/tests/oracle.rs
git commit -m "Add subgrid baseline publication oracle"
```

---

## Task 8: Add Grid-Lanes Baseline Fallback Oracle

**Files:**
- Modify `crates/surgeist/tests/support/oracle/grid/baseline.rs`
- Modify `crates/surgeist/tests/support/oracle/grid/lanes.rs`
- Modify `crates/surgeist/tests/support/oracle/grid/mod.rs`
- Modify `crates/surgeist/tests/oracle.rs`

- [ ] Add failing grid-lanes fallback tests.

```rust
#[test]
fn oracle_grid_lanes_disables_row_axis_item_baseline_offsets() {
    let report = support::oracle::grid::grid_lanes_baseline_policy(
        support::oracle::grid::GridLanesBaselineInput {
            auto_flow: support::oracle::grid::LaneAutoFlow::Row,
            queried_axis: support::oracle::grid::GridAxis::Row,
            requested_alignment: support::oracle::grid::BaselineAlignment::First,
            has_items: true,
        },
    );

    assert_eq!(report.applies_item_offsets, false);
    assert_eq!(report.reason, Some(support::oracle::grid::GridLanesBaselineReason::WebKitMasonryFallback));
}

#[test]
fn oracle_grid_lanes_disables_column_axis_item_baseline_offsets() {
    let report = support::oracle::grid::grid_lanes_baseline_policy(
        support::oracle::grid::GridLanesBaselineInput {
            auto_flow: support::oracle::grid::LaneAutoFlow::Column,
            queried_axis: support::oracle::grid::GridAxis::Column,
            requested_alignment: support::oracle::grid::BaselineAlignment::Last,
            has_items: true,
        },
    );

    assert_eq!(report.applies_item_offsets, false);
    assert_eq!(report.reason, Some(support::oracle::grid::GridLanesBaselineReason::WebKitMasonryFallback));
}

#[test]
fn oracle_grid_lanes_can_synthesize_container_baselines_from_geometry() {
    let report = support::oracle::grid::grid_lanes_container_baselines(
        vec![
            support::oracle::grid::ContainerBaselineFallbackItem {
                id: "a",
                area: support::oracle::grid::GridArea::new(1, 1, 1, 1),
                block_offset: 0.0,
                first_baseline: 20.0,
                last_baseline: 0.0,
            },
            support::oracle::grid::ContainerBaselineFallbackItem {
                id: "b",
                area: support::oracle::grid::GridArea::new(2, 1, 1, 1),
                block_offset: 30.0,
                first_baseline: 30.0,
                last_baseline: 0.0,
            },
        ],
    );

    assert_eq!(report.first, Some(20.0));
    assert_eq!(report.last, Some(30.0));
}
```

- [ ] Run:

```bash
cargo test -p surgeist --test oracle oracle_grid_lanes_disables_row_axis_item_baseline_offsets
cargo test -p surgeist --test oracle oracle_grid_lanes_disables_column_axis_item_baseline_offsets
cargo test -p surgeist --test oracle oracle_grid_lanes_can_synthesize_container_baselines_from_geometry
```

Expected: fail because grid-lanes baseline helpers do not exist.

- [ ] Implement grid-lanes baseline policy.

Expected types:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GridLanesBaselineReason {
    WebKitMasonryFallback,
    NoItems,
    NoBaselineAlignmentRequested,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GridLanesBaselineInput {
    pub auto_flow: LaneAutoFlow,
    pub queried_axis: GridAxis,
    pub requested_alignment: BaselineAlignment,
    pub has_items: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GridLanesBaselinePolicyReport {
    pub applies_item_offsets: bool,
    pub reason: Option<GridLanesBaselineReason>,
}
```

Expected behavior:

- Match WebKit's masonry fallback: do not apply item baseline offsets for row-lanes or column-lanes in either baseline-offset entry point.
- Keep the reason explicit in the report.
- `auto_flow` remains in `GridLanesBaselineInput` so tests can cover row-lanes and column-lanes even though both currently return `WebKitMasonryFallback`.
- Reuse container fallback item geometry to synthesize first/last container baselines after final lane geometry exists.

- [ ] Run:

```bash
cargo test -p surgeist --test oracle oracle_grid_lanes
cargo fmt --check
```

- [ ] Commit:

```bash
git status --short --branch
git diff --check
git add crates/surgeist/tests/support/oracle/grid/baseline.rs crates/surgeist/tests/support/oracle/grid/lanes.rs crates/surgeist/tests/support/oracle/grid/mod.rs crates/surgeist/tests/oracle.rs
git commit -m "Add grid-lanes baseline oracle"
```

---

## Task 9: Compose Baseline Reports In Oracle Scenarios

**Files:**
- Modify `crates/surgeist/tests/support/oracle/grid/scenario.rs`
- Modify `crates/surgeist/tests/support/oracle/grid/mod.rs`
- Modify `crates/surgeist/tests/oracle.rs`
- Modify `crates/surgeist/tests/layout_oracle.rs`

- [ ] Add failing composed oracle scenario tests.

```rust
#[test]
fn oracle_scenario_offsets_grid_items_by_baseline_report() {
    let rect = support::oracle::grid::compose_baseline_aligned_item_rect(
        support::oracle::grid::BaselineAlignedItemRectInput {
            area_x: 0.0,
            area_y: 0.0,
            area_width: 50.0,
            area_height: 40.0,
            item_width: 20.0,
            item_height: 30.0,
            normal_x_offset: 0.0,
            normal_y_offset: 0.0,
            baseline_y_offset: Some(6.0),
        },
    );

    assert_eq!(rect, support::oracle::grid::GridItemRect::new(0.0, 6.0, 20.0, 30.0));
}
```

- [ ] Run:

```bash
cargo test -p surgeist --test oracle oracle_scenario_offsets_grid_items_by_baseline_report
```

Expected: fail because `compose_baseline_aligned_item_rect` does not exist.

- [ ] Implement a thin scenario helper only.

Expected behavior:

- Accept explicit normal offsets and an optional baseline y-offset.
- Return a `GridItemRect`.
- Do not compute baseline groups in `scenario.rs`.
- Do not infer baselines from item sizes in `scenario.rs`.

- [ ] Add one `layout_oracle.rs` comparison test after production exposes a comparable pure function or existing production helper. If production does not yet expose such a helper, add an ignored test with the exact command and unblock condition:

```rust
#[ignore = "enable after production baseline helper exists"]
#[test]
fn layout_oracle_grid_baseline_offset_matches_oracle() {
    /* compare production baseline offset helper to oracle::grid::baseline_offset */
}
```

- [ ] Run:

```bash
cargo test -p surgeist --test oracle oracle_scenario_offsets_grid_items_by_baseline_report
cargo test -p surgeist --test layout_oracle
cargo fmt --check
```

- [ ] Commit:

```bash
git status --short --branch
git diff --check
git add crates/surgeist/tests/support/oracle/grid/scenario.rs crates/surgeist/tests/support/oracle/grid/mod.rs crates/surgeist/tests/oracle.rs crates/surgeist/tests/layout_oracle.rs
git commit -m "Compose baseline oracle scenarios"
```

---

## Task 10: Verification Before Engine Work

**Files:**
- No source files expected unless verification finds issues.

- [ ] Run focused oracle checks:

```bash
cargo test -p surgeist --test oracle oracle_baseline
cargo test -p surgeist --test oracle oracle_subgrid_baseline
cargo test -p surgeist --test oracle oracle_grid_lanes
cargo test -p surgeist --test layout_oracle
```

- [ ] Run broad checks:

```bash
cargo test -p surgeist --test oracle
cargo test -p surgeist
cargo fmt --check
```

- [ ] If verification finds issues, fix the smallest relevant oracle slice and rerun the failing command plus the nearest broad oracle command.

- [ ] Commit verification fixes if any:

```bash
git status --short --branch
git diff --check
git add crates/surgeist/tests/support/oracle/grid/baseline.rs crates/surgeist/tests/oracle.rs crates/surgeist/tests/layout_oracle.rs
git commit -m "Fix baseline oracle verification"
```

Adjust the `git add` path list to the exact files changed by verification fixes before committing.

---

## Clean-Context Review Requirement

Before marking this goal complete:

- [ ] Dispatch a clean-context reviewer with this plan, the existing oracle spec, the existing subgrid/grid-lanes oracle plan, the engine baseline plan, and the WebKit/Blink reference list.
- [ ] Ask the reviewer to check:
  - Whether this oracle plan genuinely precedes and supports the engine baseline plan.
  - Whether baseline facts remain explicit and the oracle avoids becoming a production layout engine.
  - Whether WebKit remains the primary behavioral reference and Blink is used only as a formula/data-shape cross-check.
  - Whether subgrid baseline inheritance and publication are specific enough.
  - Whether grid-lanes fallback matches the current WebKit masonry behavior.
  - Whether all tests and commands are executable.
- [ ] Implement every accepted recommendation in this plan before marking the goal complete.
- [ ] If a recommendation is rejected, record the technical reason in a short "Reviewer Notes" section at the bottom of this file.

---

## Self-Review

- Spec coverage: The plan honors the existing spec's rule that baseline facts may be explicit inputs and baseline caches must stay out of the oracle.
- File shape: The plan adds `baseline.rs` without collapsing `alignment.rs`, `subgrid.rs`, `lanes.rs`, or `scenario.rs`.
- WebKit/Blink use: WebKit is the primary source for behavior; Blink is used for formula clarity where compatible.
- Placeholder scan: Tasks include concrete tests, expected structures, commands, and commit points.

## Clean-Context Review Notes

- Reviewer: Ampere (`019ed403-48b4-7d81-95c4-1a05293ae3b0`).
- Accepted recommendation: Container baseline selection now covers first-minor fallback, last-major fallback, occupied-row tracking, grid-order fallback, and explicit synthesized fallback coordinates.
- Accepted recommendation: Subgrid baseline inheritance now uses `OracleGapReport` and reports sliced, reversed, MBP-adjusted, and final gap-adjusted coordinate stages.
- Accepted recommendation: Subgrid descendant publication now uses explicit local track offset and baseline-in-track facts, including a reversed-index test.
- Accepted recommendation: Grid-lanes baseline policy now disables item baseline offsets for both row-lanes and column-lanes while preserving container baseline synthesis from final geometry.
- Accepted recommendation: Synthesized subgrid baselines that require unavailable subgrid layout are now an explicit participation fallback fact.
