# Logical Inline Axis Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Introduce an internal logical inline/block axis model for atomic inline layout without changing existing horizontal layout behavior or enabling vertical forced breaks yet.

**Architecture:** This implements Phase 4 from `plans/2026-07-08-surgeist-layout-inline-control-item-sequencing.md`. Add small internal logical-coordinate and axis-mapping helpers in `src/inline.rs`, make `AtomicInlineInput` carry `Direction`, and route horizontal atomic inline physical placement through the helper. Vertical forced line breaks remain explicitly unsupported; this plan prepares the coordinate model that Phase 5 will use.

**Tech Stack:** Rust 2024, `surgeist-layout`, internal inline layout module, existing `WritingMode`, `Direction`, `LayoutScalar`, Cargo test/clippy/fmt.

---

## Source References

- Specification: `plans/specs/2026-07-08-surgeist-layout-inline-control-item-spec.md`
- Sequencing: `plans/2026-07-08-surgeist-layout-inline-control-item-sequencing.md`
- Previous plan: `plans/2026-07-08-surgeist-layout-line-break-clear-implementation.md`
- Modeling guidance: `guidance/surgeist-rust-modeling-guide.md`
- Workflow: `AGENTS.md`

## Scope

This plan does:

- introduce internal logical inline point and size types for atomic inline layout;
- introduce one internal axis mapper for `HorizontalTb`, `VerticalRl`, and `VerticalLr`;
- model `Direction` as inline-axis direction, not as block-axis mapping;
- make `AtomicInlineInput` carry `Direction`;
- route existing horizontal item and forced-break output placement through the axis mapper;
- preserve existing horizontal LTR and RTL block output;
- preserve existing vertical-rl box-only output for current tests;
- preserve the existing vertical forced-break unsupported behavior.

This plan does not:

- support vertical forced line breaks;
- remove `visible_horizontal_line_break` or the block-level vertical line-break panic;
- add browser parity fixtures;
- parse HTML/CSS or derive font/text metrics;
- expose new public APIs;
- change `clear`, `vertical-align`, text shaping, or intrinsic sizing behavior.

## Files

- Modify: `src/inline.rs`
  - Add `LogicalInlinePointOf<S>` and `LogicalInlineSizeOf<S>`.
  - Add `InlineAxisMapping` with physical mapping methods.
  - Add `direction: Direction` to `AtomicInlineInput<S>`.
  - Route horizontal inline report item locations through `InlineAxisMapping`.
  - Keep vertical-rl box-only path behavior unchanged while making its block-axis mapping explicit through the new helper.
- Modify: `src/inline_tests.rs`
  - Add axis-mapping tests for all `WritingMode` variants.
  - Update `AtomicInlineInput` literals to include `direction`.
  - Add direct inline RTL report tests so horizontal mirroring is owned by the inline axis model.
- Modify: `src/block.rs`
  - Pass `constants.direction` into `AtomicInlineInput`.
  - Stop applying horizontal RTL mirroring outside the inline report; keep run offset and text-align behavior unchanged.
- Modify: `src/block_tests.rs`
  - Update or add only focused assertions needed to prove existing block-level horizontal RTL behavior remains unchanged.

No fixture, README, API artifact, or sibling-crate change is part of this plan.

## Semantics

Logical coordinates used by the new helper:

- logical inline axis is where content advances within one line;
- logical block axis is where successive lines stack;
- `LogicalInlineSizeOf<S>::inline` is the line advance extent;
- `LogicalInlineSizeOf<S>::block` is the line stacking extent;
- `Direction` mirrors placement along the logical inline axis only;
- `WritingMode` maps logical axes to physical x/y and chooses physical block-axis stacking.

Physical mapping required by this plan:

| Writing Mode | Direction | Logical point `(inline, block)` maps to physical |
| --- | --- | --- |
| `HorizontalTb` | `Ltr` | `x = inline`, `y = block` |
| `HorizontalTb` | `Rtl` | `x = line_inline_extent - inline - item_inline_extent`, `y = block` |
| `VerticalRl` | `Ltr` | `x = container_block_extent - block - item_block_extent`, `y = inline` |
| `VerticalRl` | `Rtl` | `x = container_block_extent - block - item_block_extent`, `y = line_inline_extent - inline - item_inline_extent` |
| `VerticalLr` | `Ltr` | `x = block`, `y = inline` |
| `VerticalLr` | `Rtl` | `x = block`, `y = line_inline_extent - inline - item_inline_extent` |

The helper may be used by vertical box-only layout in this plan. It must not make vertical forced breaks pass; Phase 5 owns that behavior expansion.

## Task 1: Add Logical Axis Mapping Types

**Files:**
- Modify: `src/inline.rs`
- Modify: `src/inline_tests.rs`

- [ ] **Step 1: Add logical coordinate types and axis mapper**

Add these definitions near `AtomicInlineBoxItem` in `src/inline.rs`:

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct LogicalInlinePointOf<S: LayoutScalar = DefaultScalar> {
    pub inline: S,
    pub block: S,
}

impl<S: LayoutScalar> LogicalInlinePointOf<S> {
    #[must_use]
    pub(super) const fn new(inline: S, block: S) -> Self {
        Self { inline, block }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct LogicalInlineSizeOf<S: LayoutScalar = DefaultScalar> {
    pub inline: S,
    pub block: S,
}

impl<S: LayoutScalar> LogicalInlineSizeOf<S> {
    #[must_use]
    pub(super) const fn new(inline: S, block: S) -> Self {
        Self { inline, block }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct InlineAxisMapping {
    writing_mode: WritingMode,
    direction: Direction,
}

impl InlineAxisMapping {
    #[must_use]
    pub(super) const fn new(writing_mode: WritingMode, direction: Direction) -> Self {
        Self {
            writing_mode,
            direction,
        }
    }

    #[cfg(test)]
    #[must_use]
    pub(super) fn physical_size<S: LayoutScalar>(
        self,
        logical: LogicalInlineSizeOf<S>,
    ) -> Size<S> {
        match self.writing_mode {
            WritingMode::HorizontalTb => Size::new(logical.inline, logical.block),
            WritingMode::VerticalRl | WritingMode::VerticalLr => {
                Size::new(logical.block, logical.inline)
            }
        }
    }

    #[must_use]
    pub(super) fn physical_item_origin<S: LayoutScalar>(
        self,
        logical_origin: LogicalInlinePointOf<S>,
        item_size: LogicalInlineSizeOf<S>,
        line_size: LogicalInlineSizeOf<S>,
        container_block_extent: S,
    ) -> Point<S> {
        let physical_inline = match self.direction {
            Direction::Ltr => logical_origin.inline,
            Direction::Rtl => line_size.inline - logical_origin.inline - item_size.inline,
        };
        match self.writing_mode {
            WritingMode::HorizontalTb => Point::new(physical_inline, logical_origin.block),
            WritingMode::VerticalRl => Point::new(
                container_block_extent - logical_origin.block - item_size.block,
                physical_inline,
            ),
            WritingMode::VerticalLr => Point::new(logical_origin.block, physical_inline),
        }
    }
}
```

Do not put HTML, CSS, font, text, or fixture concepts in these types.
`physical_size` is test-only in this phase because production placement only
needs point mapping; keeping it behind `#[cfg(test)]` avoids dead-code warnings
under `cargo clippy --all-targets -- -D warnings`.

- [ ] **Step 2: Add axis-mapping tests**

Add these tests near the top of `src/inline_tests.rs`, after the `forced_line_break` helper:

```rust
#[test]
fn inline_axis_mapping_maps_horizontal_tb_ltr() {
    let mapping = crate::inline::InlineAxisMapping::new(WritingMode::HorizontalTb, Direction::Ltr);

    assert_eq!(
        mapping.physical_size(crate::inline::LogicalInlineSizeOf::new(30.0, 12.0)),
        Size::new(30.0, 12.0)
    );
    assert_eq!(
        mapping.physical_item_origin(
            crate::inline::LogicalInlinePointOf::new(5.0, 7.0),
            crate::inline::LogicalInlineSizeOf::new(10.0, 4.0),
            crate::inline::LogicalInlineSizeOf::new(30.0, 12.0),
            80.0,
        ),
        Point::new(5.0, 7.0)
    );
}

#[test]
fn inline_axis_mapping_maps_horizontal_tb_rtl() {
    let mapping = crate::inline::InlineAxisMapping::new(WritingMode::HorizontalTb, Direction::Rtl);

    assert_eq!(
        mapping.physical_item_origin(
            crate::inline::LogicalInlinePointOf::new(5.0, 7.0),
            crate::inline::LogicalInlineSizeOf::new(10.0, 4.0),
            crate::inline::LogicalInlineSizeOf::new(30.0, 12.0),
            80.0,
        ),
        Point::new(15.0, 7.0)
    );
}

#[test]
fn inline_axis_mapping_maps_vertical_rl_ltr() {
    let mapping = crate::inline::InlineAxisMapping::new(WritingMode::VerticalRl, Direction::Ltr);

    assert_eq!(
        mapping.physical_size(crate::inline::LogicalInlineSizeOf::new(30.0, 12.0)),
        Size::new(12.0, 30.0)
    );
    assert_eq!(
        mapping.physical_item_origin(
            crate::inline::LogicalInlinePointOf::new(5.0, 7.0),
            crate::inline::LogicalInlineSizeOf::new(10.0, 4.0),
            crate::inline::LogicalInlineSizeOf::new(30.0, 12.0),
            80.0,
        ),
        Point::new(69.0, 5.0)
    );
}

#[test]
fn inline_axis_mapping_maps_vertical_rl_rtl() {
    let mapping = crate::inline::InlineAxisMapping::new(WritingMode::VerticalRl, Direction::Rtl);

    assert_eq!(
        mapping.physical_item_origin(
            crate::inline::LogicalInlinePointOf::new(5.0, 7.0),
            crate::inline::LogicalInlineSizeOf::new(10.0, 4.0),
            crate::inline::LogicalInlineSizeOf::new(30.0, 12.0),
            80.0,
        ),
        Point::new(69.0, 15.0)
    );
}

#[test]
fn inline_axis_mapping_maps_vertical_lr_ltr() {
    let mapping = crate::inline::InlineAxisMapping::new(WritingMode::VerticalLr, Direction::Ltr);

    assert_eq!(
        mapping.physical_size(crate::inline::LogicalInlineSizeOf::new(30.0, 12.0)),
        Size::new(12.0, 30.0)
    );
    assert_eq!(
        mapping.physical_item_origin(
            crate::inline::LogicalInlinePointOf::new(5.0, 7.0),
            crate::inline::LogicalInlineSizeOf::new(10.0, 4.0),
            crate::inline::LogicalInlineSizeOf::new(30.0, 12.0),
            80.0,
        ),
        Point::new(7.0, 5.0)
    );
}

#[test]
fn inline_axis_mapping_maps_vertical_lr_rtl() {
    let mapping = crate::inline::InlineAxisMapping::new(WritingMode::VerticalLr, Direction::Rtl);

    assert_eq!(
        mapping.physical_item_origin(
            crate::inline::LogicalInlinePointOf::new(5.0, 7.0),
            crate::inline::LogicalInlineSizeOf::new(10.0, 4.0),
            crate::inline::LogicalInlineSizeOf::new(30.0, 12.0),
            80.0,
        ),
        Point::new(7.0, 15.0)
    );
}
```

- [ ] **Step 3: Run axis tests**

Run:

```sh
cargo test -p surgeist-layout inline_axis_mapping_ -- --nocapture
```

Expected: all six new tests pass.

## Task 2: Move Horizontal Direction Mapping Into Inline Layout

**Files:**
- Modify: `src/inline.rs`
- Modify: `src/inline_tests.rs`
- Modify: `src/block.rs`
- Modify: `src/block_tests.rs`

- [ ] **Step 1: Add direction to atomic inline input**

Change `AtomicInlineInput` in `src/inline.rs` from:

```rust
pub(super) struct AtomicInlineInput<S: LayoutScalar = DefaultScalar> {
    pub available_width: AvailableOf<S>,
    pub writing_mode: WritingMode,
    pub items: Vec<AtomicInlineItem<S>>,
}
```

to:

```rust
pub(super) struct AtomicInlineInput<S: LayoutScalar = DefaultScalar> {
    pub available_width: AvailableOf<S>,
    pub writing_mode: WritingMode,
    pub direction: Direction,
    pub items: Vec<AtomicInlineItem<S>>,
}
```

Update every production `crate::inline::AtomicInlineInput { ... }` literal:

- in `src/block.rs`, set `direction: constants.direction`;
- in `src/inline_tests.rs`, set `direction: Direction::Ltr` unless the test explicitly covers RTL;
- do not edit `crate::test_support::oracle::inline::AtomicInlineInput` literals or the oracle helper type in `src/test_support/oracle/inline.rs`.

Use this search to find production literals and distinguish them from oracle literals:

```sh
rg -n "AtomicInlineInput \\{" src
```

Expected production literal hits include `src/block.rs` and the top-level direct
`layout_atomic_inline_items(AtomicInlineInput { ... })` tests in
`src/inline_tests.rs`. Hits under `inline::layout_atomic_inline(inline::AtomicInlineInput { ... })`
are the separate test-support oracle model and must remain unchanged.

- [ ] **Step 2: Add the vertical-lr forced-break guard**

Add an explicit `VerticalLr` unsupported guard near the top of
`layout_atomic_inline_items`, before the horizontal line builder runs:

```rust
if input.writing_mode == WritingMode::VerticalLr
    && input
        .items
        .iter()
        .any(|item| matches!(item, AtomicInlineItem::ForcedLineBreak(_)))
{
    panic!("forced atomic inline breaks are unsupported in vertical-lr layout");
}
```

Add this test in `src/inline_tests.rs` near
`atomic_inline_vertical_rl_places_line_against_right_edge`:

```rust
#[test]
#[should_panic(expected = "forced atomic inline breaks are unsupported in vertical-lr layout")]
fn atomic_inline_vertical_lr_forced_break_panics_until_modeled() {
    let metrics = InlineMetrics::from_line_height_and_baseline(10.0, 10.0).unwrap();

    let _ = layout_atomic_inline_items(AtomicInlineInput {
        available_width: Available::MAX_CONTENT,
        writing_mode: WritingMode::VerticalLr,
        direction: Direction::Ltr,
        items: vec![forced_line_break(0, metrics)],
    });
}
```

This guard must land before the fallback mapping below so vertical forced breaks
do not route through the horizontal line builder during any task-scoped review.

- [ ] **Step 3: Route horizontal report placement through the axis mapper**

In `layout_atomic_inline_items`, create the mapping after the vertical-rl guard and before line construction:

```rust
let axis_mapping = match input.writing_mode {
    WritingMode::HorizontalTb => InlineAxisMapping::new(WritingMode::HorizontalTb, input.direction),
    WritingMode::VerticalLr => InlineAxisMapping::new(WritingMode::HorizontalTb, Direction::Ltr),
    WritingMode::VerticalRl => unreachable!("vertical-rl layout is handled before line construction"),
};
```

`VerticalLr` currently falls through the horizontal line builder. This plan adds
axis vocabulary for `VerticalLr`, but it must not change that existing observable
fallback behavior. Phase 5 or a later vertical plan owns real `VerticalLr`
inline layout behavior.

When converting each committed `InlineLine` into `AtomicInlineLayoutItem`s, replace direct `Point::new(...)` construction with logical coordinates and `axis_mapping.physical_item_origin(...)`.

Before the placement loop, compute the final report inline extent from all
committed lines:

```rust
let report_inline_extent = lines
    .iter()
    .map(|line| line.width)
    .fold(S::ZERO, S::max);
```

Use `report_inline_extent` for horizontal RTL mirroring. This preserves current
block behavior, where RTL item and line-break x positions are mirrored across
the whole atomic inline report width, not only the current line width.

For box items, replace:

```rust
location: Point::new(x, y + line.baseline - item.baseline()),
```

with:

```rust
location: axis_mapping.physical_item_origin(
    LogicalInlinePointOf::new(x, y + line.baseline - item.baseline()),
    LogicalInlineSizeOf::new(item.size.width, item.size.height),
    LogicalInlineSizeOf::new(report_inline_extent, line_height),
    line_height,
),
```

For forced line breaks, replace:

```rust
location: Point::new(x, line_baseline),
```

with:

```rust
location: axis_mapping.physical_item_origin(
    LogicalInlinePointOf::new(x, line_baseline),
    LogicalInlineSizeOf::new(S::ZERO, S::ZERO),
    LogicalInlineSizeOf::new(report_inline_extent, line_height),
    line_height,
),
```

For `HorizontalTb`, `container_block_extent` is unused; passing `line_height` keeps the call local and explicit. This task must not route vertical forced breaks through the horizontal line builder.

- [ ] **Step 4: Add direct inline RTL report test**

Add this test in `src/inline_tests.rs` near `atomic_inline_forced_line_break_starts_next_line`:

```rust
#[test]
fn atomic_inline_horizontal_rtl_maps_item_origins_in_report() {
    let metrics = InlineMetrics::from_line_height_and_baseline(10.0, 10.0).unwrap();
    let report = layout_atomic_inline_items(AtomicInlineInput {
        available_width: Available::MAX_CONTENT,
        writing_mode: WritingMode::HorizontalTb,
        direction: Direction::Rtl,
        items: vec![
            AtomicInlineItem::new(0, Size::new(20.0, 10.0), Edges::ZERO, Some(10.0)),
            forced_line_break(1, metrics),
            AtomicInlineItem::new(2, Size::new(30.0, 10.0), Edges::ZERO, Some(10.0)),
        ],
    });

    assert_eq!(report.size, Size::new(30.0, 20.0));
    assert_eq!(report.items[0].location, Point::new(10.0, 0.0));
    assert_eq!(report.items[1].location, Point::new(10.0, 10.0));
    assert_eq!(report.items[2].location, Point::new(0.0, 10.0));
}
```

This direct inline test proves the report now owns horizontal direction mapping. The block-level RTL tests remain the integration proof that text alignment and run offset still compose correctly.

- [ ] **Step 5: Remove external horizontal RTL item mirroring from block output**

In `src/block.rs`, delete `inline_item_x` and replace its call sites inside `layout_atomic_inline_run`.

For box output, replace:

```rust
let item_x = inline_item_x(
    item,
    report.size.width,
    constants.direction,
    constants.writing_mode,
);
let location = Point::new(
    run_offset + item_x + inset_offset.x,
    cursor_y + item.location.y + inset_offset.y - constants.content_box_inset.top,
);
```

with:

```rust
let location = Point::new(
    run_offset + item.location.x + inset_offset.x,
    cursor_y + item.location.y + inset_offset.y - constants.content_box_inset.top,
);
```

For `tree.set_unrounded` box output, replace `item_x` with `item.location.x`:

```rust
location: Point::new(
    constants.content_box_inset.left + run_offset + item.location.x + inset_offset.x,
    cursor_y + item.location.y + inset_offset.y,
),
```

For line-break output, replace:

```rust
let item_x = inline_item_x(
    item,
    report.size.width,
    constants.direction,
    constants.writing_mode,
);
```

and use:

```rust
location: Point::new(
    constants.content_box_inset.left + run_offset + item.location.x,
    cursor_y + item.location.y,
),
```

Do not change `inline_run_offset`; text alignment remains a block integration concern in this plan.

- [ ] **Step 6: Run horizontal behavior checks**

Run:

```sh
cargo test -p surgeist-layout atomic_inline_horizontal_rtl_maps_item_origins_in_report -- --nocapture
cargo test -p surgeist-layout block_rtl_atomic_inline_run_places_items_from_right_edge -- --nocapture
cargo test -p surgeist-layout block_rtl_atomic_inline_run_mirrors_line_break_output_x -- --nocapture
cargo test -p surgeist-layout line_break_clear_ -- --nocapture
```

Expected: all pass. The block tests should have the same expected positions they had before this plan.

## Task 3: Make Vertical Box-Only Mapping Explicit Without Enabling Breaks

**Files:**
- Modify: `src/inline.rs`
- Modify: `src/inline_tests.rs`

- [ ] **Step 1: Keep vertical forced breaks rejected for now**

Keep this assertion in `layout_vertical_rl_atomic_inline_items`:

```rust
debug_assert!(
    input
        .items
        .iter()
        .all(|item| matches!(item, AtomicInlineItem::Box(_))),
    "forced atomic inline breaks are unsupported in vertical-rl layout"
);
```

Keep the `unreachable!` for `AtomicInlineItem::ForcedLineBreak(_)` in the conversion from `input.items`.

Do not add a vertical forced-break success test in this plan.

- [ ] **Step 2: Route vertical-rl box origins through the axis mapper**

Inside `layout_vertical_rl_atomic_inline_items`, add:

```rust
let axis_mapping = InlineAxisMapping::new(WritingMode::VerticalRl, input.direction);
```

after computing `line_width` and `container_width`. Replace the current one-pass
`for item in items` placement loop with a two-pass layout of positioned items so
the logical line inline extent is known before physical mapping:

```rust
let mut logical_inline_extent = S::ZERO;
let positioned_items = items
    .into_iter()
    .map(|item| {
        logical_inline_extent = logical_inline_extent + item.margin.top;
        let logical_inline_start = logical_inline_extent;
        logical_inline_extent = logical_inline_extent + item.size.height + item.margin.bottom;
        (item, logical_inline_start)
    })
    .collect::<Vec<_>>();

let line_size = LogicalInlineSizeOf::new(logical_inline_extent, line_width);
```

Replace direct `Point::new(item_x, y)` construction with:

```rust
let logical_block_start = if item.size.height == S::ZERO {
    item.margin.right - item.size.width / S::from_f64(2.0)
} else {
    item.margin.right
};

location: axis_mapping.physical_item_origin(
    LogicalInlinePointOf::new(logical_inline_start, logical_block_start),
    LogicalInlineSizeOf::new(item.size.height, item.size.width),
    line_size,
    container_width,
),
```

The existing vertical-rl baseline and content-size conventions must stay unchanged:

- `line_width` stays the max physical margin-box width;
- `container_width` stays `definite.max(line_width)` or `line_width`;
- physical y positions stay the old accumulated y positions for `Direction::Ltr`;
- physical x positions stay the old right-edge positions for `VerticalRl`;
- zero-height vertical items keep the existing centering behavior.
- `content_size` remains `Size::new(container_width, logical_inline_extent)`.

- [ ] **Step 3: Add intentional vertical-rl RTL box-only coverage**

Add this test in `src/inline_tests.rs` near
`atomic_inline_vertical_rl_places_line_against_right_edge`:

```rust
#[test]
fn atomic_inline_vertical_rl_rtl_maps_inline_progression_bottom_to_top() {
    let report = layout_atomic_inline_items(AtomicInlineInput {
        available_width: Available::definite(70.0),
        writing_mode: WritingMode::VerticalRl,
        direction: Direction::Rtl,
        items: vec![
            AtomicInlineItem::new(0, Size::new(20.0, 20.0), Edges::ZERO, Some(20.0)),
            AtomicInlineItem::new(1, Size::new(20.0, 20.0), Edges::ZERO, Some(20.0)),
        ],
    });

    assert_eq!(report.size, Size::new(70.0, 40.0));
    assert_eq!(report.items[0].location, Point::new(50.0, 20.0));
    assert_eq!(report.items[1].location, Point::new(50.0, 0.0));
}
```

This is still box-only vertical layout. It does not enable vertical forced
breaks, and it keeps `Direction` limited to inline-axis progression.

- [ ] **Step 4: Add vertical-lr axis coverage but no vertical-lr layout routing**

The axis-mapping tests in Task 1 already cover `VerticalLr`. Do not route `layout_atomic_inline_items` to a new `VerticalLr` layout path in this plan. Current observable `VerticalLr` behavior outside the helper remains unchanged.

- [ ] **Step 5: Run vertical behavior checks**

Run:

```sh
cargo test -p surgeist-layout inline_axis_mapping_maps_vertical -- --nocapture
cargo test -p surgeist-layout atomic_inline_vertical_rl_places_line_against_right_edge -- --nocapture
cargo test -p surgeist-layout atomic_inline_vertical_rl_rtl_maps_inline_progression_bottom_to_top -- --nocapture
cargo test -p surgeist-layout atomic_inline_vertical_lr_forced_break_panics_until_modeled -- --nocapture
cargo test -p surgeist-layout vertical_rl_block_places_atomic_inline_run_at_inline_start_edge -- --nocapture
cargo test -p surgeist-layout vertical_line_break_panics_until_modeled -- --nocapture
```

Expected:

- axis-mapping tests pass for `VerticalRl` and `VerticalLr`;
- existing vertical-rl box-only layout tests pass unchanged;
- vertical-rl RTL box-only layout has explicit coverage;
- vertical-lr forced breaks remain explicitly unsupported;
- vertical line-break panic test still passes.

## Task 4: Verify Boundaries And Full Crate

**Files:**
- Modify: `src/inline.rs`
- Modify: `src/inline_tests.rs`
- Modify: `src/block.rs`
- Modify: `src/block_tests.rs`

- [ ] **Step 1: Search for scope drift**

Run:

```sh
rg -n "source-tag|HTML|html|font|line-height|surgeist_style|surgeist-style|surgeist_retained|surgeist-retained|vertical forced|vertical break support|unsupported in vertical-rl" src/inline.rs src/inline_tests.rs src/block.rs src/block_tests.rs
```

Expected:

- no style or retained dependency appears;
- no HTML/source-tag parsing appears;
- no font or line-height derivation appears;
- hits for unsupported vertical forced breaks are limited to existing explicit unsupported assertions/tests.

- [ ] **Step 2: Run focused inline and block checks**

Run:

```sh
cargo test -p surgeist-layout inline_axis_mapping_ -- --nocapture
cargo test -p surgeist-layout atomic_inline_ -- --nocapture
cargo test -p surgeist-layout line_break -- --nocapture
cargo test -p surgeist-layout vertical_rl_block_places_atomic_inline_run_at_inline_start_edge -- --nocapture
```

Expected: all pass.

- [ ] **Step 3: Run final checks**

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
- git status shows only intentional changes before commit.

## Commit Point

After the worker completes all tasks and the scoped reviewer comes back clean, the coordinator should commit:

```sh
git add src/inline.rs src/inline_tests.rs src/block.rs src/block_tests.rs
git commit -m "Extract logical inline axis mapping"
```

Do not commit before the review cycle is clean.

## Review Checklist

The clean-context reviewer should verify:

- implementation matches Phase 4 of the sequencing document;
- horizontal LTR and RTL output is unchanged at the block boundary;
- line-break clear behavior from Phase 3 is unchanged;
- `Direction` is modeled as inline-axis direction only;
- `WritingMode` owns physical block-axis mapping;
- all `WritingMode` variants are covered by axis-mapping tests;
- vertical forced breaks remain explicitly unsupported;
- no HTML/CSS parsing, fixture generation, public API exposure, text/font metric derivation, or style/retained dependency was added;
- final checks listed above were run and passed.

## Follow-Up Plans

After this plan is implemented and reviewed cleanly, the next derived plan should be Phase 5 from the sequencing document: support vertical forced breaks in inline and block integration using the logical-axis model.
