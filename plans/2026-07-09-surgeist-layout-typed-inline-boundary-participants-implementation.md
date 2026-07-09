# Typed Inline Boundary Participants Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement layout-ready typed inline start/end participants so anonymous inline wrapper boundaries can affect line construction without layout synthesizing DOM, CSS, or text state.

**Architecture:** `surgeist-layout` will expose a public scalar-generic `InlineBoundaryInputOf<S>` and `InlineBoundaryKind`, accept them through `LayoutInputOf<S>`, and lower them through the existing block inline-run path into typed inline stream participants. The inline engine will treat boundary participants as zero-advance, metric-bearing controls that do not force breaks, do not affect intrinsic widths, and work in horizontal and vertical writing modes.

**Tech Stack:** Rust 2024, `surgeist-layout`, crate-local unit tests, crate-local browser parity support only where layout-ready metadata is consumed, `guidance/surgeist-rust-modeling-guide.md`.

---

## Source References

- Modeling guidance: `guidance/surgeist-rust-modeling-guide.md`
- Current inline contract spec: `plans/specs/2026-07-09-surgeist-layout-mixed-inline-participant-contract-spec.md`
- Current inline runtime: `src/inline.rs`
- Current block inline-run lowering: `src/block.rs`
- Current layout-ready input API: `src/node_input.rs`, `src/lib.rs`, `src/compute.rs`
- Existing tests: `src/contract_tests.rs`, `src/inline_tests.rs`, `src/block_tests.rs`
- WebKit reference in this repo:
  - `tmp/WebKit/Source/WebCore/layout/formattingContexts/inline/InlineItem.h` defines `InlineBoxStart` and `InlineBoxEnd` item types.
  - `tmp/WebKit/Source/WebCore/layout/formattingContexts/inline/InlineItemsBuilder.cpp` emits start/end items around inline boxes.
  - `tmp/WebKit/Source/WebCore/rendering/RenderElement.cpp` propagates parent style to anonymous children.
  - `tmp/WebKit/Source/WebCore/style/computed/StyleComputedStyle.cpp` creates anonymous styles by inheriting from the parent and preserving display.
  - `tmp/WebKit/Source/WebCore/layout/layouttree/LayoutTreeBuilder.cpp` builds line-break and anonymous inline/text boxes with computed style.

## Boundary Decisions

Layout owns:

- typed boundary inputs once retained/style/root have decided a layout-relevant inline wrapper boundary exists;
- validation that boundary inputs are layout-ready and coherent with the parent inline flow;
- metric contribution, baseline aggregation, logical-to-physical placement, and output reporting for boundary participants.

Layout does not own:

- deciding which DOM/style wrappers need a boundary participant;
- CSS cascade, inheritance, anonymous wrapper synthesis, text shaping, bidi segmentation, or raw text measurement;
- preserving compatibility aliases for old internal names while this private inline engine is refactored.

This plan intentionally implements the boundary participant model now. Measured text remains outside this plan because typed start/end boundary participants are independently useful for atomic boxes, forced line breaks, and root-owned measured text fragments.

## File Map

- Modify `src/node_input.rs`
  - Add public `InlineBoundaryKind`.
  - Add public scalar-generic `InlineBoundaryInputOf<S>` with private fields and constructors/getters.
  - Add `InlineBoundaryInput` alias.
  - Add `LayoutInputOf::InlineBoundary`, constructor, and accessor.
- Modify `src/lib.rs`
  - Re-export the new public boundary input types.
- Modify `src/compute.rs`
  - Treat hidden inline boundary nodes like hidden line-break nodes.
- Modify `src/inline.rs`
  - Rename the private inline-run model away from "atomic only".
  - Add `InlineBoundaryControlOf<S>`.
  - Add `InlineParticipant<S>::Boundary`.
  - Add layout report kinds for `InlineBoundaryStart` and `InlineBoundaryEnd`.
  - Make boundary participants zero-advance, metric-bearing, non-breaking participants in horizontal and vertical paths.
  - Exclude boundaries from intrinsic inline-size calculations.
- Modify `src/block.rs`
  - Accept visible `LayoutInputOf::InlineBoundary` children inside inline runs.
  - Validate boundary writing mode and direction against the parent flow.
  - Lower boundaries through one named conversion path into inline participants.
  - Report zero-size boundary outputs without computing them as boxes.
- Modify `src/test_support/layout_tree.rs`
  - Add an `OracleTreeOf<S>::inline_boundary(...)` builder used by block tests.
- Modify `src/contract_tests.rs`
  - Add public API tests for boundary input construction, `f64`, and layout input discrimination.
- Modify `src/inline_tests.rs`
  - Add inline engine tests for horizontal metrics, vertical metrics, zero advance, intrinsic widths, and output item kinds.
- Modify `src/block_tests.rs`
  - Add block-level integration tests for boundary nodes around inline-level children, hidden layout, and vertical flow.
- Modify `plans/specs/2026-07-09-surgeist-layout-mixed-inline-participant-contract-spec.md`
  - Replace the old "boundaries only if proven separately" language with the implemented typed boundary participant contract.

---

## Task 1: Add Public Layout-Ready Boundary Input

**Files:**
- Modify: `src/node_input.rs`
- Modify: `src/lib.rs`
- Modify: `src/compute.rs`
- Test: `src/contract_tests.rs`

- [ ] **Step 1: Write public API tests first**

Add these tests near the existing `LineBreakInput` and `LayoutInput` contract tests in `src/contract_tests.rs`:

```rust
#[test]
fn inline_boundary_input_requires_explicit_metrics() {
    let metrics = InlineMetrics::from_line_height_and_baseline(28.0, 20.0).unwrap();
    let input = InlineBoundaryInput::new(InlineBoundaryKind::Start, metrics)
        .with_writing_mode(WritingMode::VerticalRl)
        .with_direction(Direction::Rtl)
        .with_vertical_align(VerticalAlign::Top);

    assert_eq!(input.kind(), InlineBoundaryKind::Start);
    assert_eq!(input.metrics(), metrics);
    assert_eq!(input.writing_mode(), WritingMode::VerticalRl);
    assert_eq!(input.direction(), Direction::Rtl);
    assert_eq!(input.vertical_align(), VerticalAlign::Top);
}

#[test]
fn inline_boundary_input_supports_f64_metrics() {
    let metrics = InlineMetricsOf::<f64>::from_line_height_and_baseline(40.0, 30.0).unwrap();
    let input = InlineBoundaryInputOf::<f64>::new(InlineBoundaryKind::End, metrics);

    assert_eq!(input.kind(), InlineBoundaryKind::End);
    assert_eq!(input.metrics().line_extent(), 40.0);
    assert_eq!(input.metrics().baseline(), 30.0);
}

#[test]
fn layout_input_distinguishes_inline_boundary_from_boxes_and_breaks() {
    let metrics = InlineMetrics::from_line_height_and_baseline(18.0, 14.0).unwrap();
    let boundary = InlineBoundaryInput::new(InlineBoundaryKind::Start, metrics);
    let layout_input = LayoutInput::inline_boundary(boundary);

    assert!(layout_input.as_box().is_none());
    assert!(layout_input.as_line_break().is_none());
    assert_eq!(layout_input.as_inline_boundary(), Some(boundary));
}
```

- [ ] **Step 2: Run the focused API tests and verify they fail**

Run:

```sh
cargo test -p surgeist-layout inline_boundary_input -- --nocapture
```

Expected: compile failure naming missing `InlineBoundaryInput`, `InlineBoundaryInputOf`, `InlineBoundaryKind`, `LayoutInputOf::inline_boundary`, or `LayoutInputOf::as_inline_boundary`.

- [ ] **Step 3: Add `InlineBoundaryKind` and `InlineBoundaryInputOf<S>`**

In `src/node_input.rs`, after `LineBreakInputOf<S>` and before `LayoutInputOf<S>`, add:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InlineBoundaryKind {
    Start,
    End,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InlineBoundaryInputOf<S: LayoutScalar = DefaultScalar> {
    kind: InlineBoundaryKind,
    writing_mode: WritingMode,
    direction: Direction,
    vertical_align: VerticalAlign,
    metrics: InlineMetricsOf<S>,
}

pub type InlineBoundaryInput = InlineBoundaryInputOf<DefaultScalar>;

impl<S: LayoutScalar> InlineBoundaryInputOf<S> {
    #[must_use]
    pub const fn new(kind: InlineBoundaryKind, metrics: InlineMetricsOf<S>) -> Self {
        Self {
            kind,
            writing_mode: WritingMode::HorizontalTb,
            direction: Direction::Ltr,
            vertical_align: VerticalAlign::Baseline,
            metrics,
        }
    }

    #[must_use]
    pub const fn with_writing_mode(mut self, writing_mode: WritingMode) -> Self {
        self.writing_mode = writing_mode;
        self
    }

    #[must_use]
    pub const fn with_direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    #[must_use]
    pub const fn with_vertical_align(mut self, vertical_align: VerticalAlign) -> Self {
        self.vertical_align = vertical_align;
        self
    }

    #[must_use]
    pub const fn kind(self) -> InlineBoundaryKind {
        self.kind
    }

    #[must_use]
    pub const fn writing_mode(self) -> WritingMode {
        self.writing_mode
    }

    #[must_use]
    pub const fn direction(self) -> Direction {
        self.direction
    }

    #[must_use]
    pub const fn vertical_align(self) -> VerticalAlign {
        self.vertical_align
    }

    #[must_use]
    pub const fn metrics(self) -> InlineMetricsOf<S> {
        self.metrics
    }
}
```

- [ ] **Step 4: Extend `LayoutInputOf<S>`**

Replace the `LayoutInputOf<S>` declaration and impl match arms in `src/node_input.rs` with this shape:

```rust
#[derive(Clone, Debug, PartialEq)]
pub enum LayoutInputOf<S: LayoutScalar = DefaultScalar> {
    Box(std::boxed::Box<NodeInputOf<S>>),
    LineBreak(LineBreakInputOf<S>),
    InlineBoundary(InlineBoundaryInputOf<S>),
}
```

Add this constructor and accessor:

```rust
#[must_use]
pub const fn inline_boundary(input: InlineBoundaryInputOf<S>) -> Self {
    Self::InlineBoundary(input)
}

#[must_use]
pub const fn as_inline_boundary(&self) -> Option<InlineBoundaryInputOf<S>> {
    match self {
        Self::Box(_) | Self::LineBreak(_) => None,
        Self::InlineBoundary(input) => Some(*input),
    }
}
```

Update the existing accessors:

```rust
pub fn as_box(&self) -> Option<&NodeInputOf<S>> {
    match self {
        Self::Box(input) => Some(input.as_ref()),
        Self::LineBreak(_) | Self::InlineBoundary(_) => None,
    }
}

pub const fn as_line_break(&self) -> Option<LineBreakInputOf<S>> {
    match self {
        Self::Box(_) | Self::InlineBoundary(_) => None,
        Self::LineBreak(input) => Some(*input),
    }
}
```

- [ ] **Step 5: Re-export the new public types**

In `src/lib.rs`, add `InlineBoundaryInput`, `InlineBoundaryInputOf`, and `InlineBoundaryKind` to the `pub use node_input::{ ... }` list:

```rust
InlineBoundaryInput, InlineBoundaryInputOf, InlineBoundaryKind, InlineMetrics,
```

- [ ] **Step 6: Handle hidden boundary nodes**

In `src/compute.rs`, update the hidden-layout child match:

```rust
match tree.layout_input(child) {
    LayoutInputOf::Box(_) => {
        tree.compute_child(child, ComputeInputOf::HIDDEN);
    }
    LayoutInputOf::LineBreak(_) | LayoutInputOf::InlineBoundary(_) => {
        tree.cache_clear(child);
        tree.set_unrounded(child, NodeOutputOf::with_order(0));
    }
}
```

- [ ] **Step 7: Run the focused API tests**

Run:

```sh
cargo test -p surgeist-layout inline_boundary_input -- --nocapture
cargo test -p surgeist-layout layout_input_distinguishes_inline_boundary -- --nocapture
```

Expected: all selected tests pass.

- [ ] **Step 8: Assign scoped review for Task 1**

Assign a separate reviewer after the worker reports tests and git status. The reviewer must inspect only Task 1 changes in `src/node_input.rs`, `src/lib.rs`, `src/compute.rs`, and `src/contract_tests.rs`; confirm the public input is layout-ready, scalar-generic, has private fields, requires explicit metrics, and does not add DOM/CSS/text concepts.

- [ ] **Step 9: Commit Task 1 after review is clean**

```sh
git add src/node_input.rs src/lib.rs src/compute.rs src/contract_tests.rs
git commit -m "Add inline boundary layout input"
```

---

## Task 2: Refactor Inline Runtime Names And Add Boundary Controls

**Files:**
- Modify: `src/inline.rs`
- Modify: `src/block.rs`
- Modify: `src/inline_tests.rs`

- [ ] **Step 1: Rename the private inline stream model**

In `src/inline.rs`, rename the private runtime types and functions as follows. These are crate-private names, so do not keep compatibility aliases.

```text
AtomicInlineInput -> InlineRunInput
AtomicInlineItem -> InlineParticipant
AtomicInlineBoxItem -> AtomicInlineBoxParticipant
AtomicInlineLayoutItemKind -> InlineParticipantLayoutKind
AtomicInlineLayoutItem -> InlineParticipantLayoutItem
AtomicInlineReport -> InlineRunReport
layout_atomic_inline_items -> layout_inline_run
layout_vertical_atomic_inline_items -> layout_vertical_inline_run
atomic_inline_min_content_width -> inline_run_min_content_width
atomic_inline_max_content_width -> inline_run_max_content_width
AtomicInlineRunContext -> InlineRunContext
AtomicInlineSegmentsContext -> InlineSegmentsContext
AtomicInlineRunChild -> InlineRunChild
AtomicInlineClearCandidate -> InlineClearCandidate
atomic_inline_run_end -> inline_run_end
next_atomic_inline_clear_candidate -> next_inline_clear_candidate
atomic_inline_run_contains_clear -> inline_run_contains_clear
layout_atomic_inline_segments -> layout_inline_segments
layout_atomic_inline_run_with_clear -> layout_inline_run_with_clear
layout_atomic_inline_run -> layout_inline_run_children
```

Apply matching import and call-site updates in `src/block.rs` and `src/inline_tests.rs`. Keep public oracle test-support names unchanged when they live under `tests/support/oracle` or browser parity support; this task is only the production inline runtime naming. After this rename, `src/inline.rs` owns `layout_inline_run(...)` for stream layout, and `src/block.rs` owns `layout_inline_run_children(...)` for collecting child nodes and applying their outputs.

- [ ] **Step 2: Run the renamed focused tests**

Run:

```sh
cargo test -p surgeist-layout inline_ -- --nocapture
```

Expected: compile succeeds far enough to run tests. Failures are acceptable only where the following steps add boundary behavior. Any unresolved old production name from Step 1 must be fixed before continuing.

- [ ] **Step 3: Add the boundary control model**

In `src/inline.rs`, import the public boundary kind:

```rust
use super::{
    AvailableOf, Clear, DefaultScalar, Direction, Edges, InlineBoundaryKind, InlineMetricsOf,
    LayoutScalar, Point, Size, VerticalAlign, WritingMode,
};
```

Add this struct after `ForcedLineBreakControlOf<S>`:

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct InlineBoundaryControlOf<S: LayoutScalar = DefaultScalar> {
    order: u32,
    kind: InlineBoundaryKind,
    flow: InlineFlowOf<S>,
    metrics: InlineMetricsOf<S>,
    alignment: InlineControlAlignment,
}

impl<S: LayoutScalar> InlineBoundaryControlOf<S> {
    #[must_use]
    pub(super) const fn new(
        order: u32,
        kind: InlineBoundaryKind,
        flow: InlineFlowOf<S>,
        metrics: InlineMetricsOf<S>,
        alignment: InlineControlAlignment,
    ) -> Self {
        Self {
            order,
            kind,
            flow,
            metrics,
            alignment,
        }
    }

    #[must_use]
    pub(super) const fn order(self) -> u32 {
        self.order
    }

    #[must_use]
    pub(super) const fn kind(self) -> InlineBoundaryKind {
        self.kind
    }

    #[allow(dead_code)]
    #[must_use]
    pub(super) const fn flow(self) -> InlineFlowOf<S> {
        self.flow
    }

    #[must_use]
    pub(super) const fn metrics(self) -> InlineMetricsOf<S> {
        self.metrics
    }

    #[allow(dead_code)]
    #[must_use]
    pub(super) const fn alignment(self) -> InlineControlAlignment {
        self.alignment
    }
}
```

Extend the control and participant enums:

```rust
pub(super) enum InlineControlItemOf<S: LayoutScalar = DefaultScalar> {
    ForcedLineBreak(ForcedLineBreakControlOf<S>),
    Boundary(InlineBoundaryControlOf<S>),
}

pub(super) enum InlineParticipant<S: LayoutScalar = DefaultScalar> {
    AtomicBox(AtomicInlineBoxParticipant<S>),
    ForcedLineBreak(ForcedLineBreakControlOf<S>),
    Boundary(InlineBoundaryControlOf<S>),
}
```

Add the constructor:

```rust
impl<S: LayoutScalar> InlineParticipant<S> {
    #[allow(dead_code)]
    #[must_use]
    pub(super) const fn inline_boundary(control: InlineBoundaryControlOf<S>) -> Self {
        Self::Boundary(control)
    }
}
```

- [ ] **Step 4: Add control-preservation tests**

In `src/inline_tests.rs`, add:

```rust
fn inline_boundary_for(
    order: u32,
    kind: InlineBoundaryKind,
    writing_mode: WritingMode,
    direction: Direction,
    metrics: InlineMetrics,
) -> InlineParticipant {
    InlineParticipant::inline_boundary(crate::inline::InlineBoundaryControlOf::new(
        order,
        kind,
        crate::inline::InlineFlowOf::new(writing_mode, direction, Available::MAX_CONTENT),
        metrics,
        crate::inline::InlineControlAlignment::Baseline,
    ))
}

#[test]
fn inline_boundary_control_preserves_layout_ready_fields() {
    let metrics = InlineMetrics::from_line_height_and_baseline(24.0, 18.0).unwrap();
    let control = crate::inline::InlineBoundaryControlOf::new(
        9,
        InlineBoundaryKind::End,
        crate::inline::InlineFlowOf::new(
            WritingMode::VerticalLr,
            Direction::Rtl,
            Available::Definite(300.0),
        ),
        metrics,
        crate::inline::InlineControlAlignment::Top,
    );

    assert_eq!(control.order(), 9);
    assert_eq!(control.kind(), InlineBoundaryKind::End);
    assert_eq!(control.flow().writing_mode(), WritingMode::VerticalLr);
    assert_eq!(control.flow().direction(), Direction::Rtl);
    assert_eq!(control.flow().available_inline_extent(), Available::Definite(300.0));
    assert_eq!(control.metrics(), metrics);
    assert_eq!(control.alignment(), crate::inline::InlineControlAlignment::Top);
}
```

- [ ] **Step 5: Run the focused control test**

Run:

```sh
cargo test -p surgeist-layout inline_boundary_control_preserves_layout_ready_fields -- --nocapture
```

Expected: pass.

- [ ] **Step 6: Assign scoped review for Task 2**

Assign a separate reviewer after the worker reports tests and git status. The reviewer must inspect only Task 2 changes in `src/inline.rs`, `src/block.rs`, and `src/inline_tests.rs`; confirm internal runtime names no longer imply an atomic-only stream, no compatibility aliases were added, and `InlineBoundaryControlOf<S>` carries only layout-ready data.

- [ ] **Step 7: Commit Task 2 after review is clean**

```sh
git add src/inline.rs src/block.rs src/inline_tests.rs
git commit -m "Model inline boundary participants"
```

---

## Task 3: Implement Boundary Layout Semantics In Inline Runs

**Files:**
- Modify: `src/inline.rs`
- Test: `src/inline_tests.rs`

- [ ] **Step 1: Add failing horizontal and intrinsic tests**

In `src/inline_tests.rs`, add:

```rust
#[test]
fn inline_boundaries_expand_horizontal_line_metrics_without_advance() {
    let metrics = InlineMetrics::from_line_height_and_baseline(30.0, 20.0).unwrap();
    let report = layout_inline_run(InlineRunInput {
        available_width: Available::MAX_CONTENT,
        writing_mode: WritingMode::HorizontalTb,
        direction: Direction::Ltr,
        items: vec![
            inline_boundary_for(
                0,
                InlineBoundaryKind::Start,
                WritingMode::HorizontalTb,
                Direction::Ltr,
                metrics,
            ),
            InlineParticipant::new(1, Size::new(20.0, 10.0), Edges::ZERO, Some(8.0)),
            inline_boundary_for(
                2,
                InlineBoundaryKind::End,
                WritingMode::HorizontalTb,
                Direction::Ltr,
                metrics,
            ),
        ],
    });

    assert_eq!(report.size, Size::new(20.0, 30.0));
    assert_eq!(report.first_baseline, Some(20.0));
    assert_eq!(report.last_baseline, Some(20.0));
    assert_eq!(report.items[0].kind, InlineParticipantLayoutKind::InlineBoundaryStart);
    assert_eq!(report.items[0].location, Point::new(0.0, 20.0));
    assert_eq!(report.items[0].size, Size::ZERO);
    assert_eq!(report.items[1].kind, InlineParticipantLayoutKind::Box);
    assert_eq!(report.items[1].location, Point::new(0.0, 12.0));
    assert_eq!(report.items[2].kind, InlineParticipantLayoutKind::InlineBoundaryEnd);
    assert_eq!(report.items[2].location, Point::new(20.0, 20.0));
    assert_eq!(report.items[2].size, Size::ZERO);
}

#[test]
fn inline_boundaries_do_not_affect_intrinsic_widths_or_wrapping() {
    let metrics = InlineMetrics::from_line_height_and_baseline(80.0, 60.0).unwrap();
    let items = vec![
        InlineParticipant::new(0, Size::new(30.0, 10.0), Edges::ZERO, Some(8.0)),
        inline_boundary_for(
            1,
            InlineBoundaryKind::Start,
            WritingMode::HorizontalTb,
            Direction::Ltr,
            metrics,
        ),
        InlineParticipant::new(2, Size::new(25.0, 10.0), Edges::ZERO, Some(8.0)),
        inline_boundary_for(
            3,
            InlineBoundaryKind::End,
            WritingMode::HorizontalTb,
            Direction::Ltr,
            metrics,
        ),
    ];

    assert_eq!(inline_run_min_content_width(&items), 30.0);
    assert_eq!(inline_run_max_content_width(&items), 55.0);

    let report = layout_inline_run(InlineRunInput {
        available_width: Available::Definite(40.0),
        writing_mode: WritingMode::HorizontalTb,
        direction: Direction::Ltr,
        items,
    });

    assert_eq!(report.size, Size::new(30.0, 90.0));
    assert_eq!(report.items[1].location, Point::new(30.0, 60.0));
    assert_eq!(report.items[2].location, Point::new(0.0, 80.0));
}
```

- [ ] **Step 2: Run the tests and verify boundary layout is missing**

Run:

```sh
cargo test -p surgeist-layout inline_boundaries_ -- --nocapture
```

Expected: compile or assertion failure because boundary participants are not yet handled in line construction/reporting.

- [ ] **Step 3: Add horizontal pending boundary items**

In `src/inline.rs`, extend pending/report kinds:

```rust
pub(super) enum InlineParticipantLayoutKind {
    Box,
    ForcedLineBreak,
    InlineBoundaryStart,
    InlineBoundaryEnd,
}

enum PendingInlineItem<S: LayoutScalar = DefaultScalar> {
    Box { item: AtomicInlineBoxParticipant<S>, x: S },
    ForcedLineBreak { order: u32, x: S },
    Boundary { control: InlineBoundaryControlOf<S>, x: S },
}
```

Add this helper:

```rust
fn inline_boundary_layout_kind(kind: InlineBoundaryKind) -> InlineParticipantLayoutKind {
    match kind {
        InlineBoundaryKind::Start => InlineParticipantLayoutKind::InlineBoundaryStart,
        InlineBoundaryKind::End => InlineParticipantLayoutKind::InlineBoundaryEnd,
    }
}
```

Add `InlineLine::push_boundary`:

```rust
fn push_boundary(&mut self, control: InlineBoundaryControlOf<S>) {
    let metrics = control.metrics();
    self.baseline = self.baseline.max(metrics.baseline());
    self.descent = self.descent.max(metrics.after_baseline());
    self.items.push(PendingInlineItem::Boundary {
        control,
        x: self.width,
    });
}
```

- [ ] **Step 4: Handle boundary participants in horizontal layout**

In the `for item in input.items` loop in `layout_inline_run`, add:

```rust
InlineParticipant::Boundary(control) => {
    line.push_boundary(control);
}
```

In the report emission match, add:

```rust
PendingInlineItem::Boundary { control, x } => {
    items.push(InlineParticipantLayoutItem {
        kind: inline_boundary_layout_kind(control.kind()),
        order: control.order(),
        location: axis_mapping.physical_item_origin(
            LogicalInlinePointOf::new(x, line_baseline),
            LogicalInlineSizeOf::new(S::ZERO, S::ZERO),
            LogicalInlineSizeOf::new(report_inline_extent, line_height),
            line_height,
        ),
        size: Size::ZERO,
        content_size: Size::ZERO,
        margin: Edges::ZERO,
        padding: Edges::ZERO,
        border: Edges::ZERO,
        scrollbar_size: Size::ZERO,
    });
}
```

- [ ] **Step 5: Exclude boundaries from intrinsic sizes**

In `inline_run_min_content_width`, use:

```rust
filter_map(|item| match item {
    InlineParticipant::AtomicBox(item) => Some(item.advance()),
    InlineParticipant::ForcedLineBreak(_) | InlineParticipant::Boundary(_) => None,
})
```

In `inline_run_max_content_width`, use:

```rust
match item {
    InlineParticipant::AtomicBox(item) => {
        segment_width = segment_width + item.advance();
    }
    InlineParticipant::ForcedLineBreak(_) => {
        max_width = max_width.max(segment_width);
        segment_width = S::ZERO;
    }
    InlineParticipant::Boundary(_) => {}
}
```

- [ ] **Step 6: Add failing vertical test**

In `src/inline_tests.rs`, add:

```rust
#[test]
fn inline_boundaries_expand_vertical_line_metrics_without_inline_advance() {
    let metrics = InlineMetrics::from_line_height_and_baseline(26.0, 18.0).unwrap();
    let report = layout_inline_run(InlineRunInput {
        available_width: Available::MAX_CONTENT,
        writing_mode: WritingMode::VerticalRl,
        direction: Direction::Ltr,
        items: vec![
            inline_boundary_for(
                0,
                InlineBoundaryKind::Start,
                WritingMode::VerticalRl,
                Direction::Ltr,
                metrics,
            ),
            InlineParticipant::new(1, Size::new(10.0, 30.0), Edges::ZERO, Some(24.0)),
            inline_boundary_for(
                2,
                InlineBoundaryKind::End,
                WritingMode::VerticalRl,
                Direction::Ltr,
                metrics,
            ),
        ],
    });

    assert_eq!(report.size, Size::new(26.0, 30.0));
    assert_eq!(report.items[0].kind, InlineParticipantLayoutKind::InlineBoundaryStart);
    assert_eq!(report.items[0].size, Size::ZERO);
    assert_eq!(report.items[1].kind, InlineParticipantLayoutKind::Box);
    assert_eq!(report.items[2].kind, InlineParticipantLayoutKind::InlineBoundaryEnd);
    assert_eq!(report.items[2].size, Size::ZERO);
}
```

- [ ] **Step 7: Add vertical pending boundary items**

In `src/inline.rs`, extend `PendingVerticalInlineItem`:

```rust
Boundary {
    control: InlineBoundaryControlOf<S>,
    logical_inline_start: S,
    baseline: S,
},
```

Add `VerticalInlineLine::push_boundary`:

```rust
fn push_boundary(&mut self, control: InlineBoundaryControlOf<S>) {
    let metrics = control.metrics();
    self.first_report_baseline
        .get_or_insert(self.inline_extent + metrics.baseline());
    self.last_report_baseline = Some(self.inline_extent + metrics.baseline());
    self.block_extent = self.block_extent.max(metrics.line_extent());
    self.items.push(PendingVerticalInlineItem::Boundary {
        control,
        logical_inline_start: self.inline_extent,
        baseline: metrics.baseline(),
    });
}
```

- [ ] **Step 8: Handle vertical boundary participants**

In `layout_vertical_inline_run`, add:

```rust
InlineParticipant::Boundary(control) => {
    line.push_boundary(control);
}
```

In `layout_vertical_inline_lines`, add:

```rust
PendingVerticalInlineItem::Boundary {
    control,
    logical_inline_start,
    baseline,
} => {
    items.push(InlineParticipantLayoutItem {
        kind: inline_boundary_layout_kind(control.kind()),
        order: control.order(),
        location: axis_mapping.physical_item_origin(
            LogicalInlinePointOf::new(
                logical_inline_start,
                logical_block_start + baseline,
            ),
            LogicalInlineSizeOf::new(S::ZERO, S::ZERO),
            LogicalInlineSizeOf::new(line_inline_extent, line_block_extent),
            container_block_extent,
        ),
        size: Size::ZERO,
        content_size: Size::ZERO,
        margin: Edges::ZERO,
        padding: Edges::ZERO,
        border: Edges::ZERO,
        scrollbar_size: Size::ZERO,
    });
}
```

- [ ] **Step 9: Run inline tests**

Run:

```sh
cargo test -p surgeist-layout inline_boundaries_ -- --nocapture
cargo test -p surgeist-layout inline_run -- --nocapture
```

Expected: all selected tests pass. If the second command selects no tests after the rename, run:

```sh
cargo test -p surgeist-layout inline -- --nocapture
```

and confirm the inline module tests pass.

- [ ] **Step 10: Assign scoped review for Task 3**

Assign a separate reviewer after the worker reports tests and git status. The reviewer must inspect only Task 3 changes in `src/inline.rs` and `src/inline_tests.rs`; confirm boundaries are zero-advance, non-breaking, excluded from intrinsic widths, metric-bearing in horizontal and vertical paths, and preserve start/end output kinds.

- [ ] **Step 11: Commit Task 3 after review is clean**

```sh
git add src/inline.rs src/inline_tests.rs
git commit -m "Lay out inline boundary participants"
```

---

## Task 4: Integrate Boundary Inputs Into Block Inline Runs

**Files:**
- Modify: `src/block.rs`
- Modify: `src/test_support/layout_tree.rs`
- Test: `src/block_tests.rs`

- [ ] **Step 1: Add block integration tests**

In `src/block_tests.rs`, add tests near the existing line-break inline-run tests:

```rust
#[test]
fn block_inline_boundaries_wrap_atomic_child_and_expand_line_metrics() {
    let metrics = InlineMetrics::from_line_height_and_baseline(32.0, 24.0).unwrap();
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(100.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .inline_boundary(1, InlineBoundaryInput::new(InlineBoundaryKind::Start, metrics))
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .inline_boundary(3, InlineBoundaryInput::new(InlineBoundaryKind::End, metrics));

    compute_root(&mut tree, 0, Size::new(Available::definite(100.0), Available::MAX_CONTENT));
    round_layout(&mut tree, 0);

    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(100.0, 32.0));
    assert_eq!(tree.final_layout(1).unwrap().size, Size::ZERO);
    assert_eq!(tree.final_layout(1).unwrap().location, Point::new(0.0, 24.0));
    assert_eq!(tree.final_layout(2).unwrap().location, Point::new(0.0, 14.0));
    assert_eq!(tree.final_layout(3).unwrap().size, Size::ZERO);
    assert_eq!(tree.final_layout(3).unwrap().location, Point::new(20.0, 24.0));
}

#[test]
fn vertical_block_inline_boundaries_use_parent_flow() {
    let metrics = InlineMetrics::from_line_height_and_baseline(26.0, 18.0).unwrap();
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(Dimension::px(100.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .inline_boundary(
            1,
            InlineBoundaryInput::new(InlineBoundaryKind::Start, metrics)
                .with_writing_mode(WritingMode::VerticalRl),
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(10.0), Dimension::px(30.0)),
                ..NodeInput::DEFAULT
            },
        )
        .inline_boundary(
            3,
            InlineBoundaryInput::new(InlineBoundaryKind::End, metrics)
                .with_writing_mode(WritingMode::VerticalRl),
        );

    compute_root(&mut tree, 0, Size::new(Available::definite(100.0), Available::MAX_CONTENT));
    round_layout(&mut tree, 0);

    assert_eq!(tree.final_layout(1).unwrap().size, Size::ZERO);
    assert_eq!(tree.final_layout(2).unwrap().size, Size::new(10.0, 30.0));
    assert_eq!(tree.final_layout(3).unwrap().size, Size::ZERO);
}

#[test]
fn hidden_compute_sets_inline_boundary_children_to_hidden_output() {
    let metrics = InlineMetrics::from_line_height_and_baseline(20.0, 14.0).unwrap();
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::None,
                ..NodeInput::DEFAULT
            },
        )
        .inline_boundary(1, InlineBoundaryInput::new(InlineBoundaryKind::Start, metrics));

    compute_root(&mut tree, 0, Size::new(Available::definite(100.0), Available::MAX_CONTENT));
    round_layout(&mut tree, 0);

    assert_eq!(tree.final_layout(1).unwrap(), NodeOutput::with_order(0));
}
```

- [ ] **Step 2: Add an `OracleTree` boundary helper**

In `src/test_support/layout_tree.rs`, import `InlineBoundaryInputOf` and add this method to the existing `impl<S: LayoutScalar> OracleTreeOf<S>` block near `.line_break(...)`:

```rust
pub fn inline_boundary(mut self, node: u32, input: InlineBoundaryInputOf<S>) -> Self {
    self.layout_inputs
        .insert(node, LayoutInputOf::InlineBoundary(input));
    self
}
```

- [ ] **Step 3: Run block tests and verify failure**

Run:

```sh
cargo test -p surgeist-layout block_inline_boundaries -- --nocapture
```

Expected: compile failure or test failure because block does not yet collect and report `LayoutInputOf::InlineBoundary`.

- [ ] **Step 4: Add boundary validation and conversion helpers**

In `src/block.rs`, add `InlineBoundaryInputOf` and `InlineBoundaryControlOf` to imports.

Add:

```rust
fn visible_inline_boundary_in_flow<Tree>(
    tree: &Tree,
    child: <Tree as Traverse>::Node,
    flow_writing_mode: WritingMode,
    flow_direction: Direction,
) -> Option<InlineBoundaryInputOf<<Tree as Traverse>::Scalar>>
where
    Tree: Compute,
{
    let LayoutInputOf::InlineBoundary(boundary) = tree.layout_input(child) else {
        return None;
    };
    if boundary.writing_mode() != flow_writing_mode || boundary.direction() != flow_direction {
        panic!("inline-boundary flow must match containing inline flow");
    }
    Some(boundary)
}

fn inline_boundary_control<S: LayoutScalar>(
    order: u32,
    input: InlineBoundaryInputOf<S>,
    available_inline_extent: AvailableOf<S>,
) -> InlineBoundaryControlOf<S> {
    InlineBoundaryControlOf::new(
        order,
        input.kind(),
        InlineFlowOf::new(input.writing_mode(), input.direction(), available_inline_extent),
        input.metrics(),
        InlineControlAlignment::from(input.vertical_align()),
    )
}
```

- [ ] **Step 5: Include boundaries in inline-run discovery**

In `atomic_inline_run_end`, renamed to `inline_run_end`, add:

```rust
LayoutInputOf::InlineBoundary(_) => {
    visible_inline_boundary_in_flow(
        tree,
        children[index],
        constants.writing_mode,
        constants.direction,
    );
}
```

In the main block child dispatch, add `LayoutInputOf::InlineBoundary(_)` to the same path as inline-level boxes and line breaks. Boundary children should start or continue an inline run; they must not be computed as boxes.

- [ ] **Step 6: Ignore boundaries for clear scans**

In `next_atomic_inline_clear_candidate`, renamed to `next_inline_clear_candidate`, keep existing line-break clear behavior and add:

```rust
if matches!(tree.layout_input(child), LayoutInputOf::InlineBoundary(_)) {
    continue;
}
```

- [ ] **Step 7: Collect boundary run children and participants**

Extend the run child enum:

```rust
Boundary {
    child: <Tree as Traverse>::Node,
    order: u32,
},
```

In `layout_inline_run_children`, add this match arm:

```rust
LayoutInputOf::InlineBoundary(boundary) => {
    let boundary = visible_inline_boundary_in_flow(
        tree,
        child,
        constants.writing_mode,
        constants.direction,
    )
    .unwrap();
    let available_inline_extent = node_inner_size
        .width
        .map(AvailableOf::<S>::definite)
        .unwrap_or(input.available.width);
    run_children.push(InlineRunChild::Boundary { child, order });
    items.push(InlineParticipant::inline_boundary(inline_boundary_control(
        order,
        boundary,
        available_inline_extent,
    )));
    continue;
}
```

- [ ] **Step 8: Report boundary outputs**

In the report application loop, add this arm next to `InlineRunChild::LineBreak`:

```rust
InlineRunChild::Boundary { child, order } => {
    if set_layout {
        let item = report_items_by_order[order];
        tree.set_unrounded(
            *child,
            NodeOutputOf::<S> {
                order: item.order,
                location: Point::new(
                    constants.content_box_inset.left + run_offset + item.location.x,
                    cursor_y + item.location.y,
                ),
                size: Size::ZERO,
                content_size: Size::ZERO,
                scrollbar_size: Size::ZERO,
                border: Edges::ZERO,
                padding: Edges::ZERO,
                margin: Edges::ZERO,
            },
        );
    }
}
```

- [ ] **Step 9: Run block tests**

Run:

```sh
cargo test -p surgeist-layout block_inline_boundaries -- --nocapture
cargo test -p surgeist-layout hidden_compute_sets_inline_boundary_children_to_hidden_output -- --nocapture
cargo test -p surgeist-layout vertical_block_inline_boundaries_use_parent_flow -- --nocapture
```

Expected: pass.

- [ ] **Step 10: Assign scoped review for Task 4**

Assign a separate reviewer after the worker reports tests and git status. The reviewer must inspect only Task 4 changes in `src/block.rs`, `src/block_tests.rs`, and `src/test_support/layout_tree.rs`; confirm boundary inputs join inline runs, mismatched flow is rejected, outputs use the same coordinate convention as line breaks, and no box computation path is used for boundary nodes.

- [ ] **Step 11: Commit Task 4 after review is clean**

```sh
git add src/block.rs src/block_tests.rs src/test_support/layout_tree.rs
git commit -m "Integrate inline boundaries into block runs"
```

## Task 5: Update Contract Documentation And Cross-Crate Notes

**Files:**
- Modify: `plans/specs/2026-07-09-surgeist-layout-mixed-inline-participant-contract-spec.md`
- Modify: `plans/2026-06-29-surgeist-layout-br-line-break-cross-crate-ledger.md`

- [ ] **Step 1: Record that fixture metadata remains root-owned**

Append this entry to `plans/2026-06-29-surgeist-layout-br-line-break-cross-crate-ledger.md`:

```markdown
### Root fixture metadata must expose layout-ready inline boundaries

- Affected API: `surgeist_layout::InlineBoundaryInputOf<S>` and
  `surgeist_layout::LayoutInputOf::InlineBoundary`.
- Layout status: layout accepts and computes typed inline boundary participants.
- Required upstream behavior: retained/style/root must decide when anonymous or
  explicit inline wrapper boundaries have layout-relevant effects and must pass
  layout-ready start/end kind, writing mode, direction, vertical alignment, and
  `InlineMetricsOf<S>` through root fixture metadata.
- Layout must not invent this metadata from fixture HTML, CSS strings, or tag
  names because anonymous wrapper normalization and computed style effects are
  owned outside this crate.
- Browser parity implication: boundary-backed HTML fixture checks remain
  unsupported until root/surgeist-test provide layout-ready boundary metadata.
```

- [ ] **Step 2: Revise the mixed inline participant spec**

In `plans/specs/2026-07-09-surgeist-layout-mixed-inline-participant-contract-spec.md`, replace the participant category enum with:

```rust
pub(crate) enum InlineParticipantOf<S: LayoutScalar = DefaultScalar> {
    AtomicBox(AtomicInlineBoxParticipantOf<S>),
    ForcedLineBreak(ForcedLineBreakControlOf<S>),
    Boundary(InlineBoundaryControlOf<S>),
    MeasuredText(MeasuredTextParticipantOf<S>),
}
```

Replace the old boundary decision text with:

```markdown
## Inline Boundary Participant Contract

Inline boundaries model layout-relevant inline wrapper start/end items. They are
not DOM nodes and they are not CSS style objects. Retained/style/root decide
which wrapper boundaries exist and provide layout-ready metrics; layout consumes
the resulting typed participants.

Required layout-ready data:

- stable order;
- start or end kind;
- `InlineFlowOf<S>` containing writing mode, direction, and available inline
  extent;
- validated `InlineMetricsOf<S>`;
- layout-ready alignment.

Layout-owned behavior:

- contributes line metrics and baseline data;
- preserves start/end ordering in the output stream;
- has zero inline advance, zero size, and no decorations;
- does not force a line break;
- does not affect intrinsic inline-size calculations;
- participates in horizontal and vertical logical-to-physical placement.

Non-goals:

- no anonymous wrapper synthesis in layout;
- no CSS inheritance or style propagation in layout;
- no raw text, DOM tag, selector, or font data in boundary inputs.
```

Remove the previous "Decisions Required Before Runtime Implementation" item asking whether boundaries are needed. Keep measured text decisions only if they are still unresolved.

- [ ] **Step 3: Run documentation and fixture checks**

Run:

```sh
if rg -n "future inline fragment boundaries|whether inline fragment boundaries are needed|eventually|TBD|TODO|implement later" plans/specs/2026-07-09-surgeist-layout-mixed-inline-participant-contract-spec.md plans/2026-07-09-surgeist-layout-typed-inline-boundary-participants-implementation.md plans/2026-06-29-surgeist-layout-br-line-break-cross-crate-ledger.md | rg -v "rg -n"; then
  exit 1
fi
cargo test -p surgeist-layout inline_boundary -- --nocapture
```

Expected:

- `rg` prints no stale "future boundary" or placeholder text from the edited plan/spec/ledger.
- The focused boundary tests pass.

- [ ] **Step 4: Assign scoped review for Task 5**

Assign a separate reviewer after the worker reports checks and git status. The reviewer must inspect only Task 5 changes in the mixed inline spec and cross-crate ledger; confirm the documentation says layout consumes typed layout-ready boundary inputs, root/style/retained/text own upstream wrapper and metric production, and browser parity metadata is not invented locally.

- [ ] **Step 5: Commit Task 5 after review is clean**

```sh
git add plans/specs/2026-07-09-surgeist-layout-mixed-inline-participant-contract-spec.md plans/2026-06-29-surgeist-layout-br-line-break-cross-crate-ledger.md
git commit -m "Document inline boundary participant contract"
```

---

## Task 6: Final Verification And Review

**Files:**
- Inspect all files changed by Tasks 1-5.

- [ ] **Step 1: Run formatting and focused tests**

Run:

```sh
cargo fmt --check
cargo test -p surgeist-layout inline_boundary -- --nocapture
cargo test -p surgeist-layout inline_boundaries -- --nocapture
cargo test -p surgeist-layout block_inline_boundaries -- --nocapture
cargo test -p surgeist-layout layout_input_distinguishes_inline_boundary -- --nocapture
```

Expected: all tests pass. If a focused test selector reports zero tests, run `cargo test -p surgeist-layout inline -- --nocapture` and record which selector was empty.

- [ ] **Step 2: Run the crate baseline**

Run:

```sh
cargo test -p surgeist-layout
cargo clippy -p surgeist-layout --all-targets -- -D warnings
cargo fmt --check
git diff --check
git status --short --branch
```

Expected: test, clippy, fmt, and diff checks pass. Git status shows only intentional committed changes or a clean tree after all task commits.

- [ ] **Step 3: Required final holistic review**

Assign a clean-context reviewer. Give the reviewer only:

- this plan path;
- `guidance/surgeist-rust-modeling-guide.md`;
- `plans/specs/2026-07-09-surgeist-layout-mixed-inline-participant-contract-spec.md`;
- changed file list from `git show --stat --oneline HEAD~5..HEAD`;
- commands and outputs from Steps 1-2.

Reviewer instructions:

```text
Review the implemented typed inline boundary participant work holistically.
Check the code itself, not only the plan. Confirm that layout owns only
layout-ready boundary inputs, does not synthesize DOM/style/text data, keeps
boundary participants typed, handles horizontal and vertical writing modes,
keeps boundaries zero-advance/non-breaking/non-intrinsic, avoids compatibility
aliases, preserves existing line-break behavior, and follows
guidance/surgeist-rust-modeling-guide.md. Report findings with file/line
references. If clean, say clean.
```

- [ ] **Step 4: Reconcile review findings**

If the final reviewer reports findings, assign a worker to fix only those findings, then assign a separate reviewer to review the fix. Repeat until the reviewer result is clean.

- [ ] **Step 5: Report completion**

Report:

- commits created;
- verification commands and results;
- reviewer result;
- that browser parity fixture support depends on root/surgeist-test layout-ready boundary metadata and no layout-local fixture lowering was added;
- remaining cross-crate integration requirements for root/style/retained/text.
