# Inline Control Contract Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Introduce an internal typed inline control contract for forced line breaks and route block line-break handling through it without changing observable layout behavior.

**Architecture:** This plan implements Phases 1 and 2 from `plans/2026-07-08-surgeist-layout-inline-control-item-sequencing.md`. `LineBreakInputOf<S>` remains the public layout-ready node input. `src/inline.rs` receives the internal control model, and `src/block.rs` converts line-break children into that model through one named path while preserving current horizontal output, hidden-break behavior, and vertical unsupported behavior.

**Tech Stack:** Rust 2024, `surgeist-layout`, existing `LayoutScalar` generic APIs, crate-local unit tests, Cargo test/clippy/fmt.

---

## Source References

- Specification: `plans/specs/2026-07-08-surgeist-layout-inline-control-item-spec.md`
- Sequencing: `plans/2026-07-08-surgeist-layout-inline-control-item-sequencing.md`
- Modeling guidance: `guidance/surgeist-rust-modeling-guide.md`
- Workflow: `AGENTS.md`

## Scope

This plan does:

- add an internal typed control model for forced line breaks;
- keep `LineBreakInputOf<S>` as the public layout input;
- preserve existing public API aliases and exports;
- route block line-break conversion through one named internal conversion;
- preserve all current layout behavior.

This plan does not:

- implement `clear` on line breaks;
- implement vertical line-break layout;
- change browser parity fixture generation;
- expose inline control items publicly;
- parse HTML/CSS or compute font/text metrics;
- add compatibility aliases or extra lowering layers.

## Files

- Modify: `src/inline.rs`
  - Add internal `InlineFlowOf<S>`, `InlineControlAlignment`, `ForcedLineBreakControlOf<S>`, and `InlineControlItemOf<S>`.
  - Convert `AtomicInlineItem::ForcedLineBreak` to carry `ForcedLineBreakControlOf<S>`.
  - Add focused internal construction tests.
- Modify: `src/block.rs`
  - Build `ForcedLineBreakControlOf<S>` from `LineBreakInputOf<S>` and the run context.
  - Keep hidden line breaks skipped before active control construction.
  - Keep the current vertical panic until the vertical plan.
- Modify: `src/inline_tests.rs`
  - Update forced-line-break test construction to use the new helper or control payload.
  - Add tests proving control payload fields are preserved.
- Modify: `src/block_tests.rs`
  - Add a behavior-preservation guard for line-break conversion with non-default layout-ready metadata.

## Required Coordinator Process

The coordinator must follow `AGENTS.md`:

- assign a worker for the scoped code changes;
- assign a separate reviewer after the worker completes;
- reconcile reviewer findings before committing;
- run focused checks before commit;
- commit only after the scoped review is clean;
- run a final holistic review if this plan is expanded beyond the tasks below.

Workers are not alone in the codebase and must not revert unrelated user or agent changes.

## Task 1: Add Internal Inline Control Types

**Files:**
- Modify: `src/inline.rs`
- Modify: `src/inline_tests.rs`

- [ ] **Step 1: Add failing tests for control construction**

Add these tests near the existing forced-line-break tests in `src/inline_tests.rs`:

```rust
#[test]
fn forced_line_break_control_preserves_layout_ready_fields() {
    let metrics = InlineMetrics::from_line_height_and_baseline(24.0, 18.0).unwrap();
    let control = crate::inline::ForcedLineBreakControlOf::new(
        7,
        crate::inline::InlineFlowOf::new(
            WritingMode::HorizontalTb,
            Direction::Rtl,
            Available::definite(320.0),
        ),
        metrics,
        crate::inline::InlineControlAlignment::Top,
        Clear::Both,
    );

    assert_eq!(control.order(), 7);
    assert_eq!(control.flow().writing_mode(), WritingMode::HorizontalTb);
    assert_eq!(control.flow().direction(), Direction::Rtl);
    assert_eq!(control.flow().available_inline_extent(), Available::definite(320.0));
    assert_eq!(control.metrics(), metrics);
    assert_eq!(control.alignment(), crate::inline::InlineControlAlignment::Top);
    assert_eq!(control.clear(), Clear::Both);
}

#[test]
fn forced_line_break_control_can_be_used_as_atomic_inline_item() {
    let metrics = InlineMetrics::from_line_height_and_baseline(10.0, 8.0).unwrap();
    let control = crate::inline::ForcedLineBreakControlOf::new(
        1,
        crate::inline::InlineFlowOf::new(
            WritingMode::HorizontalTb,
            Direction::Ltr,
            Available::MAX_CONTENT,
        ),
        metrics,
        crate::inline::InlineControlAlignment::Baseline,
        Clear::None,
    );

    let report = layout_atomic_inline_items(AtomicInlineInput {
        available_width: Available::MAX_CONTENT,
        writing_mode: WritingMode::HorizontalTb,
        items: vec![
            AtomicInlineItem::new(0, Size::new(20.0, 10.0), Edges::ZERO, Some(10.0)),
            AtomicInlineItem::forced_line_break(control),
            AtomicInlineItem::new(2, Size::new(15.0, 12.0), Edges::ZERO, Some(8.0)),
        ],
    });

    assert_eq!(report.size, Size::new(20.0, 24.0));
    assert_eq!(report.items[1].kind, AtomicInlineLayoutItemKind::ForcedLineBreak);
    assert_eq!(report.items[1].order, 1);
    assert_eq!(report.items[1].location, Point::new(20.0, 10.0));
    assert_eq!(report.items[1].size, Size::ZERO);
}
```

- [ ] **Step 2: Run the focused tests and verify they fail**

Run:

```sh
cargo test -p surgeist-layout forced_line_break_control -- --nocapture
```

Expected: fail to compile because `InlineFlowOf`, `InlineControlAlignment`, and `ForcedLineBreakControlOf` do not exist and `AtomicInlineItem::forced_line_break` still takes `(order, metrics)`.

- [ ] **Step 3: Add the internal control types**

In `src/inline.rs`, update the import list to include the data needed by the new model:

```rust
use super::{
    AvailableOf, Clear, DefaultScalar, Direction, Edges, InlineMetricsOf, LayoutScalar, Point,
    Size, VerticalAlign, WritingMode,
};
```

Add these types after `AtomicInlineBoxItem`:

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct InlineFlowOf<S: LayoutScalar = DefaultScalar> {
    writing_mode: WritingMode,
    direction: Direction,
    available_inline_extent: AvailableOf<S>,
}

impl<S: LayoutScalar> InlineFlowOf<S> {
    #[must_use]
    pub(super) const fn new(
        writing_mode: WritingMode,
        direction: Direction,
        available_inline_extent: AvailableOf<S>,
    ) -> Self {
        Self {
            writing_mode,
            direction,
            available_inline_extent,
        }
    }

    #[must_use]
    pub(super) const fn writing_mode(self) -> WritingMode {
        self.writing_mode
    }

    #[must_use]
    pub(super) const fn direction(self) -> Direction {
        self.direction
    }

    #[must_use]
    pub(super) const fn available_inline_extent(self) -> AvailableOf<S> {
        self.available_inline_extent
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum InlineControlAlignment {
    Baseline,
    Top,
}

impl From<VerticalAlign> for InlineControlAlignment {
    fn from(value: VerticalAlign) -> Self {
        match value {
            VerticalAlign::Baseline => Self::Baseline,
            VerticalAlign::Top => Self::Top,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ForcedLineBreakControlOf<S: LayoutScalar = DefaultScalar> {
    order: u32,
    flow: InlineFlowOf<S>,
    metrics: InlineMetricsOf<S>,
    alignment: InlineControlAlignment,
    clear: Clear,
}

impl<S: LayoutScalar> ForcedLineBreakControlOf<S> {
    #[must_use]
    pub(super) const fn new(
        order: u32,
        flow: InlineFlowOf<S>,
        metrics: InlineMetricsOf<S>,
        alignment: InlineControlAlignment,
        clear: Clear,
    ) -> Self {
        Self {
            order,
            flow,
            metrics,
            alignment,
            clear,
        }
    }

    #[must_use]
    pub(super) const fn order(self) -> u32 {
        self.order
    }

    #[must_use]
    pub(super) const fn flow(self) -> InlineFlowOf<S> {
        self.flow
    }

    #[must_use]
    pub(super) const fn metrics(self) -> InlineMetricsOf<S> {
        self.metrics
    }

    #[must_use]
    pub(super) const fn alignment(self) -> InlineControlAlignment {
        self.alignment
    }

    #[must_use]
    pub(super) const fn clear(self) -> Clear {
        self.clear
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum InlineControlItemOf<S: LayoutScalar = DefaultScalar> {
    ForcedLineBreak(ForcedLineBreakControlOf<S>),
}
```

- [ ] **Step 4: Convert `AtomicInlineItem::ForcedLineBreak` to carry the control**

In `src/inline.rs`, replace the existing forced-line-break constructor:

```rust
#[allow(dead_code)]
#[must_use]
pub(super) const fn forced_line_break(order: u32, metrics: InlineMetricsOf<S>) -> Self {
    Self::ForcedLineBreak { order, metrics }
}
```

with:

```rust
#[allow(dead_code)]
#[must_use]
pub(super) const fn forced_line_break(control: ForcedLineBreakControlOf<S>) -> Self {
    Self::ForcedLineBreak(control)
}
```

Replace the enum variant:

```rust
ForcedLineBreak {
    order: u32,
    metrics: InlineMetricsOf<S>,
},
```

with:

```rust
ForcedLineBreak(ForcedLineBreakControlOf<S>),
```

Update matches in `src/inline.rs`:

```rust
AtomicInlineItem::ForcedLineBreak(control) => {
    line.push_forced_line_break(control);
    lines.push(line);
    line = InlineLine::<S>::default();
}
```

Change `InlineLine::push_forced_line_break` to:

```rust
fn push_forced_line_break(&mut self, control: ForcedLineBreakControlOf<S>) {
    let metrics = control.metrics();
    self.baseline = self.baseline.max(metrics.baseline());
    self.descent = self.descent.max(metrics.after_baseline());
    self.items.push(PendingInlineItem::ForcedLineBreak {
        order: control.order(),
        x: self.width,
    });
}
```

Update vertical rejection and intrinsic-width matches:

```rust
AtomicInlineItem::ForcedLineBreak(_) => {
    unreachable!("forced atomic inline breaks are unsupported in vertical-rl layout")
}
```

```rust
AtomicInlineItem::ForcedLineBreak(_) => None,
```

```rust
AtomicInlineItem::ForcedLineBreak(_) => {
    max_width = max_width.max(segment_width);
    segment_width = S::ZERO;
}
```

- [ ] **Step 5: Update existing forced-line-break tests to use the helper construction**

In `src/inline_tests.rs`, add this helper near the top after the imports:

```rust
fn forced_line_break(order: u32, metrics: InlineMetrics) -> AtomicInlineItem {
    AtomicInlineItem::forced_line_break(crate::inline::ForcedLineBreakControlOf::new(
        order,
        crate::inline::InlineFlowOf::new(
            WritingMode::HorizontalTb,
            Direction::Ltr,
            Available::MAX_CONTENT,
        ),
        metrics,
        crate::inline::InlineControlAlignment::Baseline,
        Clear::None,
    ))
}
```

Replace test calls like:

```rust
AtomicInlineItem::forced_line_break(1, first_line_metrics)
```

with:

```rust
forced_line_break(1, first_line_metrics)
```

Apply the same replacement to each existing forced-line-break test in `src/inline_tests.rs`.

- [ ] **Step 6: Run focused inline tests**

Run:

```sh
cargo test -p surgeist-layout inline_tests:: -- --nocapture
```

Expected: all inline tests pass. The vertical forced-break path should still reject forced breaks because vertical behavior is outside this plan.

## Task 2: Route Block Line Breaks Through The Control Conversion

**Files:**
- Modify: `src/block.rs`
- Modify: `src/block_tests.rs`

- [ ] **Step 1: Add a focused block conversion preservation test**

Add this test near the existing line-break block tests in `src/block_tests.rs`:

```rust
#[test]
fn block_line_break_conversion_with_metadata_preserves_current_output() {
    let metrics = InlineMetrics::from_line_height_and_baseline(24.0, 18.0).unwrap();
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                direction: Direction::Rtl,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .line_break(
            2,
            LineBreakInput::new()
                .with_direction(Direction::Rtl)
                .with_writing_mode(WritingMode::HorizontalTb)
                .with_vertical_align(VerticalAlign::Top)
                .with_clear(Clear::Both)
                .with_metrics(metrics),
        )
        .style(
            3,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(15.0), Dimension::px(12.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(100.0), Available::MAX_CONTENT),
    );
    round_layout(&mut tree, 0);

    assert_eq!(tree.inputs(2), &[]);
    assert_eq!(tree.final_layout(2).unwrap().size, Size::ZERO);
    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(80.0, 18.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(100.0, 36.0));
}
```

This test intentionally checks current behavior. `Clear::Both` and `VerticalAlign::Top` are preserved in the control payload, but they must not change output in this no-behavior-change plan.

- [ ] **Step 2: Run the focused block test and record the current result**

Run:

```sh
cargo test -p surgeist-layout block_line_break_conversion_with_metadata_preserves_current_output -- --nocapture
```

Expected: pass before and after implementation. This test is a behavior-preservation guard for the block integration path; the field-preservation checks live in the focused `src/inline_tests.rs` control construction tests from Task 1.

- [ ] **Step 3: Add a named conversion helper in `src/block.rs`**

Update the `use crate::inline` import at the top of `src/block.rs` to include the new control types:

```rust
use crate::inline::{
    AtomicInlineBoxItem, AtomicInlineInput, AtomicInlineItem, AtomicInlineLayoutItem,
    ForcedLineBreakControlOf, InlineControlAlignment, InlineFlowOf, layout_atomic_inline_items,
};
```

Add this helper near `layout_atomic_inline_run`:

```rust
fn forced_line_break_control<S: LayoutScalar>(
    order: u32,
    input: LineBreakInputOf<S>,
    available_inline_extent: AvailableOf<S>,
) -> ForcedLineBreakControlOf<S> {
    ForcedLineBreakControlOf::new(
        order,
        InlineFlowOf::new(
            input.writing_mode(),
            input.direction(),
            available_inline_extent,
        ),
        input.metrics(),
        InlineControlAlignment::from(input.vertical_align()),
        input.clear(),
    )
}
```

If `LineBreakInputOf<S>` is not already imported in `src/block.rs`, add it to the existing crate import list.

- [ ] **Step 4: Use the helper in line-break run construction**

In `layout_atomic_inline_run`, replace:

```rust
run_children.push(AtomicInlineRunChild::LineBreak { child, order });
items.push(AtomicInlineItem::forced_line_break(order, input.metrics()));
continue;
```

with:

```rust
run_children.push(AtomicInlineRunChild::LineBreak { child, order });
items.push(AtomicInlineItem::forced_line_break(
    forced_line_break_control(
        order,
        input,
        node_inner_size
            .width
            .map(AvailableOf::<S>::definite)
            .unwrap_or(input.available.width),
    ),
));
continue;
```

If the local variable name `input` conflicts with the outer `ComputeInputOf<S>`, rename the matched line-break binding to `line_break` and pass `line_break` into the helper:

```rust
LayoutInputOf::LineBreak(line_break) => {
    if line_break.display().is_none() {
        if set_layout {
            tree.set_unrounded(child, NodeOutputOf::<S>::with_order(order));
        }
        continue;
    }
    if line_break.writing_mode() != WritingMode::HorizontalTb {
        panic!("vertical line-break layout is not implemented");
    }

    run_children.push(AtomicInlineRunChild::LineBreak { child, order });
    items.push(AtomicInlineItem::forced_line_break(
        forced_line_break_control(
            order,
            line_break,
            node_inner_size
                .width
                .map(AvailableOf::<S>::definite)
                .unwrap_or(input.available.width),
        ),
    ));
    continue;
}
```

- [ ] **Step 5: Preserve hidden and vertical behavior**

Do not change these existing behaviors in `src/block.rs`:

```rust
if line_break.display().is_none() {
    if set_layout {
        tree.set_unrounded(child, NodeOutputOf::<S>::with_order(order as u32));
    }
    index += 1;
    continue;
}
```

```rust
if line_break.writing_mode() != WritingMode::HorizontalTb {
    panic!("vertical line-break layout is not implemented");
}
```

The vertical panic is intentional until the later logical-axis and vertical forced-break plans.

- [ ] **Step 6: Run focused block line-break tests**

Run:

```sh
cargo test -p surgeist-layout line_break -- --nocapture
```

Expected: existing line-break tests pass, including `vertical_line_break_panics_until_modeled`.

## Task 3: Review Modeling And Boundary Invariants

**Files:**
- Modify: `src/inline.rs`
- Modify: `src/block.rs`

- [ ] **Step 1: Search for accidental broadening**

Run:

```sh
rg -n "ForcedLineBreak|InlineControl|InlineFlow|LineBreakInput|vertical line-break|clear\\(\\)" src/inline.rs src/block.rs src/inline_tests.rs src/block_tests.rs
```

Expected:

- `ForcedLineBreak` in `src/inline.rs` carries `ForcedLineBreakControlOf<S>`.
- `src/block.rs` has one named helper that converts `LineBreakInputOf<S>` into `ForcedLineBreakControlOf<S>`.
- `input.clear()` is captured by the control payload but not applied to layout.
- the current vertical panic remains.
- no HTML, CSS, font, retained, or fixture parsing logic appears in `src/inline.rs` or `src/block.rs`.

- [ ] **Step 2: Run focused tests**

Run:

```sh
cargo test -p surgeist-layout inline_tests:: -- --nocapture
cargo test -p surgeist-layout line_break -- --nocapture
```

Expected: both commands pass.

- [ ] **Step 3: Run required final checks**

Run:

```sh
cargo test -p surgeist-layout
cargo clippy -p surgeist-layout --all-targets -- -D warnings
cargo fmt --check
git diff --check
git status --short --branch
```

Expected:

- tests pass;
- clippy passes with `-D warnings`;
- formatting is clean;
- no whitespace errors;
- git status shows only intentional changes from this plan before commit.

## Commit Point

After the worker completes all tasks and the scoped reviewer comes back clean, the coordinator should commit:

```sh
git add src/inline.rs src/block.rs src/inline_tests.rs src/block_tests.rs
git commit -m "Model inline forced break controls"
```

Do not commit before the review cycle is clean.

## Review Checklist

The clean-context reviewer should verify:

- the implementation matches this plan and the inline control item spec;
- the change is behavior-preserving except for internal representation and tests;
- `LineBreakInputOf<S>` remains the public line-break input;
- inline control types are not a generic transport bag;
- `NodeInputOf<S>` is not widened or reused for line-break state;
- `clear` and `vertical-align` are captured but not behaviorally applied;
- vertical line-break layout remains explicitly unsupported;
- layout does not parse HTML/CSS or compute font/text metrics;
- final checks listed above were run and passed.

## Follow-Up Plans

After this plan is implemented and reviewed cleanly, the next derived plan should be Phase 3 from the sequencing document: apply resolved `Clear` to forced line breaks in horizontal block flow.
