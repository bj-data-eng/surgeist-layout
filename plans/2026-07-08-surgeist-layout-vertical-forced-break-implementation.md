# Vertical Forced Break Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Support forced line-break controls in vertical inline layout for `VerticalRl` and `VerticalLr` using the existing logical-axis model.

**Architecture:** This implements Phase 5 from `plans/2026-07-08-surgeist-layout-inline-control-item-sequencing.md`. Replace the current vertical forced-break rejection with shared logical line construction in `src/inline.rs`, then let `src/block.rs` route visible vertical `LineBreakInputOf<S>` values through the same inline control contract that horizontal breaks already use. Browser parity fixture enablement remains Phase 6.

**Tech Stack:** Rust 2024, `surgeist-layout`, internal inline layout module, existing `WritingMode`, `Direction`, `InlineMetricsOf<S>`, `LayoutScalar`, crate-local unit/block tests, Cargo test/clippy/fmt.

---

## Source References

- Specification: `plans/specs/2026-07-08-surgeist-layout-inline-control-item-spec.md`
- Sequencing: `plans/2026-07-08-surgeist-layout-inline-control-item-sequencing.md`
- Previous plan: `plans/2026-07-08-surgeist-layout-logical-inline-axis-extraction.md`
- Modeling guidance: `guidance/surgeist-rust-modeling-guide.md`
- Workflow: `AGENTS.md`

## Scope

This plan does:

- support `AtomicInlineItem::ForcedLineBreak` when `AtomicInlineInput::writing_mode` is `VerticalRl`;
- support `AtomicInlineItem::ForcedLineBreak` when `AtomicInlineInput::writing_mode` is `VerticalLr`;
- map vertical forced-break output locations with `InlineAxisMapping`;
- use `InlineMetricsOf<S>` as logical block-axis line metrics in vertical layout;
- make block integration accept visible vertical line-break inputs instead of panicking;
- preserve hidden line-break behavior and zero-size line-break output;
- preserve existing vertical box-only placement, wrapping, and baseline-report behavior;
- preserve horizontal behavior and line-break clear behavior.

This plan does not:

- add browser parity HTML/XML fixtures or generator changes;
- parse HTML, authored CSS, legacy `clear` attributes, font metrics, or text;
- implement vertical float clearance;
- expose new public inline-control APIs;
- introduce a separate vertical line-break type;
- change intrinsic sizing beyond the existing forced-break segment split behavior;
- complete a richer axis-aware public baseline reporting model for vertical
  writing modes.

## Files

- Modify: `src/inline.rs`
  - Replace the vertical-rl box-only helper with a vertical logical line builder that accepts boxes and forced breaks.
  - Remove the `VerticalLr` forced-break panic from `layout_atomic_inline_items`.
  - Keep horizontal line construction output unchanged.
- Modify: `src/inline_tests.rs`
  - Replace the vertical-lr panic test with green vertical forced-break tests.
  - Add vertical-rl and vertical-lr tests for breaks between boxes, consecutive breaks, RTL progression, and zero-size break output.
- Modify: `src/block.rs`
  - Rename the horizontal-only line-break chokepoint to a visible line-break chokepoint.
  - Validate visible line-break writing mode and direction against the containing inline flow.
  - Keep clear segmentation horizontal-only.
  - Route visible vertical line-breaks through `forced_line_break_control`.
- Modify: `src/block_tests.rs`
  - Replace the vertical line-break panic test with block integration tests for vertical-rl and vertical-lr line-break placement.

No fixture, README, API artifact, generated XML, or sibling-crate change is part of this plan.

## Execution Scope

Tasks 1 through 4 are one tightly coupled scoped worker task group for the
`AGENTS.md` workflow. The red tests, inline implementation, block integration,
and final search are not independently committable because the first test task
intentionally fails until the implementation tasks land. Use one implementation
worker for this task group, one scoped reviewer after the group is complete,
one logical commit after that scoped review is clean, then the final holistic
review gate below.

## Semantics

Vertical inline layout should use logical coordinates first:

- logical inline advance maps to physical `y`;
- logical block line stacking maps to physical `x`;
- `VerticalRl` stacks new lines right-to-left;
- `VerticalLr` stacks new lines left-to-right;
- `Direction` reverses placement along the logical inline axis only;
- forced-break controls have zero physical size and no margins, padding, border, scrollbars, or children;
- forced-break metrics contribute to the committed line's logical block extent
  and the break insertion point.
- existing vertical box-only layout does not wrap by `available_width`; preserve
  that behavior. In this phase, vertical lines split only at forced breaks.
- `AtomicInlineReport::first_baseline` and `last_baseline` remain the existing
  report fields used by block integration. Use forced-break metrics for line
  stacking and break insertion placement, but do not reinterpret those report
  fields as a complete vertical baseline model in this plan.

For a vertical line containing only a forced break with metrics `(line_extent = 20, baseline = 14)`:

- the line contributes 20 units of physical width;
- the break output size is zero;
- the break output location is the physical insertion point mapped from logical `(inline = 0, block = 14)`.

For consecutive vertical breaks:

- each break commits one metric-bearing empty vertical line;
- the report logical block extent is the sum of the two line extents;
- forced-break output locations use the break baseline in logical block
  coordinates;
- baseline report fields stay on the existing vertical report convention until
  a later vertical baseline model revisits `NodeOutputOf<S>::first_baselines`
  and `last_baselines`.

Clear behavior in this plan:

- horizontal clear support remains exactly as implemented in Phase 3;
- vertical line breaks with `Clear::None` are supported;
- vertical line breaks with `clear != Clear::None` must be rejected explicitly before layout with:

```rust
panic!("vertical line-break clear layout is not implemented");
```

This keeps the normal vertical forced-break path unblocked while avoiding an implicit, incorrect vertical float-clear model.

## Task 1: Add Vertical Forced-Break Inline Tests

**Files:**
- Modify: `src/inline_tests.rs`

- [ ] **Step 1: Add a vertical-aware forced-break helper**

Replace the existing `forced_line_break` helper with a helper that delegates to a writing-mode-specific helper:

```rust
fn forced_line_break(order: u32, metrics: InlineMetrics) -> AtomicInlineItem {
    forced_line_break_for(order, WritingMode::HorizontalTb, Direction::Ltr, metrics)
}

fn forced_line_break_for(
    order: u32,
    writing_mode: WritingMode,
    direction: Direction,
    metrics: InlineMetrics,
) -> AtomicInlineItem {
    AtomicInlineItem::forced_line_break(crate::inline::ForcedLineBreakControlOf::new(
        order,
        crate::inline::InlineFlowOf::new(writing_mode, direction, Available::MAX_CONTENT),
        metrics,
        crate::inline::InlineControlAlignment::Baseline,
        Clear::None,
    ))
}
```

- [ ] **Step 2: Replace the vertical-lr panic test with green vertical tests**

Replace `atomic_inline_vertical_lr_forced_break_panics_until_modeled` with:

```rust
#[test]
fn atomic_inline_vertical_rl_forced_break_starts_next_line() {
    let metrics = InlineMetrics::from_line_height_and_baseline(20.0, 14.0).unwrap();
    let report = layout_atomic_inline_items(AtomicInlineInput {
        available_width: Available::definite(80.0),
        writing_mode: WritingMode::VerticalRl,
        direction: Direction::Ltr,
        items: vec![
            AtomicInlineItem::new(0, Size::new(10.0, 30.0), Edges::ZERO, Some(24.0)),
            forced_line_break_for(1, WritingMode::VerticalRl, Direction::Ltr, metrics),
            AtomicInlineItem::new(2, Size::new(12.0, 16.0), Edges::ZERO, Some(12.0)),
        ],
    });

    assert_eq!(report.size, Size::new(80.0, 30.0));
    assert_eq!(report.items[0].location, Point::new(70.0, 0.0));
    assert_eq!(report.items[1].location, Point::new(66.0, 30.0));
    assert_eq!(report.items[1].kind, AtomicInlineLayoutItemKind::ForcedLineBreak);
    assert_eq!(report.items[1].size, Size::ZERO);
    assert_eq!(report.items[2].location, Point::new(48.0, 0.0));
    assert_eq!(report.first_baseline, Some(24.0));
    assert_eq!(report.last_baseline, Some(12.0));
}

#[test]
fn atomic_inline_vertical_lr_forced_break_starts_next_line() {
    let metrics = InlineMetrics::from_line_height_and_baseline(20.0, 14.0).unwrap();
    let report = layout_atomic_inline_items(AtomicInlineInput {
        available_width: Available::definite(80.0),
        writing_mode: WritingMode::VerticalLr,
        direction: Direction::Ltr,
        items: vec![
            AtomicInlineItem::new(0, Size::new(10.0, 30.0), Edges::ZERO, Some(24.0)),
            forced_line_break_for(1, WritingMode::VerticalLr, Direction::Ltr, metrics),
            AtomicInlineItem::new(2, Size::new(12.0, 16.0), Edges::ZERO, Some(12.0)),
        ],
    });

    assert_eq!(report.size, Size::new(22.0, 30.0));
    assert_eq!(report.items[0].location, Point::new(0.0, 0.0));
    assert_eq!(report.items[1].location, Point::new(14.0, 30.0));
    assert_eq!(report.items[1].kind, AtomicInlineLayoutItemKind::ForcedLineBreak);
    assert_eq!(report.items[1].size, Size::ZERO);
    assert_eq!(report.items[2].location, Point::new(10.0, 0.0));
    assert_eq!(report.first_baseline, Some(24.0));
    assert_eq!(report.last_baseline, Some(12.0));
}
```

- [ ] **Step 3: Add consecutive and RTL tests**

Add:

```rust
#[test]
fn atomic_inline_vertical_forced_breaks_create_empty_metric_bearing_lines() {
    let metrics = InlineMetrics::from_line_height_and_baseline(20.0, 14.0).unwrap();
    let report = layout_atomic_inline_items(AtomicInlineInput {
        available_width: Available::definite(80.0),
        writing_mode: WritingMode::VerticalRl,
        direction: Direction::Ltr,
        items: vec![
            forced_line_break_for(0, WritingMode::VerticalRl, Direction::Ltr, metrics),
            forced_line_break_for(1, WritingMode::VerticalRl, Direction::Ltr, metrics),
        ],
    });

    assert_eq!(report.size, Size::new(80.0, 0.0));
    assert_eq!(report.items[0].location, Point::new(66.0, 0.0));
    assert_eq!(report.items[1].location, Point::new(46.0, 0.0));
    assert_eq!(report.first_baseline, Some(0.0));
    assert_eq!(report.last_baseline, Some(0.0));
}

#[test]
fn atomic_inline_vertical_rl_rtl_forced_break_uses_bottom_to_top_inline_progression() {
    let metrics = InlineMetrics::from_line_height_and_baseline(20.0, 14.0).unwrap();
    let report = layout_atomic_inline_items(AtomicInlineInput {
        available_width: Available::definite(80.0),
        writing_mode: WritingMode::VerticalRl,
        direction: Direction::Rtl,
        items: vec![
            AtomicInlineItem::new(0, Size::new(10.0, 30.0), Edges::ZERO, Some(24.0)),
            forced_line_break_for(1, WritingMode::VerticalRl, Direction::Rtl, metrics),
            AtomicInlineItem::new(2, Size::new(12.0, 16.0), Edges::ZERO, Some(12.0)),
        ],
    });

    assert_eq!(report.size, Size::new(80.0, 30.0));
    assert_eq!(report.items[0].location, Point::new(70.0, 0.0));
    assert_eq!(report.items[1].location, Point::new(66.0, 0.0));
    assert_eq!(report.items[2].location, Point::new(48.0, 14.0));
}
```

- [ ] **Step 4: Add vertical preservation regression tests**

Add:

```rust
#[test]
fn atomic_inline_vertical_rl_tall_box_run_does_not_wrap_by_available_width() {
    let report = layout_atomic_inline_items(AtomicInlineInput {
        available_width: Available::definite(40.0),
        writing_mode: WritingMode::VerticalRl,
        direction: Direction::Ltr,
        items: vec![
            AtomicInlineItem::new(0, Size::new(10.0, 35.0), Edges::ZERO, Some(35.0)),
            AtomicInlineItem::new(1, Size::new(10.0, 35.0), Edges::ZERO, Some(35.0)),
        ],
    });

    assert_eq!(report.size, Size::new(40.0, 70.0));
    assert_eq!(report.items[0].location, Point::new(30.0, 0.0));
    assert_eq!(report.items[1].location, Point::new(30.0, 35.0));
    assert_eq!(report.first_baseline, Some(35.0));
    assert_eq!(report.last_baseline, Some(70.0));
}

#[test]
fn atomic_inline_vertical_rl_preserves_zero_height_box_centering() {
    let report = layout_atomic_inline_items(AtomicInlineInput {
        available_width: Available::definite(70.0),
        writing_mode: WritingMode::VerticalRl,
        direction: Direction::Ltr,
        items: vec![AtomicInlineItem::new(
            0,
            Size::new(10.0, 0.0),
            Edges::ZERO,
            Some(0.0),
        )],
    });

    assert_eq!(report.size, Size::new(70.0, 0.0));
    assert_eq!(report.items[0].location, Point::new(65.0, 0.0));
}
```

- [ ] **Step 5: Run the new inline tests and verify they fail**

Run:

```sh
cargo test -p surgeist-layout atomic_inline_vertical -- --nocapture
```

Expected: the new vertical forced-break tests fail or panic on the current vertical rejection paths. Existing vertical box-only tests should still pass.

## Task 2: Implement Vertical Logical Line Construction

**Files:**
- Modify: `src/inline.rs`

- [ ] **Step 1: Replace the vertical dispatch and remove the vertical-lr panic**

Change the start of `layout_atomic_inline_items` to:

```rust
if matches!(input.writing_mode, WritingMode::VerticalRl | WritingMode::VerticalLr) {
    return layout_vertical_atomic_inline_items(input);
}
```

Remove the special `VerticalLr` forced-break panic block.

- [ ] **Step 2: Add vertical-specific pending line state**

Do not reuse `InlineLine<S>` for vertical layout. Its `baseline` and `descent`
fields currently mean horizontal block-axis line metrics. Vertical layout needs
to preserve the existing vertical baseline-report convention while also tracking
logical block-axis line extent for line stacking.

Add these private types near `InlineLine<S>`:

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
enum PendingVerticalInlineItem<S: LayoutScalar = DefaultScalar> {
    Box {
        item: AtomicInlineBoxItem<S>,
        logical_inline_start: S,
    },
    ForcedLineBreak {
        order: u32,
        logical_inline_start: S,
        baseline: S,
    },
}

#[derive(Clone, Debug, Default, PartialEq)]
struct VerticalInlineLine<S: LayoutScalar = DefaultScalar> {
    items: Vec<PendingVerticalInlineItem<S>>,
    inline_extent: S,
    block_extent: S,
    first_report_baseline: Option<S>,
    last_report_baseline: Option<S>,
}

impl<S: LayoutScalar> VerticalInlineLine<S> {
    #[must_use]
    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    fn push_box(&mut self, item: AtomicInlineBoxItem<S>) {
        self.inline_extent = self.inline_extent + item.margin.top;
        let logical_inline_start = self.inline_extent;
        self.inline_extent = self.inline_extent + item.size.height + item.margin.bottom;
        let baseline = logical_inline_start + item.baseline();
        self.first_report_baseline.get_or_insert(baseline);
        self.last_report_baseline = Some(baseline);
        self.block_extent = self
            .block_extent
            .max(item.margin.left + item.size.width + item.margin.right);
        self.items.push(PendingVerticalInlineItem::Box {
            item,
            logical_inline_start,
        });
    }

    fn push_forced_line_break(&mut self, control: ForcedLineBreakControlOf<S>) {
        let metrics = control.metrics();
        self.first_report_baseline.get_or_insert(self.inline_extent);
        self.last_report_baseline = Some(self.inline_extent);
        self.block_extent = self.block_extent.max(metrics.line_extent());
        self.items.push(PendingVerticalInlineItem::ForcedLineBreak {
            order: control.order(),
            logical_inline_start: self.inline_extent,
            baseline: metrics.baseline(),
        });
    }
}
```

This keeps vertical line stacking (`block_extent`) separate from vertical
baseline reporting (`first_report_baseline`/`last_report_baseline`), matching
the existing box-only vertical path. A forced break's `metrics.baseline()` is
used for its logical block-axis insertion point, not for the current report
field.

- [ ] **Step 3: Replace `layout_vertical_rl_atomic_inline_items` with `layout_vertical_atomic_inline_items`**

Rename the helper and implement line construction with `VerticalInlineLine<S>`:

```rust
fn layout_vertical_atomic_inline_items<S: LayoutScalar>(
    input: AtomicInlineInput<S>,
) -> AtomicInlineReport<S> {
    debug_assert!(matches!(
        input.writing_mode,
        WritingMode::VerticalRl | WritingMode::VerticalLr
    ));

    let mut lines = Vec::new();
    let mut line = VerticalInlineLine::<S>::default();

    for item in input.items {
        match item {
            AtomicInlineItem::Box(item) => {
                line.push_box(item);
            }
            AtomicInlineItem::ForcedLineBreak(control) => {
                line.push_forced_line_break(control);
                lines.push(line);
                line = VerticalInlineLine::<S>::default();
            }
        }
    }

    if !line.is_empty() {
        lines.push(line);
    }

    layout_vertical_inline_lines(input.writing_mode, input.direction, input.available_width, lines)
}
```

- [ ] **Step 4: Add a vertical line placement helper**

Add:

```rust
fn layout_vertical_inline_lines<S: LayoutScalar>(
    writing_mode: WritingMode,
    direction: Direction,
    available_width: AvailableOf<S>,
    lines: Vec<VerticalInlineLine<S>>,
) -> AtomicInlineReport<S> {
    let line_inline_extent = lines
        .iter()
        .map(|line| line.inline_extent)
        .fold(S::ZERO, S::max);
    let logical_block_extent = lines
        .iter()
        .map(|line| line.block_extent)
        .fold(S::ZERO, |sum, extent| sum + extent);
    let container_block_extent = match writing_mode {
        WritingMode::VerticalRl => match available_width {
            AvailableOf::Definite(width) => width.max(logical_block_extent),
            AvailableOf::MinContent | AvailableOf::MaxContent => logical_block_extent,
        },
        WritingMode::VerticalLr => logical_block_extent,
        WritingMode::HorizontalTb => unreachable!("horizontal inline layout uses the horizontal path"),
    };
    let axis_mapping = InlineAxisMapping::new(writing_mode, direction);
    let mut logical_block_start = S::ZERO;
    let mut items = Vec::new();
    let mut first_baseline = None;
    let mut last_baseline = None;

    for line in lines {
        let line_block_extent = line.block_extent;
        if let Some(baseline) = line.first_report_baseline {
            first_baseline.get_or_insert(baseline);
        }
        if let Some(baseline) = line.last_report_baseline {
            last_baseline = Some(baseline);
        }

        for pending in line.items {
            match pending {
                PendingVerticalInlineItem::Box {
                    item,
                    logical_inline_start,
                } => {
                    let logical_block_start_for_item = if item.size.height == S::ZERO {
                        logical_block_start + item.margin.right - item.size.width / S::from_f64(2.0)
                    } else {
                        logical_block_start + item.margin.right
                    };
                    items.push(AtomicInlineLayoutItem {
                        kind: AtomicInlineLayoutItemKind::Box,
                        order: item.order,
                        location: axis_mapping.physical_item_origin(
                            LogicalInlinePointOf::new(
                                logical_inline_start,
                                logical_block_start_for_item,
                            ),
                            LogicalInlineSizeOf::new(item.size.height, item.size.width),
                            LogicalInlineSizeOf::new(line_inline_extent, line_block_extent),
                            container_block_extent,
                        ),
                        size: item.size,
                        content_size: item.content_size,
                        margin: item.margin,
                        padding: item.padding,
                        border: item.border,
                        scrollbar_size: item.scrollbar_size,
                    });
                }
                PendingVerticalInlineItem::ForcedLineBreak {
                    order,
                    logical_inline_start,
                    baseline,
                } => {
                    items.push(AtomicInlineLayoutItem {
                        kind: AtomicInlineLayoutItemKind::ForcedLineBreak,
                        order,
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
            }
        }

        logical_block_start = logical_block_start + line_block_extent;
    }

    let content_size = Size::new(container_block_extent, line_inline_extent);
    AtomicInlineReport {
        size: content_size,
        content_size,
        first_baseline,
        last_baseline,
        items,
    }
}
```

The worker may factor the helper differently, but the implementation must keep
these concepts explicit: logical inline extent, logical block extent, vertical
baseline report values, and physical placement through `InlineAxisMapping`.

- [ ] **Step 5: Run focused inline checks**

Run:

```sh
cargo test -p surgeist-layout atomic_inline_vertical -- --nocapture
cargo test -p surgeist-layout line_break -- --nocapture
cargo test -p surgeist-layout inline_axis_mapping_ -- --nocapture
```

Expected: all pass.

## Task 3: Route Vertical Line Breaks Through Block Integration

**Files:**
- Modify: `src/block.rs`
- Modify: `src/block_tests.rs`

- [ ] **Step 1: Add block-level vertical tests**

Replace `vertical_line_break_panics_until_modeled` with:

```rust
#[test]
fn vertical_rl_line_break_is_laid_out_as_zero_size_inline_control() {
    let metrics = InlineMetrics::from_line_height_and_baseline(20.0, 14.0).unwrap();
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(0, NodeInput {
            display: Display::Block,
            writing_mode: WritingMode::VerticalRl,
            size: Size::new(Dimension::px(80.0), Dimension::AUTO),
            ..NodeInput::DEFAULT
        })
        .style(1, NodeInput {
            display: Display::InlineBlock,
            writing_mode: WritingMode::VerticalRl,
            size: Size::new(Dimension::px(10.0), Dimension::px(30.0)),
            ..NodeInput::DEFAULT
        })
        .line_break(
            2,
            LineBreakInput::new()
                .with_writing_mode(WritingMode::VerticalRl)
                .with_metrics(metrics),
        )
        .style(3, NodeInput {
            display: Display::InlineBlock,
            writing_mode: WritingMode::VerticalRl,
            size: Size::new(Dimension::px(12.0), Dimension::px(16.0)),
            ..NodeInput::DEFAULT
        });

    compute_root(&mut tree, 0, Size::new(Available::definite(80.0), Available::MAX_CONTENT));
    round_layout(&mut tree, 0);

    assert_eq!(tree.final_layout(2).unwrap().size, Size::ZERO);
    assert_eq!(tree.final_layout(2).unwrap().location, Point::new(66.0, 30.0));
    assert_eq!(tree.final_layout(3).unwrap().location.x, 48.0);
}

#[test]
fn vertical_lr_line_break_is_laid_out_as_zero_size_inline_control() {
    let metrics = InlineMetrics::from_line_height_and_baseline(20.0, 14.0).unwrap();
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(0, NodeInput {
            display: Display::Block,
            writing_mode: WritingMode::VerticalLr,
            size: Size::new(Dimension::px(80.0), Dimension::AUTO),
            ..NodeInput::DEFAULT
        })
        .style(1, NodeInput {
            display: Display::InlineBlock,
            writing_mode: WritingMode::VerticalLr,
            size: Size::new(Dimension::px(10.0), Dimension::px(30.0)),
            ..NodeInput::DEFAULT
        })
        .line_break(
            2,
            LineBreakInput::new()
                .with_writing_mode(WritingMode::VerticalLr)
                .with_metrics(metrics),
        )
        .style(3, NodeInput {
            display: Display::InlineBlock,
            writing_mode: WritingMode::VerticalLr,
            size: Size::new(Dimension::px(12.0), Dimension::px(16.0)),
            ..NodeInput::DEFAULT
        });

    compute_root(&mut tree, 0, Size::new(Available::definite(80.0), Available::MAX_CONTENT));
    round_layout(&mut tree, 0);

    assert_eq!(tree.final_layout(2).unwrap().size, Size::ZERO);
    assert_eq!(tree.final_layout(2).unwrap().location, Point::new(14.0, 30.0));
    assert_eq!(tree.final_layout(3).unwrap().location.x, 10.0);
}
```

- [ ] **Step 2: Add an explicit vertical clear rejection test**

Add:

```rust
#[test]
#[should_panic(expected = "vertical line-break clear layout is not implemented")]
fn vertical_line_break_clear_panics_until_vertical_clear_is_modeled() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .style(0, NodeInput {
            display: Display::Block,
            writing_mode: WritingMode::VerticalRl,
            ..NodeInput::DEFAULT
        })
        .line_break(
            1,
            LineBreakInput::new()
                .with_writing_mode(WritingMode::VerticalRl)
                .with_clear(Clear::Both),
        );

    compute_root(&mut tree, 0, Size::new(Available::definite(80.0), Available::MAX_CONTENT));
}

#[test]
#[should_panic(expected = "vertical line-break clear layout is not implemented")]
fn vertical_parent_rejects_clear_even_when_line_break_input_defaults_horizontal() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .style(0, NodeInput {
            display: Display::Block,
            writing_mode: WritingMode::VerticalRl,
            ..NodeInput::DEFAULT
        })
        .line_break(1, LineBreakInput::new().with_clear(Clear::Both));

    compute_root(&mut tree, 0, Size::new(Available::definite(80.0), Available::MAX_CONTENT));
}

#[test]
#[should_panic(expected = "line-break flow must match containing inline flow")]
fn vertical_parent_rejects_default_line_break_flow_until_input_is_layout_ready() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .style(0, NodeInput {
            display: Display::Block,
            writing_mode: WritingMode::VerticalRl,
            direction: Direction::Ltr,
            ..NodeInput::DEFAULT
        })
        .line_break(1, LineBreakInput::new());

    compute_root(&mut tree, 0, Size::new(Available::definite(80.0), Available::MAX_CONTENT));
}
```

- [ ] **Step 3: Add a hidden vertical line-break test**

Add:

```rust
#[test]
fn hidden_vertical_line_break_does_not_create_inline_control() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(0, NodeInput {
            display: Display::Block,
            writing_mode: WritingMode::VerticalRl,
            size: Size::new(Dimension::px(80.0), Dimension::AUTO),
            ..NodeInput::DEFAULT
        })
        .style(1, NodeInput {
            display: Display::InlineBlock,
            writing_mode: WritingMode::VerticalRl,
            size: Size::new(Dimension::px(10.0), Dimension::px(30.0)),
            ..NodeInput::DEFAULT
        })
        .line_break(
            2,
            LineBreakInput::new()
                .with_writing_mode(WritingMode::VerticalRl)
                .hidden(),
        )
        .style(3, NodeInput {
            display: Display::InlineBlock,
            writing_mode: WritingMode::VerticalRl,
            size: Size::new(Dimension::px(12.0), Dimension::px(16.0)),
            ..NodeInput::DEFAULT
        });

    compute_root(&mut tree, 0, Size::new(Available::definite(80.0), Available::MAX_CONTENT));
    round_layout(&mut tree, 0);

    assert_eq!(tree.final_layout(2).unwrap().size, Size::ZERO);
    assert_eq!(tree.final_layout(3).unwrap().location, Point::new(68.0, 30.0));
}
```

- [ ] **Step 4: Verify block tests fail before implementation**

Run:

```sh
cargo test -p surgeist-layout vertical_ -- --nocapture
```

Expected: new vertical line-break tests fail on the current horizontal-only gate.

- [ ] **Step 5: Rename and relax the line-break chokepoint**

In `src/block.rs`, rename `visible_horizontal_line_break` to
`visible_line_break_in_flow`. The function must receive the containing inline
flow's `WritingMode` and `Direction` from `constants`, not infer vertical clear
support from the line-break node alone.

```rust
fn visible_line_break_in_flow<Tree>(
    tree: &Tree,
    child: <Tree as Traverse>::Node,
    flow_writing_mode: WritingMode,
    flow_direction: Direction,
) -> Option<LineBreakInputOf<<Tree as Traverse>::Scalar>>
where
    Tree: Compute,
{
    let LayoutInputOf::LineBreak(line_break) = tree.layout_input(child) else {
        return None;
    };
    if line_break.display().is_none() {
        return None;
    }
    if flow_writing_mode != WritingMode::HorizontalTb && line_break.clear() != Clear::None {
        panic!("vertical line-break clear layout is not implemented");
    }
    if line_break.writing_mode() != flow_writing_mode || line_break.direction() != flow_direction {
        panic!("line-break flow must match containing inline flow");
    }
    Some(line_break)
}
```

This rejects mismatched line-break flow instead of adopting default-valued
fields. `LineBreakInputOf<S>` does not track whether a default-valued field was
authored explicitly, so layout cannot distinguish `LineBreakInput::new()` from
an explicitly authored `HorizontalTb`/`Ltr` line break. The layout-ready
contract for this phase is therefore strict: visible line-break inputs must
carry the containing inline flow before they reach layout.

Update callers and signatures:

- `atomic_inline_run_end` should accept `constants: &Constants<S>` or the
  containing writing mode and direction, then call
  `visible_line_break_in_flow(tree, children[index], constants.writing_mode, constants.direction);`
- both call sites of `atomic_inline_run_end` should pass `constants`;
- `layout_atomic_inline_run` should call
  `visible_line_break_in_flow(tree, child, constants.writing_mode, constants.direction).unwrap();`

- [ ] **Step 6: Keep clear segmentation horizontal-only**

Update `next_atomic_inline_clear_candidate` so it also receives the containing
writing mode and direction. Skip clear segmentation unless the containing flow
is horizontal:

```rust
if let Some(line_break) =
    visible_line_break_in_flow(tree, child, flow_writing_mode, flow_direction)
{
    if flow_writing_mode != WritingMode::HorizontalTb {
        continue;
    }
    let clear = line_break.clear();
    if clear != Clear::None {
        return Some(AtomicInlineClearCandidate {
            end: index + 1,
            clear,
        });
    }
}
```

Do not apply horizontal `FloatExclusions::clearance_y` to vertical flow in this plan.
`atomic_inline_run_contains_clear`, `layout_atomic_inline_run_with_clear`, and
`layout_atomic_inline_segments` should pass the containing flow through to this
candidate search from `context.constants`.

- [ ] **Step 7: Run focused block checks**

Run:

```sh
cargo test -p surgeist-layout vertical_ -- --nocapture
cargo test -p surgeist-layout line_break -- --nocapture
cargo test -p surgeist-layout block_rtl_atomic_inline_run_ -- --nocapture
```

Expected: all pass. Horizontal clear and RTL tests must remain green.

## Task 4: Boundary Search And Full Verification

**Files:**
- Inspect only unless failures require task-local edits.

- [ ] **Step 1: Search for stale unsupported vertical forced-break text**

Run:

```sh
rg -n "forced atomic inline breaks are unsupported|vertical line-break layout is not implemented|vertical-lr layout" src tests plans/2026-07-08-surgeist-layout-vertical-forced-break-implementation.md
```

Expected:

- no stale forced-break unsupported panic remains in `src/inline.rs`;
- no stale vertical line-break panic remains in `src/block.rs`;
- the only remaining vertical unsupported source text should be the explicit vertical clear panic and documentation in this plan or future-phase docs.

- [ ] **Step 2: Run final checks**

Run:

```sh
cargo test -p surgeist-layout
cargo clippy -p surgeist-layout --all-targets -- -D warnings
cargo fmt --check
git diff --check
git status --short --branch
```

Expected: all checks pass. `git status` should show only task-owned edits before the coordinator commits.

- [ ] **Step 3: Scoped review for the task group**

Ask a clean-context reviewer to inspect the task diff against:

- this plan;
- `plans/specs/2026-07-08-surgeist-layout-inline-control-item-spec.md`;
- `plans/2026-07-08-surgeist-layout-inline-control-item-sequencing.md`;
- `guidance/surgeist-rust-modeling-guide.md`;
- crate boundary requirements in `AGENTS.md`.

The reviewer must confirm:

- vertical forced breaks are implemented as logical-axis line construction;
- vertical-lr and vertical-rl do not have separate `<br>` special cases;
- layout still consumes layout-ready metrics and does not derive font/text data;
- vertical clear remains explicitly unsupported rather than silently wrong;
- horizontal behavior, clear, and RTL placement are preserved.

- [ ] **Step 4: Commit after clean scoped review**

After the worker result, focused checks, and scoped review are clean, commit:

```sh
git add src/inline.rs src/inline_tests.rs src/block.rs src/block_tests.rs
git commit -m "Support vertical forced line breaks"
```

## Final Holistic Review Gate

After the scoped task commit, assign a final clean-context holistic reviewer. The final reviewer must inspect the complete implementation against:

- this implementation plan;
- the inline control item spec;
- the sequencing plan;
- modeling guidance;
- the actual code, even where the plan may be incomplete.

Completion requires the final reviewer to come back clean and these final checks to pass:

```sh
cargo test -p surgeist-layout
cargo clippy -p surgeist-layout --all-targets -- -D warnings
cargo fmt --check
git diff --check
git status --short --branch
```

## Cross-Crate Ledger

This plan should not need a blocking cross-crate entry. Root/style/retained still
own real HTML `<br>` classification, computed writing-mode/direction/clear/
vertical-align lowering, and text/font metric production. Because layout now
requires visible line-break inputs to match the containing inline flow exactly,
root/style integration must provide `LineBreakInputOf<S>` with the resolved
writing mode and direction before vertical `<br>` cases can pass through layout.
Phase 6 will decide which browser parity vertical `<br>` fixtures can move out
of unsupported classification once this engine behavior exists.
