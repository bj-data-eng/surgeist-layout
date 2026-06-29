# Surgeist Layout BR Line Break Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Support HTML `<br>` in layout browser parity as a typed inline line-break primitive with basic style context, without treating it as an ordinary block or measurable box.

**Architecture:** Add a layout-owned semantic line-break role to `NodeInput`, then teach the atomic inline run path to split horizontal lines at that role while assigning a zero-size output to the `<br>` node. Browser parity should stop using the stale generic `<br>` unsupported bucket, generate only the supported horizontal block-inline `<br>` fixtures, preserve style attributes, and lower `source-tag="br"` to the new line-break role after normal `surgeist-style` declaration resolution. This does not implement vertical-writing `<br>` behavior, flex/grid-parent `<br>` behavior, full mixed text/element inline layout, generated content, or `<br clear>` float clearance beyond preserving the `clear` field for later work.

**Tech Stack:** Rust 2024, `surgeist-layout`, source-side tests in `src/*_tests.rs`, browser parity support in `tests/layout/browser_parity/support.rs`, generator helper in `tests/layout/browser_parity/scripts/gentest/test_helper.js`, generator tests in `tests/bin/surgeist-layout-generate/generator.rs`, generated XML under `tests/layout/browser_parity/xml`.

---

## Product Boundary

`<br>` support belongs in layout as a semantic inline primitive, not as a special DOM tag hard-coded into algorithms. The layout crate should expose a typed line-break input that upstream HTML/style adapters can set when they see an HTML line break element. Browser parity may use `source-tag="br"` as fixture metadata to exercise that primitive, but production layout behavior must not depend on parsing the string `"br"` inside core layout algorithms.

The first implementation supports:

- A line-break role that forces a new horizontal atomic inline line when a parseable `display: block` parent collects it into an atomic inline run.
- Zero-size output for the line-break node so fixture geometry can compare it.
- `display: none` suppressing the line break through the existing hidden-layout path.
- Normal style resolution for inherited/applicable fields already captured by fixture attributes, including `direction`, `writing-mode`, `vertical-align`, and `clear`, even if some fields are only carried for future behavior.
- Non-`none` display values on a line-break node cannot make it participate as a normal block, flex, grid, or leaf box. Core layout should route `InlineRole::LineBreak` through inline-run handling, and browser parity should normalize the line-break node's parent-flow display to `InlineBlock` after style lowering unless style resolved it to `Display::None`.

The first implementation does not support:

- Full mixed text and element inline layout.
- Generated content around `<br>`.
- Treating `<br>` as a normal block, flex, grid, or leaf box.
- Vertical-writing `<br>` line progression. The generator must keep those fixtures explicitly unsupported with a distinct vertical-writing reason until focused vertical inline-line behavior is designed.
- Horizontal `<br>` outside a parseable `display: block` parent that can collect
  it into an atomic inline run. Keep those fixtures explicitly unsupported with
  a distinct outside-block-inline-run reason until `flow-root`, `list-item`,
  flex/grid, or other parent semantics are designed.
- Empty line-height-only behavior for leading, trailing, or consecutive `<br>` beyond explicitly tested atomic-inline behavior. If those cases appear in the corpus, classify them separately instead of guessing.
- Browser-specific `<br clear>` float clearance. Preserve `clear` on the node, but do not claim float-clear behavior until there are focused tests.

## Current Evidence

The current generated corpus report has:

```text
240 unsupported variants / 60 unique HTML fixtures: Unsupported <br> line-break semantics
100 unsupported variants / 25 unique HTML fixtures: Unsupported mixed text/element content
16 unsupported variants / 4 unique HTML fixtures: Unsupported missing #test-root fixture root
```

All `<br>` unsupported fixtures are in the `subgrid` suite and mostly have inline-block spans separated by `<br>` inside baseline fixtures. Some are vertical-writing fixtures; this plan keeps those in a separate explicit unsupported bucket instead of pretending the horizontal atomic-inline implementation covers them.

## Coordinator Execution Workflow

Follow `AGENTS.md` for every implementation task. The coordinator assigns one
worker to the current task scope, then assigns a separate scoped reviewer before
the task commit. A task is committable only after:

- the worker reports changed files, commands run, results, and git status;
- the scoped reviewer is clean or all findings have been reconciled;
- the coordinator reruns the task's focused check after any review fixes;
- the diff is reviewed with `git diff --stat` and the relevant detailed diff.

The commit steps below assume that this worker/reviewer gate is already clean.
Do not skip scoped review merely because a final holistic review is also
required.

## Task 1: Add A Typed Line-Break Role To Layout Input

**Files:**

- Modify: `src/node_input.rs`
- Modify: `src/lib.rs`
- Modify: `src/contract_tests.rs`
- Modify: `api/public-api.txt`

- [ ] **Step 1: Add the public semantic role**

In `src/node_input.rs`, add this enum near `Display`:

```rust
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum InlineRole {
    #[default]
    Box,
    LineBreak,
}

impl InlineRole {
    #[must_use]
    pub const fn is_line_break(self) -> bool {
        matches!(self, Self::LineBreak)
    }
}
```

- [ ] **Step 2: Add the role to `NodeInputOf`**

Add this public field immediately after `display`:

```rust
pub inline_role: InlineRole,
```

Update both default constructors:

```rust
inline_role: InlineRole::Box,
```

- [ ] **Step 3: Reexport the role**

In `src/lib.rs`, add `InlineRole` to the `pub use node_input::{ ... }` list.

- [ ] **Step 4: Add contract tests**

In `src/contract_tests.rs`, add:

```rust
#[test]
fn node_input_defaults_to_box_inline_role() {
    assert_eq!(NodeInput::default().inline_role, InlineRole::Box);
}

#[test]
fn inline_role_marks_line_break_semantics_without_changing_display() {
    let input = NodeInput {
        display: Display::Block,
        inline_role: InlineRole::LineBreak,
        ..NodeInput::default()
    };

    assert!(input.inline_role.is_line_break());
    assert_eq!(input.display, Display::Block);
}
```

- [ ] **Step 5: Run focused contract tests**

Run:

```sh
cargo test -p surgeist-layout inline_role -- --nocapture
```

Expected: both tests pass.

- [ ] **Step 6: Refresh API artifact**

Run:

```sh
cargo run --manifest-path api/generator/Cargo.toml
```

Expected: `api/public-api.txt` records `InlineRole` and the new `NodeInputOf::inline_role` field.

- [ ] **Step 7: Commit**

```sh
git add src/node_input.rs src/lib.rs src/contract_tests.rs api/public-api.txt
git commit -m "Add line break role to layout input"
```

## Task 2: Teach Atomic Inline Layout About Forced Break Items

**Files:**

- Modify: `src/inline.rs`
- Modify: `src/inline_tests.rs`

- [ ] **Step 1: Replace the single item payload with an enum**

In `src/inline.rs`, rename the current struct to `AtomicInlineBoxItem` and add an enum named `AtomicInlineItem`:

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum AtomicInlineItem<S: LayoutScalar = DefaultScalar> {
    Box(AtomicInlineBoxItem<S>),
    ForcedLineBreak { order: u32 },
}
```

Keep the existing box fields on `AtomicInlineBoxItem<S>`:

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct AtomicInlineBoxItem<S: LayoutScalar = DefaultScalar> {
    pub order: u32,
    pub size: Size<S>,
    pub content_size: Size<S>,
    pub margin: Edges<S>,
    pub padding: Edges<S>,
    pub border: Edges<S>,
    pub scrollbar_size: Size<S>,
    pub first_baseline: Option<S>,
}
```

Move `advance`, `baseline`, `line_baseline`, and `line_descent` onto `AtomicInlineBoxItem<S>`.

- [ ] **Step 2: Preserve the existing test constructor**

For test ergonomics, keep `AtomicInlineItem::new(...)` returning a box item:

```rust
impl<S: LayoutScalar> AtomicInlineItem<S> {
    #[cfg(test)]
    pub(super) const fn new(
        order: u32,
        size: Size<S>,
        margin: Edges<S>,
        first_baseline: Option<S>,
    ) -> Self {
        Self::Box(AtomicInlineBoxItem {
            order,
            size,
            content_size: size,
            margin,
            padding: Edges::ZERO,
            border: Edges::ZERO,
            scrollbar_size: Size::ZERO,
            first_baseline,
        })
    }

    #[must_use]
    pub(super) const fn forced_line_break(order: u32) -> Self {
        Self::ForcedLineBreak { order }
    }
}
```

- [ ] **Step 3: Represent line-break output in reports**

Add an item kind to the layout report:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum AtomicInlineLayoutItemKind {
    Box,
    ForcedLineBreak,
}
```

Add this field to `AtomicInlineLayoutItem<S>`:

```rust
pub kind: AtomicInlineLayoutItemKind,
```

When reporting ordinary boxes, set `kind: AtomicInlineLayoutItemKind::Box`.

- [ ] **Step 4: Split horizontal lines on forced breaks**

In `layout_atomic_inline_items`, change the loop over `input.items` to match each item:

```rust
for item in input.items {
    match item {
        AtomicInlineItem::Box(item) => {
            let advance = item.advance();
            if let Some(available_width) = available_width
                && !line.is_empty()
                && line.width + advance > available_width
            {
                lines.push(line);
                line = InlineLine::<S>::default();
            }
            line.push(item);
        }
        AtomicInlineItem::ForcedLineBreak { order } => {
            let x = line.width;
            line.breaks.push(PendingLineBreak { order, x });
            lines.push(line);
            line = InlineLine::<S>::default();
        }
    }
}
```

Implement this with concrete structs that compile:

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
struct PendingLineBreak<S: LayoutScalar = DefaultScalar> {
    order: u32,
    x: S,
}
```

and add `breaks: Vec<PendingLineBreak<S>>` to `InlineLine<S>`.

Update `InlineLine::is_empty` so a line with only a break is still reportable:

```rust
fn is_empty(&self) -> bool {
    self.items.is_empty() && self.breaks.is_empty()
}
```

When converting each line to report items, emit each break after the boxes in that line:

```rust
items.push(AtomicInlineLayoutItem {
    kind: AtomicInlineLayoutItemKind::ForcedLineBreak,
    order: pending_break.order,
    location: Point::new(pending_break.x, y + line.baseline),
    size: Size::ZERO,
    content_size: Size::ZERO,
    margin: Edges::ZERO,
    padding: Edges::ZERO,
    border: Edges::ZERO,
    scrollbar_size: Size::ZERO,
});
```

This places the zero-size break at the end of the current line. Do not add an empty trailing line after a final break unless a later task adds line-height metrics.

- [ ] **Step 5: Reject vertical forced breaks until vertical line progression is designed**

At the top of `layout_vertical_rl_atomic_inline_items`, add a debug assertion that no forced breaks reach the vertical path yet:

```rust
debug_assert!(
    input
        .items
        .iter()
        .all(|item| !matches!(item, AtomicInlineItem::ForcedLineBreak { .. })),
    "vertical forced line breaks are not modeled yet"
);
```

Do not add a best-effort vertical placement. The generator in Task 5 keeps vertical-writing `<br>` fixtures explicitly unsupported, so reaching this assertion means a caller created a `LineBreak` node in a vertical writing mode without a supported model.

Because `AtomicInlineItem` becomes an enum, update the existing vertical layout
loop and `line_width` calculation to operate on `AtomicInlineItem::Box(item)`
values after the assertion. Use an explicit match so the function still
compiles after the enum conversion:

```rust
let boxes = input
    .items
    .iter()
    .map(|item| match item {
        AtomicInlineItem::Box(item) => item,
        AtomicInlineItem::ForcedLineBreak { .. } => {
            unreachable!("vertical forced line breaks are not modeled yet")
        }
    })
    .collect::<Vec<_>>();
```

Then use `boxes` for the existing line-width and placement loops. Do not
silently drop forced breaks in vertical writing mode.

- [ ] **Step 6: Update intrinsic widths**

Update `atomic_inline_min_content_width` and `atomic_inline_max_content_width` so forced breaks split max-content sums into line segments:

```rust
pub(super) fn atomic_inline_min_content_width<S: LayoutScalar>(items: &[AtomicInlineItem<S>]) -> S {
    items
        .iter()
        .filter_map(|item| match item {
            AtomicInlineItem::Box(item) => Some(item.advance()),
            AtomicInlineItem::ForcedLineBreak { .. } => None,
        })
        .fold(S::ZERO, S::max)
}
```

For max content, track the maximum segment sum between forced breaks:

```rust
pub(super) fn atomic_inline_max_content_width<S: LayoutScalar>(items: &[AtomicInlineItem<S>]) -> S {
    let mut max_width = S::ZERO;
    let mut current_width = S::ZERO;
    for item in items {
        match item {
            AtomicInlineItem::Box(item) => {
                current_width = current_width + item.advance();
            }
            AtomicInlineItem::ForcedLineBreak { .. } => {
                max_width = max_width.max(current_width);
                current_width = S::ZERO;
            }
        }
    }
    max_width.max(current_width)
}
```

- [ ] **Step 7: Add focused inline tests**

In `src/inline_tests.rs`, add:

```rust
#[test]
fn atomic_inline_forced_line_break_starts_next_line() {
    let report = layout_atomic_inline_items(AtomicInlineInput {
        available_width: Available::MAX_CONTENT,
        writing_mode: WritingMode::HorizontalTb,
        items: vec![
            AtomicInlineItem::new(0, Size::new(20.0, 10.0), Edges::ZERO, Some(10.0)),
            AtomicInlineItem::forced_line_break(1),
            AtomicInlineItem::new(2, Size::new(30.0, 10.0), Edges::ZERO, Some(10.0)),
        ],
    });

    assert_eq!(report.size, Size::new(30.0, 20.0));
    assert_eq!(report.first_baseline, Some(10.0));
    assert_eq!(report.last_baseline, Some(20.0));
    assert_eq!(report.items[0].location, Point::new(0.0, 0.0));
    assert_eq!(report.items[1].kind, AtomicInlineLayoutItemKind::ForcedLineBreak);
    assert_eq!(report.items[1].location, Point::new(20.0, 10.0));
    assert_eq!(report.items[2].location, Point::new(0.0, 10.0));
}

#[test]
fn atomic_inline_intrinsic_widths_split_at_forced_line_breaks() {
    let items = vec![
        AtomicInlineItem::new(0, Size::new(20.0, 10.0), Edges::ZERO, Some(10.0)),
        AtomicInlineItem::new(1, Size::new(30.0, 10.0), Edges::ZERO, Some(10.0)),
        AtomicInlineItem::forced_line_break(2),
        AtomicInlineItem::new(3, Size::new(40.0, 10.0), Edges::ZERO, Some(10.0)),
    ];

    assert_eq!(atomic_inline_min_content_width(&items), 40.0);
    assert_eq!(atomic_inline_max_content_width(&items), 50.0);
}
```

- [ ] **Step 8: Run focused inline tests**

Run:

```sh
cargo test -p surgeist-layout forced_line_break -- --nocapture
```

Expected: both tests pass.

- [ ] **Step 9: Commit**

```sh
git add src/inline.rs src/inline_tests.rs
git commit -m "Support forced breaks in atomic inline layout"
```

## Task 3: Wire Line-Break Nodes Through Block Atomic Inline Runs

**Files:**

- Modify: `src/block.rs`
- Modify: `src/block_tests.rs`
- Modify: `src/compute.rs`
- Modify: `src/compute_tests.rs`
- Modify: `src/lib.rs`
- Modify: `src/test_support/layout_tree.rs`
- Modify: `api/public-api.txt`

- [ ] **Step 1: Add a local helper for inline-run participation**

In `src/block.rs`, add this helper near `layout_in_flow_children`:

```rust
fn participates_in_atomic_inline_run<S: LayoutScalar>(style: &NodeInputOf<S>) -> bool {
    style.display.is_inline_level() || style.inline_role == InlineRole::LineBreak
}
```

Add `InlineRole` to the existing `use super::{ ... }` import list in `src/block.rs`.

Then update the two inline-run checks in `layout_in_flow_children`:

```rust
if participates_in_atomic_inline_run(&child_style) && child_style.float.is_none() {
```

and:

```rust
if !participates_in_atomic_inline_run(run_style) {
    break;
}
```

Keep the existing `Display::None`, absolute-position, and float checks before this helper. Hidden line breaks must still be skipped before inline-run handling.

- [ ] **Step 2: Add a test for a line-break child between atomic inline boxes**

In `src/block_tests.rs`, add a test shaped like this:

```rust
#[test]
fn block_atomic_inline_run_honors_line_break_child() {
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
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                inline_role: InlineRole::LineBreak,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(30.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::MAX_CONTENT));
    round_layout(&mut tree, 0);

    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(30.0, 20.0));
    assert_eq!(tree.final_layout(1).unwrap().location, Point::new(0.0, 0.0));
    assert_eq!(tree.final_layout(2).unwrap().location, Point::new(20.0, 10.0));
    assert_eq!(tree.final_layout(2).unwrap().size, Size::ZERO);
    assert_eq!(tree.final_layout(3).unwrap().location, Point::new(0.0, 10.0));
}
```

Adjust only imports or helper names needed to compile against current `block_tests.rs` patterns.

- [ ] **Step 3: Add a test proving non-inline display cannot turn a line break into a block**

In `src/block_tests.rs`, add:

```rust
#[test]
fn line_break_role_participates_as_inline_even_with_block_display() {
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
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                inline_role: InlineRole::LineBreak,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(30.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::MAX_CONTENT));
    round_layout(&mut tree, 0);

    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(30.0, 20.0));
    assert_eq!(tree.final_layout(2).unwrap().size, Size::ZERO);
    assert_eq!(tree.final_layout(3).unwrap().location, Point::new(0.0, 10.0));
}
```

- [ ] **Step 4: Convert line-break children to forced inline items**

In `layout_atomic_inline_run`, before computing a child as a normal atomic box, check:

```rust
if child_style.inline_role == InlineRole::LineBreak {
    let item = AtomicInlineItem::forced_line_break(order_start + offset as u32);
    run_children.push(AtomicInlineRunChild::LineBreak {
        child,
        child_style,
        order: order_start + offset as u32,
    });
    items.push(item);
    continue;
}
```

Use a real local enum rather than extending the current tuple with unused fields:

```rust
enum AtomicInlineRunChild<Node, S: LayoutScalar> {
    Box {
        child: Node,
        style: NodeInputOf<S>,
        output: ComputeOutputOf<S>,
    },
    LineBreak {
        child: Node,
        style: NodeInputOf<S>,
        order: u32,
    },
}
```

For `LineBreak`, do not call `compute_child` as a normal box. If `set_layout` is true, write the zero-size layout from the matching report item.

- [ ] **Step 5: Match report items by order**

Because forced line breaks produce report items too, stop relying on a positional zip that assumes every run child is a box. Build a small lookup:

```rust
let report_items = report
    .items
    .iter()
    .map(|item| (item.order, *item))
    .collect::<BTreeMap<_, _>>();
```

Import `std::collections::BTreeMap` at the top of `src/block.rs`. Then, for each `AtomicInlineRunChild`, fetch the report item by order and write either normal box output or line-break output.

- [ ] **Step 6: Preserve `display: none` suppression**

Do not special-case hidden line breaks. Existing code should still skip any child with `display == Display::None` before checking `inline_role`.

Add this test to `src/block_tests.rs`:

```rust
#[test]
fn hidden_line_break_does_not_split_atomic_inline_run() {
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
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::None,
                inline_role: InlineRole::LineBreak,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(30.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::MAX_CONTENT));
    round_layout(&mut tree, 0);

    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(50.0, 10.0));
    assert_eq!(tree.final_layout(3).unwrap().location, Point::new(20.0, 0.0));
}
```

- [ ] **Step 7: Add a zero-size fallback compute helper**

In `src/compute.rs`, add a helper that callers can use when a line-break node is
computed outside the block atomic-inline collector:

```rust
pub fn compute_line_break<Tree>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    _input: ComputeInputOf<<Tree as Traverse>::Scalar>,
) -> ComputeOutputOf<<Tree as Traverse>::Scalar>
where
    Tree: Compute,
{
    tree.set_unrounded(node, NodeOutputOf::with_order(0));
    ComputeOutputOf::from_outer_size(Size::ZERO)
}
```

Reexport it from `src/lib.rs` beside `compute_hidden`, `compute_leaf`, and
`compute_root`.

This helper is a guardrail, not a substitute for block inline-run handling. It
prevents a semantic line-break node from being accidentally computed as a
normal block, flex, grid, or leaf formatting context when a dispatcher sees it
outside an inline-run collection.

Run the API generator after adding the public helper:

```sh
cargo run --manifest-path api/generator/Cargo.toml
```

Expected: `api/public-api.txt` records `compute_line_break`.

- [ ] **Step 8: Make crate test dispatchers prefer line-break role before display**

In `src/test_support/layout_tree.rs`, update both `Compute` impls so
`compute_child` checks the role before matching `display.inner_display()`:

```rust
let style = self.node_input(node);
if style.display != Display::None && style.inline_role == InlineRole::LineBreak {
    return compute_line_break(self, node, input);
}
```

Keep the existing `Display::None` behavior hidden. Do not make hidden line-break
nodes produce a visible zero-size layout.

Add or adjust any local test-only compute dispatchers that are directly used by
the new tests if they would otherwise compute a line-break node as block/flex/grid.
Do not sweep through unrelated ad hoc test dispatchers unless the compiler or
focused tests require it.

- [ ] **Step 9: Add fallback tests for non-block parent escape**

In `src/compute_tests.rs`, add a focused test using a tiny `Compute` fixture or
the existing oracle tree support that proves a node with
`inline_role: InlineRole::LineBreak` and `display: Display::Flex` returns
`ComputeOutput::from_outer_size(Size::ZERO)` through the dispatcher instead of
entering flex layout. Add a second assertion or test for `Display::Grid` if the
available fixture can exercise it without needing a full grid setup.

In `src/block_tests.rs`, keep the block-parent `Display::Block` line-break test
from Step 3. Together, these tests document that line breaks participate in
block atomic inline runs when collected there and otherwise fall back to a
zero-size line-break output rather than escaping into display-driven layout.

- [ ] **Step 10: Run focused block and fallback tests**

Run:

```sh
cargo test -p surgeist-layout line_break -- --nocapture
cargo test -p surgeist-layout compute_line_break -- --nocapture
```

Expected: the line-break block and fallback tests pass.

- [ ] **Step 11: Request scoped review**

Ask a separate reviewer to inspect only Task 3 changes. The reviewer must check
that:

- line-break nodes are collected by block atomic inline runs;
- `display: none` still suppresses line-break nodes before inline-run handling;
- non-`none` line-break nodes cannot dispatch as normal block/flex/grid/leaf
  layout when encountered outside block inline-run collection.

Reconcile reviewer findings before committing.

- [ ] **Step 12: Commit**

```sh
git add src/block.rs src/block_tests.rs src/compute.rs src/compute_tests.rs src/lib.rs src/test_support/layout_tree.rs api/public-api.txt
git commit -m "Wire line break nodes through block inline runs"
```

## Task 4: Lower Browser Parity `<br>` Metadata To The Line-Break Role

**Files:**

- Modify: `tests/layout/browser_parity/support.rs`

- [ ] **Step 1: Replace the rejection test with a positive lowering test**

Replace `source_tag_br_is_rejected_until_line_break_semantics_are_modeled` with these positive tests:

```rust
#[test]
fn source_tag_br_lowers_to_line_break_role() {
    let input = to_node_input(
        &StyleAttrs {
            attrs: BTreeMap::from([("source-tag".to_string(), "br".to_string())]),
        },
        &mut s::adapters::layout::LayoutLoweringSession::new(),
    )
    .expect("source-tag br should lower to a line break role");

    assert_eq!(input.inline_role, layout::InlineRole::LineBreak);
    assert_eq!(input.display, layout::Display::InlineBlock);
}

#[test]
fn source_tag_br_display_none_suppresses_line_break_role_in_flow() {
    let input = to_node_input(
        &StyleAttrs {
            attrs: BTreeMap::from([
                ("source-tag".to_string(), "br".to_string()),
                ("display".to_string(), "none".to_string()),
            ]),
        },
        &mut s::adapters::layout::LayoutLoweringSession::new(),
    )
    .expect("display none br should lower");

    assert_eq!(input.inline_role, layout::InlineRole::LineBreak);
    assert_eq!(input.display, layout::Display::None);
}

#[test]
fn source_tag_br_non_none_display_keeps_line_break_inline_participation() {
    let input = to_node_input(
        &StyleAttrs {
            attrs: BTreeMap::from([
                ("source-tag".to_string(), "br".to_string()),
                ("display".to_string(), "block".to_string()),
            ]),
        },
        &mut s::adapters::layout::LayoutLoweringSession::new(),
    )
    .expect("styled br should lower");

    assert_eq!(input.inline_role, layout::InlineRole::LineBreak);
    assert_eq!(input.display, layout::Display::InlineBlock);
}
```

- [ ] **Step 2: Stop rejecting `source-tag="br"` in declarations**

Remove this early return from `to_declarations`:

```rust
if attrs.get("source-tag") == Some("br") {
    return Err(Error::new(
        "unsupported source-tag `br`; line-break semantics are not represented",
    ));
}
```

Update the source-tag default display match so `br` gets an inline-level default:

```rust
None => match attrs.get("source-tag") {
    Some("div") => Some(layout::Display::Block),
    Some("br") => Some(layout::Display::InlineBlock),
    _ => None,
},
```

- [ ] **Step 3: Set the line-break role after style lowering**

In `to_node_input`, after the existing `surgeist-style::adapters::layout` lowering returns a `layout::NodeInput`, set:

```rust
if attrs.get("source-tag") == Some("br") {
    input.inline_role = layout::InlineRole::LineBreak;
    if input.display != layout::Display::None {
        input.display = layout::Display::InlineBlock;
    }
}
```

Keep all other style-derived fields from the style adapter result. The display normalization is not an extra style lowering layer; it is fixture metadata selecting layout parent-flow participation for the semantic line-break role after the normal style adapter has run.

- [ ] **Step 4: Keep checked XML quarantine green until regeneration**

Leave `checked_fixture_enumerator_quarantines_unsupported_br_xml` in place for this task. It should still pass because checked XML has not been regenerated yet. Task 6 replaces this test after generated XML contains `source-tag="br"` fixtures.

- [ ] **Step 5: Dispatch parity line-break nodes before leaf/display layout**

In `TestTree::compute_uncached` in `tests/layout/browser_parity/support.rs`,
after the existing hidden-layout check and before `can_use_leaf_measurement`,
add:

```rust
if node_input.inline_role == layout::InlineRole::LineBreak {
    return layout::compute_line_break(self, node, input);
}
```

Keep `Display::None` and `PerformHiddenLayout` ahead of this role check so
hidden line breaks stay hidden. This is required because browser parity uses
the same `support.rs` lowering path that sets `InlineRole::LineBreak`; without
the dispatcher gate, a generated `<br>` node can still fall through to leaf or
display-driven layout.

Add a focused support test that builds or loads a `source-tag="br"` node with a
non-`none` display and verifies `TestTree` computes it through the zero-size
line-break path rather than leaf measurement. If the existing test helpers make
a direct `TestTree` setup awkward, use the smallest XML fixture string accepted
by support tests.

- [ ] **Step 6: Run focused support tests**

Run:

```sh
cargo test -p surgeist-layout --test layout source_tag_br -- --nocapture
cargo test -p surgeist-layout --test layout checked_fixture_enumerator_quarantines_unsupported_br_xml -- --nocapture
cargo test -p surgeist-layout --test layout parity_line_break -- --nocapture
```

Expected: the positive source-tag tests pass, the existing checked-fixture quarantine still passes, and the parity dispatcher test passes.

- [ ] **Step 7: Commit**

```sh
git add tests/layout/browser_parity/support.rs
git commit -m "Lower browser parity br as line break"
```

## Task 5: Generate Only Supported Horizontal Block-Inline `<br>` Fixtures

**Files:**

- Modify: `tests/layout/browser_parity/scripts/gentest/test_helper.js`
- Modify: `tests/bin/surgeist-layout-generate/generator.rs`

- [ ] **Step 1: Narrow the JavaScript unsupported element branch to modeled contexts**

In `tests/layout/browser_parity/scripts/gentest/test_helper.js`, change the call in `describeElement` from:

```javascript
unsupportedReason: unsupportedElementReason(e) || unsupportedChildNodesReason(e),
```

to:

```javascript
unsupportedReason: unsupportedElementReason(e, computedStyle) || unsupportedChildNodesReason(e),
```

Then change:

```javascript
function unsupportedElementReason(e) {
  if (e.tagName === 'BR') return "Unsupported <br> line-break semantics";
  return undefined;
}
```

to:

```javascript
function unsupportedElementReason(e, computedStyle) {
  if (e.tagName === 'BR' && isVerticalWritingMode(computedStyle.writingMode)) {
    return "Unsupported vertical <br> line-break semantics";
  }
  if (e.tagName === 'BR' && !hasSupportedBrLineBreakParent(e)) {
    return "Unsupported <br> outside block inline-run semantics";
  }
  return undefined;
}
```

Add the helper near `unsupportedElementReason`:

```javascript
function hasSupportedBrLineBreakParent(e) {
  const parent = e.parentElement;
  if (!parent) return false;
  const parentDisplay = getComputedStyle(parent).display;
  return parentDisplay === "block";
}
```

Do not remove `tagName: e.tagName.toLowerCase()` from `describeElement`; Rust
support needs `source-tag="br"`. Horizontal `<br>` should be generated only
when its immediate parent has computed `display: block`, which layout can parse
and collect into an atomic inline run. Horizontal `<br>` in `flow-root`,
`list-item`, flex, grid, inline-grid, inline-flex, table, or other unsupported
parent contexts must stay unsupported with the outside-block-inline-run reason.

- [ ] **Step 2: Replace generator helper test assertions**

In `tests/bin/surgeist-layout-generate/generator.rs`, replace `bundled_helper_rejects_br_instead_of_lowering_to_measured_size` with:

```rust
#[test]
fn bundled_helper_describes_br_as_source_tag_without_measured_box_special_case() {
    assert!(TEST_HELPER_SOURCE.contains("tagName: e.tagName.toLowerCase()"));
    assert!(TEST_HELPER_SOURCE.contains("unsupportedElementReason(e, computedStyle)"));
    assert!(!TEST_HELPER_SOURCE.contains("Unsupported <br> line-break semantics"));
    assert!(TEST_HELPER_SOURCE.contains("Unsupported vertical <br> line-break semantics"));
    assert!(TEST_HELPER_SOURCE.contains("Unsupported <br> outside block inline-run semantics"));
    assert!(TEST_HELPER_SOURCE.contains("hasSupportedBrLineBreakParent(e)"));
}
```

Also add a small helper-source assertion:

```rust
#[test]
fn bundled_helper_keeps_vertical_br_explicitly_unsupported() {
    assert!(TEST_HELPER_SOURCE.contains("isVerticalWritingMode(computedStyle.writingMode)"));
    assert!(TEST_HELPER_SOURCE.contains("Unsupported vertical <br> line-break semantics"));
}
```

Also add:

```rust
#[test]
fn bundled_helper_keeps_unmodeled_br_parent_contexts_unsupported() {
    assert!(TEST_HELPER_SOURCE.contains("hasSupportedBrLineBreakParent(e)"));
    assert!(TEST_HELPER_SOURCE.contains("Unsupported <br> outside block inline-run semantics"));
    assert!(TEST_HELPER_SOURCE.contains("parentDisplay === \"block\""));
}
```

- [ ] **Step 3: Update unsupported report unit test**

In `unsupported_browser_semantics_are_reported_without_xml_generation`, remove the fake `<br>` unsupported entry from the test descriptor and assert the remaining unsupported count:

```rust
let desc = json!({
    "borderBoxLtrData": unsupported_node("Unsupported mixed text/element content"),
    "contentBoxLtrData": unsupported_node("Unsupported mixed text/element content"),
    "borderBoxRtlData": unsupported_node("Unsupported mixed text/element content"),
    "contentBoxRtlData": unsupported_node("Unsupported mixed text/element content")
});

assert_eq!(report.unsupported.len(), 4);
```

If the existing test already has four mixed-text variants after edit, keep the count at `4`.

- [ ] **Step 4: Run generator tests**

Run:

```sh
cargo test -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate bundled_helper_describes_br_as_source_tag_without_measured_box_special_case -- --nocapture
cargo test -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate bundled_helper_keeps_vertical_br_explicitly_unsupported -- --nocapture
cargo test -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate bundled_helper_keeps_unmodeled_br_parent_contexts_unsupported -- --nocapture
cargo test -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate unsupported_browser_semantics_are_reported_without_xml_generation -- --nocapture
```

Expected: all four tests pass.

- [ ] **Step 5: Commit**

```sh
git add tests/layout/browser_parity/scripts/gentest/test_helper.js tests/bin/surgeist-layout-generate/generator.rs
git commit -m "Generate browser parity br fixtures"
```

## Task 6: Regenerate Browser Parity XML And Reports For `<br>` Fixtures

**Files:**

- Modify: `tests/layout/browser_parity/xml/**`
- Modify: `tests/layout/browser_parity/xml/generation-reports/*.json`
- Modify if counts changed: `tests/layout/browser_parity/README.md`

- [ ] **Step 1: Regenerate the affected subgrid fixtures**

Run:

```sh
SURGEIST_PARITY_FILTER=subgrid_baseline cargo run -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate
```

Expected: XML files are generated for horizontal `<br>` subgrid baseline
fixtures whose immediate parent has parseable `display: block` and can be
collected as a block atomic inline run.
The subgrid generation report no longer has the generic
`Unsupported <br> line-break semantics` bucket. Vertical-writing `<br>` fixtures,
and any horizontal `<br>` fixtures outside supported `display: block` parents,
remain explicitly unsupported with their distinct reasons.

- [ ] **Step 2: Regenerate the full report**

Run:

```sh
cargo run -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate
```

Expected: `tests/layout/browser_parity/xml/generation-reports/all.json` changes from:

```text
generated: 4984
unsupported: 356
```

to a report where:

```text
generated > 4984
unsupported < 356
generic "Unsupported <br> line-break semantics" count = 0
```

Run this summary command and record the output in the worker result:

```sh
jq -r '(.unsupported // [])[] | (.reason // .kind // .error // "unknown")' \
  tests/layout/browser_parity/xml/generation-reports/all.json \
  | sort | uniq -c | sort -nr
```

Expected remaining unsupported reasons are:

```text
Unsupported vertical <br> line-break semantics
Unsupported <br> outside block inline-run semantics
Unsupported mixed text/element content
Unsupported missing #test-root fixture root
```

If any generic `Unsupported <br> line-break semantics` entries remain, stop and fix the generator/helper classification before committing.

- [ ] **Step 3: Update README measured impact if needed**

If `tests/layout/browser_parity/README.md` has measured unsupported counts that now mention `<br>`, replace them with the new counts from `all.json`. Keep the wording factual:

```markdown
After horizontal `<br>` line-break support, the full generation report no
longer has a generic `Unsupported <br> line-break semantics` bucket. Remaining
unsupported fixtures are vertical-writing `<br>` line breaks, `<br>` fixtures
outside supported `display: block` parent contexts, mixed text/element content,
and missing `#test-root` fixture roots.
```

- [ ] **Step 4: Replace the checked-fixture quarantine test with a report assertion**

Now that XML regeneration is done, replace `checked_fixture_enumerator_quarantines_unsupported_br_xml` in `tests/layout/browser_parity/support.rs` with:

```rust
#[test]
fn generation_report_no_longer_classifies_horizontal_br_as_unsupported() {
    let report = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/layout/browser_parity/xml/generation-reports/all.json");
    let raw = std::fs::read_to_string(&report)
        .unwrap_or_else(|error| panic!("{} should read: {error}", report.display()));
    let report_json: serde_json::Value = serde_json::from_str(&raw)
        .unwrap_or_else(|error| panic!("{} should parse as JSON: {error}", report.display()));
    let unsupported = report_json["unsupported"]
        .as_array()
        .expect("unsupported report bucket should be an array");

    assert!(
        unsupported.iter().all(|entry| {
            entry["reason"].as_str() != Some("Unsupported <br> line-break semantics")
                && entry["kind"].as_str() != Some("Unsupported <br> line-break semantics")
                && entry["error"].as_str() != Some("Unsupported <br> line-break semantics")
        }),
        "regenerated corpus should not use the stale generic br unsupported bucket"
    );
}
```

This replacement belongs in Task 6, not Task 4, because it only passes after regeneration.

- [ ] **Step 5: Run XML parsing and report tests**

Run:

```sh
cargo test -p surgeist-layout --test layout parses_all_checked_in_browser_parity_xml -- --nocapture
cargo test -p surgeist-layout --test layout all_checked_in_browser_parity_xml_has_generator_provenance -- --nocapture
cargo test -p surgeist-layout --test layout browser_parity_generation_report_counts_full_scope -- --nocapture
cargo test -p surgeist-layout --test layout generation_report_no_longer_classifies_horizontal_br_as_unsupported -- --nocapture
```

Expected: all listed tests pass after updating any hard-coded report counts.

- [ ] **Step 6: Run a targeted full parity sweep for the new horizontal BR fixtures**

Run:

```sh
SURGEIST_PARITY_FILTER=subgrid_baseline cargo test -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored --nocapture
```

Expected: The command may still fail with concrete geometry or baseline
mismatch buckets. It must not fail with parse errors, unsupported
`source-tag="br"`, or stale generic unsupported-generation classifications. If
vertical-writing or outside-block-inline-run fixtures remain unsupported, they
should not have generated XML and should not enter this parity run. Record any
new concrete failure buckets in the task output rather than hiding the fixtures.

- [ ] **Step 7: Commit**

```sh
git add tests/layout/browser_parity/xml tests/layout/browser_parity/README.md tests/layout/browser_parity/support.rs
git commit -m "Regenerate parity fixtures with br line breaks"
```

## Task 7: Record Cross-Crate Follow-Up For Real HTML/Style Adapters

**Files:**

- Create: `plans/2026-06-29-surgeist-layout-br-line-break-cross-crate-ledger.md`

- [ ] **Step 1: Create the ledger**

Create `plans/2026-06-29-surgeist-layout-br-line-break-cross-crate-ledger.md`:

```markdown
# Surgeist Layout BR Line Break Cross-Crate Ledger

This ledger records follow-up work outside `surgeist-layout` discovered while
implementing `plans/2026-06-29-surgeist-layout-br-line-break-implementation.md`.

## Entry Status

- `open`: Confirmed cross-crate work remains.
- `reported`: The owning crate or root coordinator has been informed.
- `resolved`: The owning crate has landed the needed change and layout has
  verified against it.

## Entries

### HTML/style adapter needs to lower `<br>` to `InlineRole::LineBreak`

- Status: `open`
- Owning crate: root `surgeist` or the future HTML/DOM adapter crate
- Affected API: `surgeist_layout::NodeInputOf::inline_role`
- Observed behavior: `surgeist-layout` browser parity can lower
  `source-tag="br"` metadata to `InlineRole::LineBreak`, but production HTML
  tree construction outside this crate still needs to map real HTML `<br>`
  elements to that layout input.
- Expected behavior: the real HTML/style adapter should preserve normal style
  resolution for the element and then set `NodeInputOf::inline_role` to
  `InlineRole::LineBreak`, while still allowing `display: none` to suppress the
  break.
- Required owning change: add a root or adapter implementation plan after this
  layout API lands. Do not implement that adapter from the layout crate project.
```

- [ ] **Step 2: Commit**

```sh
git add plans/2026-06-29-surgeist-layout-br-line-break-cross-crate-ledger.md
git commit -m "Record br adapter follow-up"
```

## Task 8: Final Verification And Review

**Files:**

- Review all files changed by Tasks 1-7.

- [ ] **Step 1: Run focused checks**

Run:

```sh
cargo test -p surgeist-layout inline_role -- --nocapture
cargo test -p surgeist-layout line_break -- --nocapture
cargo test -p surgeist-layout --test layout source_tag_br_lowers_to_line_break_role -- --nocapture
cargo test -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- --nocapture
```

Expected: all pass.

- [ ] **Step 2: Run crate baseline**

Run:

```sh
cargo test -p surgeist-layout
cargo clippy -p surgeist-layout --all-targets -- -D warnings
cargo fmt --check
```

Expected: all pass.

- [ ] **Step 3: Run generated-artifact checks**

Run:

```sh
cargo run --manifest-path api/generator/Cargo.toml
cargo run -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- check-taffy-corpus
```

Expected: the API artifact is current and the pinned Taffy corpus check passes.

- [ ] **Step 4: Request final holistic review**

Ask a clean-context reviewer to inspect the final result against this plan, AGENTS.md, and the product modeling guidance. The reviewer must check:

- `<br>` is modeled as a typed line-break primitive, not a normal block box.
- `display: none` suppresses the line break.
- Browser parity still uses the single style lowering path and only sets the layout line-break role from fixture metadata after style lowering.
- Mixed text/element content remains explicitly unsupported.
- Generated XML and reports no longer use the stale generic `<br>` unsupported reason.
- Vertical-writing `<br>` fixtures remain explicitly unsupported with the vertical-writing reason until vertical inline-line progression is designed.
- Horizontal `<br>` fixtures outside supported `display: block` parent contexts remain explicitly unsupported until those parent semantics are designed.
- Cross-crate adapter follow-up is recorded rather than implemented from this repo.

- [ ] **Step 5: Address review findings**

If the reviewer reports findings, assign workers for code fixes and separate reviewers for the fixes. Do not complete this plan until the final holistic reviewer comes back clean.

- [ ] **Step 6: Final commit if review fixes changed files**

If review fixes were needed, commit them:

```sh
git add <changed-files>
git commit -m "Polish br line break support"
```

Completion is when all implementation tasks are done, required verification passes, generated artifacts are current, cross-crate follow-up is recorded, and the final holistic clean-context review is clean.
