# Surgeist Atomic Inline Oracle Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement a test-only oracle for atomic inline layout before production inline-block and inline-grid engine work begins.

**Architecture:** Add a focused `inline.rs` oracle module under `crates/surgeist/tests/support/oracle`. The oracle consumes explicit atomic inline item facts and returns deterministic line boxes, item offsets, intrinsic widths, baselines, and wrapper facts; it does not parse CSS, traverse trees, measure text, call production layout, or know about browser fixture XML. Grid/subgrid/grid-lanes oracle phases can consume the resulting wrapper facts later, but atomic inline line construction remains independent from grid track sizing.

**Tech Stack:** Rust test support under `crates/surgeist/tests/support/oracle`, pure oracle tests in `crates/surgeist/tests/oracle.rs`, composed production/oracle tests in `crates/surgeist/tests/layout_oracle.rs` after production support lands, verification with `cargo test -p surgeist --test oracle`, `cargo test -p surgeist --test layout_oracle`, `cargo fmt --check`, and `git diff --check`.

---

## Source References

- Engine plan that this oracle plan must precede:
  - `docs/superpowers/plans/2026-06-17-surgeist-inline-block-inline-grid-implementation.md`
- Existing oracle modules:
  - `crates/surgeist/tests/support/oracle/mod.rs`
  - `crates/surgeist/tests/support/oracle/grid/mod.rs`
  - `crates/surgeist/tests/support/oracle/grid/baseline.rs`
  - `crates/surgeist/tests/support/oracle/grid/contributions.rs`
  - `crates/surgeist/tests/support/oracle/grid/lanes.rs`
  - `crates/surgeist/tests/support/oracle/grid/scenario.rs`
  - `crates/surgeist/tests/oracle.rs`
  - `crates/surgeist/tests/layout_oracle.rs`
- Browser references already pulled into `tmp`:
  - `tmp/WebKit/Source/WebCore/rendering/RenderBlock.cpp`
  - `tmp/WebKit/Source/WebCore/rendering/RenderObjectInlines.h`
  - `tmp/WebKit/Source/WebCore/rendering/RenderElement.cpp`
  - `tmp/WebKit/Source/WebCore/rendering/RenderGrid.cpp`
  - `tmp/chromium-inline/third_party/blink/renderer/core/layout/inline/inline_node.cc`
  - `tmp/chromium-inline/third_party/blink/renderer/core/layout/inline/inline_items_builder.cc`
  - `tmp/chromium-inline/third_party/blink/renderer/core/layout/layout_block.cc`
  - `tmp/chromium-inline/third_party/blink/renderer/core/layout/grid/grid_layout_algorithm.cc`

---

## Guardrails

- [ ] Do not import production layout code into `support::oracle::inline`.
- [ ] Do not parse CSS, HTML, XML, or style declarations inside the oracle.
- [ ] Do not measure text or traverse production/test trees in the oracle.
- [ ] Do not model non-atomic inline spans, bidi reordering, ruby, editable caret behavior, selection painting, or inline text shaping in this plan.
- [ ] Do not alias `inline-grid` or `inline-grid-lanes` to block-level grid behavior. The oracle must preserve outer atomic-inline participation and inner formatting context as separate facts.
- [ ] Keep the oracle explicit: all item sizes, margins, baselines, inner display kinds, and intrinsic contributions are input facts.
- [ ] Commit after logical checkpoints with short concrete messages.

---

## Oracle Model

The oracle should answer:

```text
Given explicit atomic inline item facts, what line boxes, item offsets, intrinsic widths, line baselines, and wrapper facts should atomic inline layout produce?
```

It should not answer:

```text
Given a styled document tree, where should browser-compatible inline content be laid out?
```

Coordinate convention:

- All sizes are physical horizontal-tb values for this oracle phase.
- `item.size` is the atomic inline border-box size.
- `item.margin` is physical margin around the atomic inline border box.
- `first_baseline` is a distance from the atomic inline border-box top edge.
- Missing `first_baseline` synthesizes to the border-box bottom edge.
- `advance = margin.left + size.width + margin.right`.
- `baseline = margin.top + first_baseline`.
- `descent = margin.bottom + size.height - first_baseline`.
- `line_baseline = max(item baseline)`.
- `line_descent = max(item descent)`.
- `line_height = line_baseline + line_descent`.
- Item border-box x is `line_start + inline_cursor + margin.left`.
- Item border-box y is `line_top + line_baseline - baseline + margin.top`.

Intrinsic widths:

- `max-content` width is the sum of item advances.
- `min-content` width is the maximum item advance.
- Definite inline layout wraps only between atomic items.
- A too-wide item may overflow its line; it is not split.

Wrapper facts:

- `inline-block` has atomic inline participation, `InlineOuterDisplay::InlineBlock`, and inner formatting context `Block`.
- `inline-grid` has atomic inline participation, `InlineOuterDisplay::InlineGrid`, and inner formatting context `Grid`.
- `inline-grid-lanes` has atomic inline participation, `InlineOuterDisplay::InlineGridLanes`, and inner formatting context `GridLanes`.
- Atomic participation is exposed by `AtomicInlineWrapperFacts::as_item()`, which turns the wrapper into an `AtomicInlineItemFacts` line item.
- The wrapper's line contribution is always its outer border-box plus margins, regardless of inner context.

---

## File Structure

- Create `crates/surgeist/tests/support/oracle/inline.rs`
  - Owns atomic inline vocabulary, line layout reports, intrinsic width functions, and wrapper facts.
- Modify `crates/surgeist/tests/support/oracle/mod.rs`
  - Exports the `inline` module. Keep inline oracle APIs under `support::oracle::inline`, mirroring the existing `support::oracle::grid` namespace.
- Modify `crates/surgeist/tests/oracle.rs`
  - Adds pure oracle tests for line layout, wrapping, baselines, intrinsic widths, and wrapper facts.
- Do not modify `crates/surgeist/tests/layout_oracle.rs` in this oracle-only plan.
  - Production comparison tests belong in the engine implementation plan after production atomic inline behavior exists, so this plan's verification can stay fully passing.
- Modify `docs/superpowers/plans/2026-06-17-surgeist-inline-block-inline-grid-implementation.md`
  - Adds this oracle plan as a prerequisite before engine Task 1.

---

## Task 1: Add Atomic Inline Oracle Vocabulary

**Files:**
- Create: `crates/surgeist/tests/support/oracle/inline.rs`
- Modify: `crates/surgeist/tests/support/oracle/mod.rs`
- Modify: `crates/surgeist/tests/oracle.rs`

- [ ] Add failing tests for atomic inline item metrics and synthesized baselines.

Add to `crates/surgeist/tests/oracle.rs`:

```rust
#[test]
fn oracle_atomic_inline_item_metrics_include_margins_and_baseline() {
    let item = support::oracle::inline::AtomicInlineItemFacts {
        id: "a",
        size: support::oracle::inline::InlineSize::new(20.0, 10.0),
        margin: support::oracle::inline::InlineEdges {
            top: 2.0,
            right: 3.0,
            bottom: 4.0,
            left: 5.0,
        },
        first_baseline: Some(7.0),
    };

    let metrics = support::oracle::inline::AtomicInlineMetrics::from_item(item);

    assert_eq!(metrics.advance, 28.0);
    assert_eq!(metrics.baseline, 9.0);
    assert_eq!(metrics.descent, 7.0);
    assert_eq!(metrics.margin_box_size, support::oracle::inline::InlineSize::new(28.0, 16.0));
}

#[test]
fn oracle_atomic_inline_item_synthesizes_missing_baseline_from_bottom_edge() {
    let item = support::oracle::inline::AtomicInlineItemFacts {
        id: "a",
        size: support::oracle::inline::InlineSize::new(20.0, 10.0),
        margin: support::oracle::inline::InlineEdges::ZERO,
        first_baseline: None,
    };

    let metrics = support::oracle::inline::AtomicInlineMetrics::from_item(item);

    assert_eq!(metrics.baseline, 10.0);
    assert_eq!(metrics.descent, 0.0);
    assert!(metrics.synthesized_baseline);
}
```

- [ ] Run the failing tests.

```bash
cargo test -p surgeist --test oracle oracle_atomic_inline_item_metrics_include_margins_and_baseline
cargo test -p surgeist --test oracle oracle_atomic_inline_item_synthesizes_missing_baseline_from_bottom_edge
```

Expected: compile failure because `support::oracle::inline::AtomicInlineItemFacts` and related types do not exist.

- [ ] Create `crates/surgeist/tests/support/oracle/inline.rs` with base vocabulary.

Expected code:

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct InlineSize {
    pub width: f32,
    pub height: f32,
}

impl InlineSize {
    pub const ZERO: Self = Self::new(0.0, 0.0);

    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct InlinePoint {
    pub x: f32,
    pub y: f32,
}

impl InlinePoint {
    pub const ZERO: Self = Self::new(0.0, 0.0);

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct InlineEdges {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl InlineEdges {
    pub const ZERO: Self = Self::all(0.0);

    pub const fn all(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    pub const fn horizontal_sum(self) -> f32 {
        self.left + self.right
    }

    pub const fn vertical_sum(self) -> f32 {
        self.top + self.bottom
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AtomicInlineItemFacts {
    pub id: &'static str,
    pub size: InlineSize,
    pub margin: InlineEdges,
    pub first_baseline: Option<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AtomicInlineMetrics {
    pub id: &'static str,
    pub advance: f32,
    pub baseline: f32,
    pub descent: f32,
    pub margin_box_size: InlineSize,
    pub synthesized_baseline: bool,
}

impl AtomicInlineMetrics {
    pub fn from_item(item: AtomicInlineItemFacts) -> Self {
        let first_baseline = item.first_baseline.unwrap_or(item.size.height);
        Self {
            id: item.id,
            advance: item.margin.left + item.size.width + item.margin.right,
            baseline: item.margin.top + first_baseline,
            descent: item.margin.bottom + item.size.height - first_baseline,
            margin_box_size: InlineSize::new(
                item.size.width + item.margin.horizontal_sum(),
                item.size.height + item.margin.vertical_sum(),
            ),
            synthesized_baseline: item.first_baseline.is_none(),
        }
    }
}
```

- [ ] Export the module from `crates/surgeist/tests/support/oracle/mod.rs`.

Expected code:

```rust
pub mod grid;
pub mod inline;
```

- [ ] Run the tests.

```bash
cargo test -p surgeist --test oracle oracle_atomic_inline_item_metrics_include_margins_and_baseline
cargo test -p surgeist --test oracle oracle_atomic_inline_item_synthesizes_missing_baseline_from_bottom_edge
```

Expected: pass.

- [ ] Commit.

```bash
git add crates/surgeist/tests/support/oracle/inline.rs crates/surgeist/tests/support/oracle/mod.rs crates/surgeist/tests/oracle.rs
git commit -m "Add atomic inline oracle vocabulary"
```

---

## Task 2: Lay Out A Single Atomic Inline Line

**Files:**
- Modify: `crates/surgeist/tests/support/oracle/inline.rs`
- Modify: `crates/surgeist/tests/support/oracle/mod.rs`
- Modify: `crates/surgeist/tests/oracle.rs`

- [ ] Add failing tests for one-line baseline alignment.

Add to `crates/surgeist/tests/oracle.rs`:

```rust
#[test]
fn oracle_atomic_inline_line_aligns_items_to_max_baseline() {
    let report = support::oracle::inline::layout_atomic_inline(
        support::oracle::inline::AtomicInlineInput {
            available_width: support::oracle::inline::InlineAvailable::Definite(200.0),
            items: vec![
                support::oracle::inline::AtomicInlineItemFacts {
                    id: "short",
                    size: support::oracle::inline::InlineSize::new(20.0, 10.0),
                    margin: support::oracle::inline::InlineEdges::ZERO,
                    first_baseline: Some(7.0),
                },
                support::oracle::inline::AtomicInlineItemFacts {
                    id: "tall",
                    size: support::oracle::inline::InlineSize::new(10.0, 20.0),
                    margin: support::oracle::inline::InlineEdges::ZERO,
                    first_baseline: Some(12.0),
                },
            ],
        },
    );

    assert_eq!(report.size, support::oracle::inline::InlineSize::new(30.0, 20.0));
    assert_eq!(report.first_baseline, Some(12.0));
    assert_eq!(report.last_baseline, Some(12.0));
    assert_eq!(report.lines.len(), 1);
    assert_eq!(
        report.lines[0],
        support::oracle::inline::AtomicInlineLine {
            start_item: 0,
            end_item: 2,
            y: 0.0,
            width: 30.0,
            height: 20.0,
            baseline: 12.0,
            descent: 8.0,
        }
    );
    assert_eq!(report.items[0].id, "short");
    assert_eq!(report.items[0].location, support::oracle::inline::InlinePoint::new(0.0, 5.0));
    assert_eq!(report.items[1].location, support::oracle::inline::InlinePoint::new(20.0, 0.0));
}

#[test]
fn oracle_atomic_inline_line_positions_margin_boxes_and_border_boxes() {
    let report = support::oracle::inline::layout_atomic_inline(
        support::oracle::inline::AtomicInlineInput {
            available_width: support::oracle::inline::InlineAvailable::Definite(200.0),
            items: vec![
                support::oracle::inline::AtomicInlineItemFacts {
                    id: "a",
                    size: support::oracle::inline::InlineSize::new(20.0, 10.0),
                    margin: support::oracle::inline::InlineEdges {
                        top: 2.0,
                        right: 3.0,
                        bottom: 4.0,
                        left: 5.0,
                    },
                    first_baseline: Some(7.0),
                },
                support::oracle::inline::AtomicInlineItemFacts {
                    id: "b",
                    size: support::oracle::inline::InlineSize::new(10.0, 20.0),
                    margin: support::oracle::inline::InlineEdges {
                        top: 1.0,
                        right: 2.0,
                        bottom: 3.0,
                        left: 4.0,
                    },
                    first_baseline: Some(12.0),
                },
            ],
        },
    );

    assert_eq!(report.size, support::oracle::inline::InlineSize::new(44.0, 24.0));
    assert_eq!(
        report.lines[0],
        support::oracle::inline::AtomicInlineLine {
            start_item: 0,
            end_item: 2,
            y: 0.0,
            width: 44.0,
            height: 24.0,
            baseline: 13.0,
            descent: 11.0,
        }
    );
    assert_eq!(report.items[0].location, support::oracle::inline::InlinePoint::new(5.0, 6.0));
    assert_eq!(report.items[1].location, support::oracle::inline::InlinePoint::new(32.0, 1.0));
}
```

- [ ] Run the failing test.

```bash
cargo test -p surgeist --test oracle oracle_atomic_inline_line_aligns_items_to_max_baseline
cargo test -p surgeist --test oracle oracle_atomic_inline_line_positions_margin_boxes_and_border_boxes
```

Expected: compile failure because `AtomicInlineInput`, `InlineAvailable`, and `layout_atomic_inline` do not exist.

- [ ] Add line layout report types and one-line-capable implementation.

Expected code in `inline.rs`:

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InlineAvailable {
    Definite(f32),
    MinContent,
    MaxContent,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AtomicInlineInput {
    pub available_width: InlineAvailable,
    pub items: Vec<AtomicInlineItemFacts>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AtomicInlineLine {
    pub start_item: usize,
    pub end_item: usize,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub baseline: f32,
    pub descent: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AtomicInlinePositionedItem {
    pub id: &'static str,
    pub line_index: usize,
    pub location: InlinePoint,
    pub size: InlineSize,
    pub margin: InlineEdges,
    pub first_baseline: f32,
    pub synthesized_baseline: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AtomicInlineReport {
    pub size: InlineSize,
    pub first_baseline: Option<f32>,
    pub last_baseline: Option<f32>,
    pub lines: Vec<AtomicInlineLine>,
    pub items: Vec<AtomicInlinePositionedItem>,
}

pub fn layout_atomic_inline(input: AtomicInlineInput) -> AtomicInlineReport {
    let metrics = input
        .items
        .iter()
        .copied()
        .map(AtomicInlineMetrics::from_item)
        .collect::<Vec<_>>();
    let line_ranges = line_ranges(&metrics, input.available_width);
    build_report(&input.items, &metrics, &line_ranges)
}
```

Add private helpers:

```rust
fn line_ranges(metrics: &[AtomicInlineMetrics], available: InlineAvailable) -> Vec<(usize, usize)> {
    if metrics.is_empty() {
        return Vec::new();
    }
    let Some(width) = wrap_width(metrics, available) else {
        return vec![(0, metrics.len())];
    };

    let mut ranges = Vec::new();
    let mut start = 0;
    let mut current = 0.0;
    for (index, item) in metrics.iter().enumerate() {
        if index > start && current + item.advance > width {
            ranges.push((start, index));
            start = index;
            current = 0.0;
        }
        current += item.advance;
    }
    ranges.push((start, metrics.len()));
    ranges
}

fn wrap_width(metrics: &[AtomicInlineMetrics], available: InlineAvailable) -> Option<f32> {
    match available {
        InlineAvailable::Definite(width) => Some(width),
        InlineAvailable::MinContent => Some(
            metrics
                .iter()
                .map(|metrics| metrics.advance)
                .fold(0.0, f32::max),
        ),
        InlineAvailable::MaxContent => None,
    }
}
```

`line_ranges` should call `wrap_width`, not a helper that treats `MinContent` as unwrapped. `build_report` must use the coordinate convention from this plan: line baseline is maximum item baseline; line descent is maximum item descent; line height is baseline plus descent; item x advances by margin-box advance; item y aligns baselines. `AtomicInlineReport::size` is the max line width by total line height, including overflow from a too-wide atomic item.

- [ ] Keep the new types/functions public from `support::oracle::inline`; do not add top-level `support::oracle` re-exports.

- [ ] Run the test.

```bash
cargo test -p surgeist --test oracle oracle_atomic_inline_line_aligns_items_to_max_baseline
```

Expected: pass.

- [ ] Commit.

```bash
git add crates/surgeist/tests/support/oracle/inline.rs crates/surgeist/tests/support/oracle/mod.rs crates/surgeist/tests/oracle.rs
git commit -m "Add atomic inline line oracle"
```

---

## Task 3: Model Wrapping And Intrinsic Widths

**Files:**
- Modify: `crates/surgeist/tests/support/oracle/inline.rs`
- Modify: `crates/surgeist/tests/support/oracle/mod.rs`
- Modify: `crates/surgeist/tests/oracle.rs`

- [ ] Add failing tests for wrapping between atomic items and min/max content widths.

Add to `crates/surgeist/tests/oracle.rs`:

```rust
#[test]
fn oracle_atomic_inline_wraps_between_items_for_definite_width() {
    let item = |id| support::oracle::inline::AtomicInlineItemFacts {
        id,
        size: support::oracle::inline::InlineSize::new(20.0, 10.0),
        margin: support::oracle::inline::InlineEdges::ZERO,
        first_baseline: Some(10.0),
    };

    let report = support::oracle::inline::layout_atomic_inline(
        support::oracle::inline::AtomicInlineInput {
            available_width: support::oracle::inline::InlineAvailable::Definite(25.0),
            items: vec![item("a"), item("b")],
        },
    );

    assert_eq!(report.size, support::oracle::inline::InlineSize::new(20.0, 20.0));
    assert_eq!(report.lines.len(), 2);
    assert_eq!(
        report.lines[0],
        support::oracle::inline::AtomicInlineLine {
            start_item: 0,
            end_item: 1,
            y: 0.0,
            width: 20.0,
            height: 10.0,
            baseline: 10.0,
            descent: 0.0,
        }
    );
    assert_eq!(
        report.lines[1],
        support::oracle::inline::AtomicInlineLine {
            start_item: 1,
            end_item: 2,
            y: 10.0,
            width: 20.0,
            height: 10.0,
            baseline: 10.0,
            descent: 0.0,
        }
    );
    assert_eq!(report.items[0].location, support::oracle::inline::InlinePoint::new(0.0, 0.0));
    assert_eq!(report.items[1].location, support::oracle::inline::InlinePoint::new(0.0, 10.0));
    assert_eq!(report.first_baseline, Some(10.0));
    assert_eq!(report.last_baseline, Some(20.0));
}

#[test]
fn oracle_atomic_inline_intrinsic_widths_use_max_item_and_sum() {
    let items = vec![
        support::oracle::inline::AtomicInlineItemFacts {
            id: "a",
            size: support::oracle::inline::InlineSize::new(25.0, 10.0),
            margin: support::oracle::inline::InlineEdges::ZERO,
            first_baseline: Some(10.0),
        },
        support::oracle::inline::AtomicInlineItemFacts {
            id: "b",
            size: support::oracle::inline::InlineSize::new(100.0, 10.0),
            margin: support::oracle::inline::InlineEdges::ZERO,
            first_baseline: Some(10.0),
        },
        support::oracle::inline::AtomicInlineItemFacts {
            id: "c",
            size: support::oracle::inline::InlineSize::new(95.0, 10.0),
            margin: support::oracle::inline::InlineEdges {
                left: 10.0,
                right: 7.0,
                ..support::oracle::inline::InlineEdges::ZERO
            },
            first_baseline: Some(10.0),
        },
    ];

    assert_eq!(support::oracle::inline::atomic_inline_min_content_width(&items), 112.0);
    assert_eq!(support::oracle::inline::atomic_inline_max_content_width(&items), 237.0);
}

#[test]
fn oracle_atomic_inline_min_content_wraps_at_max_item_advance() {
    let items = vec![
        support::oracle::inline::AtomicInlineItemFacts {
            id: "wide",
            size: support::oracle::inline::InlineSize::new(95.0, 10.0),
            margin: support::oracle::inline::InlineEdges {
                left: 10.0,
                right: 7.0,
                ..support::oracle::inline::InlineEdges::ZERO
            },
            first_baseline: Some(10.0),
        },
        support::oracle::inline::AtomicInlineItemFacts {
            id: "next",
            size: support::oracle::inline::InlineSize::new(50.0, 10.0),
            margin: support::oracle::inline::InlineEdges::ZERO,
            first_baseline: Some(10.0),
        },
    ];

    let report = support::oracle::inline::layout_atomic_inline(
        support::oracle::inline::AtomicInlineInput {
            available_width: support::oracle::inline::InlineAvailable::MinContent,
            items,
        },
    );

    assert_eq!(report.size, support::oracle::inline::InlineSize::new(112.0, 20.0));
    assert_eq!(report.lines.len(), 2);
    assert_eq!(report.lines[0].width, 112.0);
    assert_eq!(report.lines[1].width, 50.0);
}

#[test]
fn oracle_atomic_inline_too_wide_item_overflows_without_empty_line() {
    let item = |id, width| support::oracle::inline::AtomicInlineItemFacts {
        id,
        size: support::oracle::inline::InlineSize::new(width, 10.0),
        margin: support::oracle::inline::InlineEdges::ZERO,
        first_baseline: Some(10.0),
    };

    let report = support::oracle::inline::layout_atomic_inline(
        support::oracle::inline::AtomicInlineInput {
            available_width: support::oracle::inline::InlineAvailable::Definite(25.0),
            items: vec![item("wide", 40.0), item("next", 10.0)],
        },
    );

    assert_eq!(report.size, support::oracle::inline::InlineSize::new(40.0, 20.0));
    assert_eq!(report.lines.len(), 2);
    assert_eq!(report.lines[0].start_item, 0);
    assert_eq!(report.lines[0].end_item, 1);
    assert_eq!(report.lines[0].width, 40.0);
    assert_eq!(report.lines[1].start_item, 1);
    assert_eq!(report.lines[1].end_item, 2);
    assert_eq!(report.lines[1].width, 10.0);
}
```

- [ ] Run the failing tests.

```bash
cargo test -p surgeist --test oracle oracle_atomic_inline_wraps_between_items_for_definite_width
cargo test -p surgeist --test oracle oracle_atomic_inline_intrinsic_widths_use_max_item_and_sum
cargo test -p surgeist --test oracle oracle_atomic_inline_min_content_wraps_at_max_item_advance
cargo test -p surgeist --test oracle oracle_atomic_inline_too_wide_item_overflows_without_empty_line
```

Expected: first test may fail if Task 2 only handled one line; second fails because intrinsic helpers do not exist.

- [ ] Implement wrapping and intrinsic helpers.

Expected public functions:

```rust
pub fn atomic_inline_min_content_width(items: &[AtomicInlineItemFacts]) -> f32 {
    items
        .iter()
        .copied()
        .map(AtomicInlineMetrics::from_item)
        .map(|metrics| metrics.advance)
        .fold(0.0, f32::max)
}

pub fn atomic_inline_max_content_width(items: &[AtomicInlineItemFacts]) -> f32 {
    items
        .iter()
        .copied()
        .map(AtomicInlineMetrics::from_item)
        .map(|metrics| metrics.advance)
        .sum()
}
```

`layout_atomic_inline` must use:

```rust
match input.available_width {
    InlineAvailable::Definite(width) => wrap at item boundaries using width,
    InlineAvailable::MinContent => lay out using atomic_inline_min_content_width(&input.items),
    InlineAvailable::MaxContent => lay out all items on one line,
}
```

- [ ] Keep intrinsic helpers public from `support::oracle::inline`; do not add top-level `support::oracle` re-exports.

- [ ] Run the focused tests.

```bash
cargo test -p surgeist --test oracle oracle_atomic_inline_wraps_between_items_for_definite_width
cargo test -p surgeist --test oracle oracle_atomic_inline_intrinsic_widths_use_max_item_and_sum
cargo test -p surgeist --test oracle oracle_atomic_inline_min_content_wraps_at_max_item_advance
cargo test -p surgeist --test oracle oracle_atomic_inline_too_wide_item_overflows_without_empty_line
```

Expected: pass.

- [ ] Commit.

```bash
git add crates/surgeist/tests/support/oracle/inline.rs crates/surgeist/tests/support/oracle/mod.rs crates/surgeist/tests/oracle.rs
git commit -m "Model atomic inline wrapping"
```

---

## Task 4: Add Atomic Inline Wrapper Facts

**Files:**
- Modify: `crates/surgeist/tests/support/oracle/inline.rs`
- Modify: `crates/surgeist/tests/support/oracle/mod.rs`
- Modify: `crates/surgeist/tests/oracle.rs`

- [ ] Add failing tests for inline-block, inline-grid, and inline-grid-lanes wrapper facts.

Add to `crates/surgeist/tests/oracle.rs`:

```rust
#[test]
fn oracle_atomic_inline_wrapper_preserves_outer_and_inner_display_roles() {
    let cases = [
        (
            support::oracle::inline::InlineOuterDisplay::InlineBlock,
            support::oracle::inline::InnerFormattingContext::Block,
        ),
        (
            support::oracle::inline::InlineOuterDisplay::InlineGrid,
            support::oracle::inline::InnerFormattingContext::Grid,
        ),
        (
            support::oracle::inline::InlineOuterDisplay::InlineGridLanes,
            support::oracle::inline::InnerFormattingContext::GridLanes,
        ),
    ];

    for (outer_display, inner_context) in cases {
        let wrapper = support::oracle::inline::AtomicInlineWrapperFacts::new(
            "wrapper",
            outer_display,
            support::oracle::inline::InlineSize::new(40.0, 20.0),
            support::oracle::inline::InlineEdges::ZERO,
            Some(15.0),
        );

        assert_eq!(wrapper.outer_display, outer_display);
        assert_eq!(wrapper.inner_context, inner_context);
        assert_eq!(
            wrapper.as_item(),
            support::oracle::inline::AtomicInlineItemFacts {
                id: "wrapper",
                size: support::oracle::inline::InlineSize::new(40.0, 20.0),
                margin: support::oracle::inline::InlineEdges::ZERO,
                first_baseline: Some(15.0),
            }
        );
    }
}

#[test]
fn oracle_atomic_inline_wrapper_metrics_use_outer_box_and_margins() {
    let cases = [
        support::oracle::inline::InlineOuterDisplay::InlineBlock,
        support::oracle::inline::InlineOuterDisplay::InlineGrid,
        support::oracle::inline::InlineOuterDisplay::InlineGridLanes,
    ];

    for outer_display in cases {
        let wrapper = support::oracle::inline::AtomicInlineWrapperFacts::new(
            "wrapper",
            outer_display,
            support::oracle::inline::InlineSize::new(40.0, 20.0),
            support::oracle::inline::InlineEdges {
                top: 2.0,
                right: 3.0,
                bottom: 4.0,
                left: 5.0,
            },
            Some(15.0),
        );

        let metrics = support::oracle::inline::AtomicInlineMetrics::from_item(wrapper.as_item());

        assert_eq!(metrics.advance, 48.0);
        assert_eq!(metrics.baseline, 17.0);
        assert_eq!(metrics.descent, 9.0);
        assert_eq!(metrics.margin_box_size, support::oracle::inline::InlineSize::new(48.0, 26.0));
    }
}
```

- [ ] Run the failing test.

```bash
cargo test -p surgeist --test oracle oracle_atomic_inline_wrapper_preserves_outer_and_inner_display_roles
cargo test -p surgeist --test oracle oracle_atomic_inline_wrapper_metrics_use_outer_box_and_margins
```

Expected: compile failure because wrapper facts do not exist.

- [ ] Implement wrapper facts.

Expected code:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InlineOuterDisplay {
    InlineBlock,
    InlineGrid,
    InlineGridLanes,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InnerFormattingContext {
    Block,
    Grid,
    GridLanes,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AtomicInlineWrapperFacts {
    pub id: &'static str,
    pub outer_display: InlineOuterDisplay,
    pub inner_context: InnerFormattingContext,
    pub outer_size: InlineSize,
    pub margin: InlineEdges,
    pub first_baseline: Option<f32>,
}

impl AtomicInlineWrapperFacts {
    pub fn new(
        id: &'static str,
        outer_display: InlineOuterDisplay,
        outer_size: InlineSize,
        margin: InlineEdges,
        first_baseline: Option<f32>,
    ) -> Self {
        Self {
            id,
            outer_display,
            inner_context: outer_display.inner_context(),
            outer_size,
            margin,
            first_baseline,
        }
    }

    pub fn as_item(self) -> AtomicInlineItemFacts {
        AtomicInlineItemFacts {
            id: self.id,
            size: self.outer_size,
            margin: self.margin,
            first_baseline: self.first_baseline,
        }
    }
}

impl InlineOuterDisplay {
    pub const fn inner_context(self) -> InnerFormattingContext {
        match self {
            Self::InlineBlock => InnerFormattingContext::Block,
            Self::InlineGrid => InnerFormattingContext::Grid,
            Self::InlineGridLanes => InnerFormattingContext::GridLanes,
        }
    }
}
```

- [ ] Keep wrapper types public from `support::oracle::inline`; do not add top-level `support::oracle` re-exports.

- [ ] Run the test.

```bash
cargo test -p surgeist --test oracle oracle_atomic_inline_wrapper_preserves_outer_and_inner_display_roles
cargo test -p surgeist --test oracle oracle_atomic_inline_wrapper_metrics_use_outer_box_and_margins
```

Expected: pass.

- [ ] Commit.

```bash
git add crates/surgeist/tests/support/oracle/inline.rs crates/surgeist/tests/support/oracle/mod.rs crates/surgeist/tests/oracle.rs
git commit -m "Add atomic inline wrapper oracle"
```

---

## Task 5: Bridge Wrapper Facts To Grid Contribution Inputs

**Files:**
- Modify: `crates/surgeist/tests/support/oracle/inline.rs`
- Modify: `crates/surgeist/tests/support/oracle/mod.rs`
- Modify: `crates/surgeist/tests/oracle.rs`

- [ ] Add failing tests proving wrapper facts can feed grid contribution phases without losing outer inline identity.

Add to `crates/surgeist/tests/oracle.rs`:

```rust
#[test]
fn oracle_atomic_inline_wrapper_produces_grid_contribution_facts() {
    let wrapper = support::oracle::inline::AtomicInlineWrapperFacts::new(
        "inline-grid",
        support::oracle::inline::InlineOuterDisplay::InlineGrid,
        support::oracle::inline::InlineSize::new(80.0, 30.0),
        support::oracle::inline::InlineEdges::ZERO,
        Some(24.0),
    );

    let contribution = support::oracle::inline::atomic_inline_grid_item_facts(
        wrapper,
        support::oracle::grid::GridArea::new(1, 1, 1, 1),
        80.0,
        80.0,
    );

    assert_eq!(contribution.id, "inline-grid");
    assert_eq!(contribution.outer_display, support::oracle::inline::InlineOuterDisplay::InlineGrid);
    assert_eq!(contribution.inner_context, support::oracle::inline::InnerFormattingContext::Grid);
    assert_eq!(contribution.item.area, support::oracle::grid::GridArea::new(1, 1, 1, 1));
    assert_eq!(contribution.item.min_content, 80.0);
    assert_eq!(contribution.item.max_content, 80.0);
    assert_eq!(
        contribution.item.preferred,
        support::oracle::grid::ContributionSize::Definite(80.0)
    );
    assert_eq!(contribution.item.margin_before, 0.0);
    assert_eq!(contribution.item.margin_after, 0.0);
    assert_eq!(contribution.item.contributions().max_content, 80.0);
}

#[test]
fn oracle_atomic_inline_grid_lanes_contribution_preserves_margins() {
    let wrapper = support::oracle::inline::AtomicInlineWrapperFacts::new(
        "inline-grid-lanes",
        support::oracle::inline::InlineOuterDisplay::InlineGridLanes,
        support::oracle::inline::InlineSize::new(80.0, 30.0),
        support::oracle::inline::InlineEdges {
            left: 5.0,
            right: 7.0,
            ..support::oracle::inline::InlineEdges::ZERO
        },
        Some(24.0),
    );

    let contribution = support::oracle::inline::atomic_inline_grid_item_facts(
        wrapper,
        support::oracle::grid::GridArea::new(1, 1, 1, 1),
        60.0,
        80.0,
    );

    assert_eq!(contribution.id, "inline-grid-lanes");
    assert_eq!(
        contribution.outer_display,
        support::oracle::inline::InlineOuterDisplay::InlineGridLanes
    );
    assert_eq!(
        contribution.inner_context,
        support::oracle::inline::InnerFormattingContext::GridLanes
    );
    assert_eq!(contribution.item.margin_before, 5.0);
    assert_eq!(contribution.item.margin_after, 7.0);
    assert_eq!(contribution.item.contributions().min_content, 72.0);
    assert_eq!(contribution.item.contributions().max_content, 92.0);
}
```

- [ ] Run the failing test.

```bash
cargo test -p surgeist --test oracle oracle_atomic_inline_wrapper_produces_grid_contribution_facts
cargo test -p surgeist --test oracle oracle_atomic_inline_grid_lanes_contribution_preserves_margins
```

Expected: compile failure because contribution adapter facts do not exist.

- [ ] Implement contribution facts in `inline.rs`.

Expected code:

```rust
use super::grid::{ContributionSize, GridArea, ItemContributionFacts};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AtomicInlineGridItemFacts {
    pub id: &'static str,
    pub outer_display: InlineOuterDisplay,
    pub inner_context: InnerFormattingContext,
    pub item: ItemContributionFacts,
}

pub fn atomic_inline_grid_item_facts(
    wrapper: AtomicInlineWrapperFacts,
    area: GridArea,
    min_content_inline_size: f32,
    max_content_inline_size: f32,
) -> AtomicInlineGridItemFacts {
    AtomicInlineGridItemFacts {
        id: wrapper.id,
        outer_display: wrapper.outer_display,
        inner_context: wrapper.inner_context,
        item: ItemContributionFacts {
            area,
            min_content: min_content_inline_size,
            max_content: max_content_inline_size,
            preferred: ContributionSize::Definite(wrapper.outer_size.width),
            min_size: ContributionSize::Auto,
            max_size: ContributionSize::Infinite,
            margin_before: wrapper.margin.left,
            margin_after: wrapper.margin.right,
            automatic_minimum_applies: true,
        },
    }
}
```

- [ ] Keep `AtomicInlineGridItemFacts` and `atomic_inline_grid_item_facts` public from `support::oracle::inline`; do not add top-level `support::oracle` re-exports.

- [ ] Run the test.

```bash
cargo test -p surgeist --test oracle oracle_atomic_inline_wrapper_produces_grid_contribution_facts
cargo test -p surgeist --test oracle oracle_atomic_inline_grid_lanes_contribution_preserves_margins
```

Expected: pass.

- [ ] Commit.

```bash
git add crates/surgeist/tests/support/oracle/inline.rs crates/surgeist/tests/support/oracle/mod.rs crates/surgeist/tests/oracle.rs
git commit -m "Bridge atomic inline oracle to grid facts"
```

---

## Task 6: Link The Engine Plan Without Adding Failing Hooks

**Files:**
- Modify: `docs/superpowers/plans/2026-06-17-surgeist-inline-block-inline-grid-implementation.md`

- [ ] Update the engine plan to make this oracle plan a prerequisite and to require production/oracle comparison tests when engine support lands.

Add near the top of `docs/superpowers/plans/2026-06-17-surgeist-inline-block-inline-grid-implementation.md`:

```markdown
## Prerequisite

Complete `docs/superpowers/plans/2026-06-17-surgeist-atomic-inline-oracle-implementation.md` before starting this engine plan. The engine implementation should use the oracle's atomic item facts, line reports, intrinsic width rules, and wrapper facts as the expected model for focused tests.

The first engine test task should add non-ignored production/oracle comparison tests in `crates/surgeist/tests/layout_oracle.rs` for inline-block, inline-grid, and inline-grid-lanes line placement. Do not add intentionally panicking ignored tests; keep all committed verification passing.
```

- [ ] Run verification for the plan link.

```bash
rg -n "atomic-inline-oracle" docs/superpowers/plans/2026-06-17-surgeist-inline-block-inline-grid-implementation.md
```

Expected: the `rg` command finds the prerequisite link.

- [ ] Commit.

```bash
git add docs/superpowers/plans/2026-06-17-surgeist-inline-block-inline-grid-implementation.md
git commit -m "Link atomic inline oracle plan"
```

---

## Final Verification

- [ ] Run pure oracle verification.

```bash
cargo test -p surgeist --test oracle oracle_atomic_inline
```

Expected: all atomic inline oracle tests pass.

- [ ] Run broad oracle suites.

```bash
cargo test -p surgeist --test oracle
cargo test -p surgeist --test layout_oracle
```

Expected: pass. Future production-comparison tests are not added by this oracle-only plan.

- [ ] Run formatting and diff checks.

```bash
cargo fmt --check
git diff --check
git status --short --branch
```

Expected: formatting and diff checks pass; status shows only intended files before commit or clean after commit.

---

## Self-Review

- Spec coverage: this plan covers oracle vocabulary, item metrics, single-line placement, wrapping, intrinsic widths, wrapper facts for `inline-block`/`inline-grid`/`inline-grid-lanes`, grid contribution bridging, and engine-plan prerequisite wiring.
- Placeholder scan: no unfinished placeholder steps are allowed. This plan must not add intentionally failing ignored tests; production comparison tests are added by the engine implementation plan when production behavior exists.
- Type consistency: `InlineSize`, `InlinePoint`, `InlineEdges`, `AtomicInlineItemFacts`, `AtomicInlineInput`, `AtomicInlineReport`, `InlineOuterDisplay`, and `AtomicInlineWrapperFacts` are introduced before use.
