# Surgeist Layout Oracle Tests Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a complete layout-facing grid oracle test layer that compares `surgeist::layout::compute_grid` output against oracle-computed expected geometry for the major grid behavior categories covered by parity XML.

**Architecture:** Keep the oracle independent from production layout by computing expected placement, track sizing, alignment, and scenario rects in `tests/support/oracle/grid`. Add a layout-test harness that consumes oracle reports, builds an `OracleTree`, runs layout, and compares root size plus child rects. Keep parser/XML/style concerns out of this layer.

**Tech Stack:** Rust integration tests, `surgeist::layout`, existing `OracleTree`, existing grid oracle modules.

---

### Task 1: Add Layout Oracle Harness

**Files:**
- Create: `crates/surgeist/tests/support/oracle/grid/layout.rs`
- Modify: `crates/surgeist/tests/support/oracle/grid/mod.rs`

- [ ] **Step 1: Add a failing layout-oracle smoke test**

Create `crates/surgeist/tests/layout_oracle.rs` with one fixed-track test that imports a non-existent `GridLayoutOracle` helper:

```rust
mod support;

use support::oracle::grid::{GridArea, GridLayoutOracle, GridTrack, TrackSizing};
use surgeist::layout::{Dimension, Size, TrackComponent};

#[test]
fn oracle_layout_fixed_tracks_match_layout_child_rects() {
    let expected_columns = TrackSizing::definite(210.0, 10.0)
        .track(GridTrack::fixed(80.0))
        .track(GridTrack::fixed(120.0))
        .solve();
    let expected_rows = TrackSizing::definite(40.0, 0.0)
        .track(GridTrack::fixed(40.0))
        .solve();

    GridLayoutOracle::new()
        .container(Size::new(210.0, 40.0))
        .columns(vec![TrackComponent::px(80.0), TrackComponent::px(120.0)])
        .rows(vec![TrackComponent::px(40.0)])
        .gap(Size::new(10.0, 0.0))
        .expected_tracks(expected_columns, expected_rows)
        .child(GridArea::new(1, 1, 1, 1))
        .child(GridArea::new(2, 1, 1, 1))
        .assert_layout();
}
```

- [ ] **Step 2: Run test to verify failure**

Run:

```bash
cargo test -p surgeist --test layout_oracle oracle_layout_fixed_tracks_match_layout_child_rects
```

Expected: compile failure because `GridLayoutOracle` does not exist.

- [ ] **Step 3: Implement the harness**

Add `layout.rs` with:

```rust
use super::{
    AlignmentSafety, GridArea, GridItemRect, GridScenarioReport, TrackAlignment,
    TrackSizingReport, align_tracks_report, compose_grid_scenario,
};
use crate::support::oracle_tree::OracleTree;
use surgeist::layout::{
    AlignContent, Available, ComputeInput, ComputeOutput, Dimension, Display, Edges,
    GridAutoFlow, GridPlacement, Length, NodeInput, Point, RequestedAxis, RunMode, Size,
    SizingMode, TrackComponent, compute_grid,
};

#[derive(Clone, Debug)]
pub struct GridLayoutOracle {
    container: Size<f32>,
    columns: Vec<TrackComponent>,
    rows: Vec<TrackComponent>,
    gap: Size<f32>,
    justify_content: AlignContent,
    align_content: AlignContent,
    expected_columns: Option<TrackSizingReport>,
    expected_rows: Option<TrackSizingReport>,
    children: Vec<GridLayoutChild>,
}

#[derive(Clone, Copy, Debug)]
struct GridLayoutChild {
    area: GridArea,
    measurement: Size<f32>,
}

impl Default for GridLayoutOracle {
    fn default() -> Self {
        Self {
            container: Size::new(0.0, 0.0),
            columns: Vec::new(),
            rows: Vec::new(),
            gap: Size::new(0.0, 0.0),
            justify_content: AlignContent::Start,
            align_content: AlignContent::Start,
            expected_columns: None,
            expected_rows: None,
            children: Vec::new(),
        }
    }
}

impl GridLayoutOracle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn container(mut self, container: Size<f32>) -> Self {
        self.container = container;
        self
    }

    pub fn columns(mut self, columns: Vec<TrackComponent>) -> Self {
        self.columns = columns;
        self
    }

    pub fn rows(mut self, rows: Vec<TrackComponent>) -> Self {
        self.rows = rows;
        self
    }

    pub fn gap(mut self, gap: Size<f32>) -> Self {
        self.gap = gap;
        self
    }

    pub fn justify_content(mut self, justify_content: AlignContent) -> Self {
        self.justify_content = justify_content;
        self
    }

    pub fn align_content(mut self, align_content: AlignContent) -> Self {
        self.align_content = align_content;
        self
    }

    pub fn expected_tracks(
        mut self,
        columns: TrackSizingReport,
        rows: TrackSizingReport,
    ) -> Self {
        self.expected_columns = Some(columns);
        self.expected_rows = Some(rows);
        self
    }

    pub fn child(mut self, area: GridArea) -> Self {
        self.children.push(GridLayoutChild {
            area,
            measurement: Size::new(0.0, 0.0),
        });
        self
    }

    pub fn measured_child(mut self, area: GridArea, measurement: Size<f32>) -> Self {
        self.children.push(GridLayoutChild { area, measurement });
        self
    }

    pub fn assert_layout(self) {
        let expected_columns = self
            .expected_columns
            .clone()
            .expect("expected columns must be supplied");
        let expected_rows = self
            .expected_rows
            .clone()
            .expect("expected rows must be supplied");
        let scenario = self.expected_scenario(expected_columns, expected_rows);
        let mut tree = self.tree();

        let output = compute_grid(
            &mut tree,
            1,
            ComputeInput {
                run_mode: RunMode::PerformLayout,
                sizing_mode: SizingMode::InherentSize,
                axis: RequestedAxis::Both,
                known: Size::NONE,
                parent: Size::new(Some(self.container.width), Some(self.container.height)),
                available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
            },
        );

        assert_size_close(output.size, self.container);
        for (index, expected) in scenario.item_rects.iter().enumerate() {
            let node = (index + 2) as u32;
            let actual = tree.layout(node).expect("child layout must be recorded");
            assert_rect_close(node, actual.location, actual.size, *expected);
        }
    }

    fn expected_scenario(
        &self,
        expected_columns: TrackSizingReport,
        expected_rows: TrackSizingReport,
    ) -> GridScenarioReport {
        let placement = super::PlacementReport {
            areas: self.children.iter().map(|child| child.area).collect(),
            implicit_columns_after: 0,
            implicit_rows_after: 0,
            cursor: super::PlacementCursor { column: 1, row: 1 },
        };
        let column_alignment = align_tracks_report(
            self.container.width,
            expected_columns
                .final_tracks
                .iter()
                .map(|track| track.size)
                .collect(),
            self.gap.width,
            track_alignment(self.justify_content),
            alignment_safety(self.justify_content),
        );
        let row_alignment = align_tracks_report(
            self.container.height,
            expected_rows
                .final_tracks
                .iter()
                .map(|track| track.size)
                .collect(),
            self.gap.height,
            track_alignment(self.align_content),
            alignment_safety(self.align_content),
        );

        compose_grid_scenario(
            placement,
            expected_columns,
            expected_rows,
            column_alignment,
            row_alignment,
        )
    }

    fn tree(&self) -> OracleTree {
        let child_nodes = (0..self.children.len()).map(|index| (index + 2) as u32);
        let mut tree = OracleTree::new().children(1, child_nodes);
        tree = tree.style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(
                    Dimension::px(self.container.width),
                    Dimension::px(self.container.height),
                ),
                grid_template_columns: self.columns.clone(),
                grid_template_rows: self.rows.clone(),
                justify_content: Some(self.justify_content),
                align_content: Some(self.align_content),
                gap: Size::new(Length::px(self.gap.width), Length::px(self.gap.height)),
                ..NodeInput::default()
            },
        );

        for (index, child) in self.children.iter().enumerate() {
            let node = (index + 2) as u32;
            tree = tree.children(node, []).style(
                node,
                NodeInput {
                    grid_column: GridPlacement::line_span(
                        child.area.column_start as isize,
                        child.area.column_span,
                    ),
                    grid_row: GridPlacement::line_span(
                        child.area.row_start as isize,
                        child.area.row_span,
                    ),
                    ..NodeInput::default()
                },
            );
            tree = tree.measure(
                node,
                ComputeOutput::from_sizes(child.measurement, child.measurement),
            );
        }

        tree
    }
}

fn track_alignment(alignment: AlignContent) -> TrackAlignment {
    match alignment {
        AlignContent::Start | AlignContent::FlexStart | AlignContent::Stretch => {
            TrackAlignment::Start
        }
        AlignContent::End | AlignContent::FlexEnd | AlignContent::SafeEnd
        | AlignContent::SafeFlexEnd => TrackAlignment::End,
        AlignContent::Center | AlignContent::SafeCenter => TrackAlignment::Center,
        AlignContent::SpaceBetween => TrackAlignment::SpaceBetween,
        AlignContent::SpaceAround => TrackAlignment::SpaceAround,
        AlignContent::SpaceEvenly => TrackAlignment::SpaceEvenly,
    }
}

fn alignment_safety(alignment: AlignContent) -> AlignmentSafety {
    match alignment {
        AlignContent::SafeEnd | AlignContent::SafeFlexEnd | AlignContent::SafeCenter => {
            AlignmentSafety::Safe
        }
        _ => AlignmentSafety::Unsafe,
    }
}

fn assert_rect_close(node: u32, location: Point<f32>, size: Size<f32>, expected: GridItemRect) {
    assert_close(location.x, expected.x, "node {node} x");
    assert_close(location.y, expected.y, "node {node} y");
    assert_close(size.width, expected.width, "node {node} width");
    assert_close(size.height, expected.height, "node {node} height");
}

fn assert_size_close(actual: Size<f32>, expected: Size<f32>) {
    assert_close(actual.width, expected.width, "root width");
    assert_close(actual.height, expected.height, "root height");
}

fn assert_close(actual: f32, expected: f32, label: &str) {
    assert!(
        (actual - expected).abs() <= 0.000_1,
        "{label}: expected {expected}, got {actual}"
    );
}
```

Export it from `grid/mod.rs`:

```rust
pub mod layout;
pub use layout::GridLayoutOracle;
```

- [ ] **Step 4: Run test to verify it passes**

Run:

```bash
cargo test -p surgeist --test layout_oracle oracle_layout_fixed_tracks_match_layout_child_rects
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/surgeist/tests/layout_oracle.rs crates/surgeist/tests/support/oracle/grid/layout.rs crates/surgeist/tests/support/oracle/grid/mod.rs
git commit -m "Add grid layout oracle harness"
```

### Task 2: Cover Core Grid Track Categories

**Files:**
- Modify: `crates/surgeist/tests/layout_oracle.rs`

- [ ] **Step 1: Add tests for fixed/gap, percent, flex, sub-one flex, minmax, and stretch**
- [ ] **Step 2: Run `cargo test -p surgeist --test layout_oracle` and fix mismatches**
- [ ] **Step 3: Commit with `git commit -m "Add grid oracle track layout tests"`**

### Task 3: Cover Placement And Alignment Categories

**Files:**
- Modify: `crates/surgeist/tests/layout_oracle.rs`

- [ ] **Step 1: Add tests for line placement, spans, auto-placement, center/end/space distribution, and safe fallback**
- [ ] **Step 2: Run `cargo test -p surgeist --test layout_oracle` and fix mismatches**
- [ ] **Step 3: Commit with `git commit -m "Add grid oracle placement alignment tests"`**

### Task 4: Cover Intrinsic Contribution Categories

**Files:**
- Modify: `crates/surgeist/tests/layout_oracle.rs`
- Modify: `crates/surgeist/tests/support/oracle/grid/layout.rs` if child measurements need expected input assertions

- [ ] **Step 1: Add tests for auto track intrinsic sizing, spanning intrinsic sizing, fit-content clamping, and row sizing from resolved columns**
- [ ] **Step 2: Run `cargo test -p surgeist --test layout_oracle` and fix mismatches**
- [ ] **Step 3: Commit with `git commit -m "Add grid oracle intrinsic layout tests"`**

### Task 5: Final Verification And Audit

**Files:**
- No expected code edits

- [ ] **Step 1: Run focused tests**

```bash
cargo test -p surgeist --test layout_oracle
cargo test -p surgeist --test oracle
```

- [ ] **Step 2: Run full crate tests**

```bash
cargo fmt --check
cargo test -p surgeist
```

- [ ] **Step 3: Audit oracle independence**

```bash
rg "compute_grid|compute_leaf|compute_block|compute_flex|surgeist::layout::grid|css|html|xml" crates/surgeist/tests/support/oracle/grid -n
```

Expected: matches only in `grid/layout.rs`, because that file is the layout-facing adapter for tests; pure oracle phase modules must stay free of production layout.

- [ ] **Step 4: Commit any verification-only cleanup**

```bash
git status --short --branch
```

Expected: clean worktree.
