# Surgeist Inline Block And Inline Grid Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add browser-compatible atomic inline-level support for `inline-block` first and `inline-grid` second, so Surgeist can run inline-heavy grid/subgrid parity fixtures without hiding unrelated grid failures behind an unsupported inline formatting context bucket.

**Architecture:** Split display behavior into outer participation and inner formatting context. `inline-block`, `inline-grid`, and the already parsed `inline-grid-lanes` participate in a parent block's inline formatting context as atomic inline-level boxes, while their interiors reuse the existing block, grid, and grid-lanes algorithms. Keep this first implementation scoped to atomic inline boxes and line boxes; do not add full non-atomic inline spans, bidi text shaping, ruby, floats inside inline text, or editable caret behavior.

**Tech Stack:** Rust under `crates/surgeist`, layout engine modules in `crates/surgeist/src/layout`, style/CSS lowering in `crates/surgeist/src/style` and `crates/surgeist/src/css`, browser parity harness in `crates/surgeist/tests/layout_browser_parity`, reference algorithms from WebKit `RenderBlock`/`RenderObjectInlines`/`RenderGrid` and Blink `InlineNode`/`InlineItemsBuilder`/`LayoutBlock`/`GridLayoutAlgorithm`, verification with focused `cargo test -p surgeist ...`, full `cargo test -p surgeist`, `cargo fmt --check`, and `cargo clippy -p surgeist --all-targets --all-features -- -D warnings`.

---

## Prerequisite

Complete `docs/superpowers/plans/2026-06-17-surgeist-atomic-inline-oracle-implementation.md` before starting this engine plan. The engine implementation should use the oracle's atomic item facts, line reports, intrinsic width rules, and wrapper facts as the expected model for focused tests.

The first engine test task should add non-ignored production/oracle comparison tests in `crates/surgeist/tests/layout_oracle.rs` for inline-block, inline-grid, and inline-grid-lanes line placement. Do not add intentionally panicking ignored tests; keep all committed verification passing.

---

## Source References

- Current baseline/grid engine plan: `docs/superpowers/plans/2026-06-17-surgeist-baseline-alignment-engine-implementation.md`
- Current broad subgrid parity signal:
  - `840` checked-in subgrid XML fixtures
  - current `SURGEIST_PARITY_FILTER=subgrid` failures: `704`
  - current inline-context bucket: `UnsupportedInlineFormattingContext: 484`
- Current Surgeist display and dispatch surfaces:
  - `crates/surgeist/src/layout/node_input.rs`
  - `crates/surgeist/src/layout/mod.rs`
  - `crates/surgeist/src/layout/block.rs`
  - `crates/surgeist/src/layout/compute.rs`
  - `crates/surgeist/src/style/value.rs`
  - `crates/surgeist/src/style/adapters/layout.rs`
  - `crates/surgeist/src/css/mod.rs`
  - `crates/surgeist/tests/layout_browser_parity/support.rs`
  - `crates/surgeist/tests/layout_browser_parity.rs`
  - `crates/surgeist/tests/support/oracle_tree.rs`
- Existing text/inline-box vocabulary worth reusing conceptually, not importing into layout yet:
  - `crates/surgeist/src/text/source.rs`
  - `crates/surgeist/src/text/layout.rs`
- WebKit references:
  - `tmp/WebKit/Source/WebCore/rendering/RenderBlock.cpp`
  - `tmp/WebKit/Source/WebCore/rendering/RenderObjectInlines.h`
  - `tmp/WebKit/Source/WebCore/rendering/RenderElement.cpp`
  - `tmp/WebKit/Source/WebCore/rendering/RenderGrid.cpp`
- Blink references:
  - `tmp/chromium-inline/third_party/blink/renderer/core/layout/inline/inline_node.cc`
  - `tmp/chromium-inline/third_party/blink/renderer/core/layout/inline/inline_items_builder.cc`
  - `tmp/chromium-inline/third_party/blink/renderer/core/layout/layout_block.cc`
  - `tmp/chromium-inline/third_party/blink/renderer/core/layout/grid/grid_layout_algorithm.cc`

---

## Browser Algorithm Lessons To Preserve

- WebKit marks block renderers with inline displays as `BlockLevelReplacedOrAtomicInline` and treats `display().isInlineType()` as atomic inline-level participation for non-replaced block/grid renderers.
- WebKit creates the same `RenderGrid` renderer for block grid and inline grid; the inline distinction changes parent participation, not the inner grid algorithm.
- Blink emits atomic inline boxes into the inline item stream using an object replacement character, then lays out the associated layout object as an indivisible inline item.
- Blink anonymous block creation normalizes `inline-grid` to an inner `grid` and `inline-grid-lanes` to an inner `grid-lanes`.
- Blink grid intrinsic sizing explicitly calls out `display: inline-grid` cases whose column min/max contribution can depend on grid-area block size; this plan should preserve the existing Surgeist grid intrinsic sizing passes instead of adding a special inline-grid sizing shortcut.

---

## Boundary Decisions

- [ ] Implement only atomic inline-level boxes: `inline-block`, `inline-grid`, and `inline-grid-lanes`.
- [ ] Do not implement full `display: inline`, inline text runs, bidi ordering, ruby, line breaking inside text, selection, or caret geometry in this plan.
- [ ] Do not alias `inline-block` to `block` after Task 2, and do not alias `inline-grid`/`inline-grid-lanes` to grid displays after Task 7. Preserve the outer display once the corresponding atomic inline path exists.
- [ ] Let inline-grid reuse `compute_grid` and inline-block reuse `compute_block`/`compute_leaf` internally.
- [ ] Make empty inline-grid use grid layout, not the leaf fast path, because track templates and grid gaps can create a nonzero box without children.
- [ ] Keep line layout in `crates/surgeist/src/layout/inline.rs`; keep block layout responsible only for identifying inline runs and inserting their anonymous-line contribution into normal block flow.
- [ ] Commit after logical checkpoints with short concrete messages.

---

## Implementation Overview

Surgeist currently has one `layout::Display` enum with block-level `Block`, `Flex`, `Grid`, `GridLanes`, and `None`. CSS style values know `InlineGrid` and `InlineGridLanes`, but not `InlineBlock`; `style::adapters::layout::lower_display` rejects `InlineGrid`; the parity XML parser currently aliases `inline-block` to `Block` and `inline-grid` to `Grid`, then classifies any fixture containing those raw strings as `UnsupportedInlineFormattingContext`.

The implementation should introduce inline display values at the layout boundary and add a small atomic inline formatting context for block containers:

```text
parent block formatting context
  block child                 -> existing block-flow placement
  inline-block child          -> atomic inline item, inner display block
  inline-grid child           -> atomic inline item, inner display grid
  inline-grid-lanes child     -> atomic inline item, inner display grid-lanes
  block child                 -> existing block-flow placement
```

Atomic inline line layout uses border-box metrics from the child output:

```text
item advance = margin-left + border-box width + margin-right
item baseline = child.first_baselines.y.unwrap_or(child.size.height)
line baseline = max(item baseline)
line descent = max(item border-box height - item baseline)
line height = line baseline + line descent
item x = line cursor + margin-left
item y = line y + line baseline - item baseline
```

For intrinsic widths:

```text
max-content inline run width = sum(item advances)
min-content inline run width = max(item advances)
definite inline run width = wrap between atomic items when the next item would overflow a non-empty line
```

This is intentionally smaller than browser inline layout, but it covers the WPT fixture pattern that uses inline-block/inline-grid as atomic wrappers around grid/subgrid content.

---

## Task 1: Add Layout Display Vocabulary And Helpers

**Files:**
- Modify: `crates/surgeist/src/layout/node_input.rs`
- Modify: `crates/surgeist/src/layout/mod.rs`
- Modify: `crates/surgeist/src/layout/tests.rs`

- [ ] Add failing tests for outer/inner display behavior in `crates/surgeist/src/layout/tests.rs`.

```rust
#[test]
fn inline_display_values_preserve_outer_participation_and_inner_context() {
    assert!(Display::InlineBlock.is_inline_level());
    assert!(Display::InlineGrid.is_inline_level());
    assert!(Display::InlineGridLanes.is_inline_level());

    assert_eq!(Display::InlineBlock.inner_display(), Display::Block);
    assert_eq!(Display::InlineGrid.inner_display(), Display::Grid);
    assert_eq!(Display::InlineGridLanes.inner_display(), Display::GridLanes);

    assert!(!Display::Block.is_inline_level());
    assert_eq!(Display::Grid.inner_display(), Display::Grid);
}

#[test]
fn grid_formatting_context_values_include_inline_grid_variants() {
    assert!(Display::Grid.establishes_grid_formatting_context());
    assert!(Display::GridLanes.establishes_grid_formatting_context());
    assert!(Display::InlineGrid.establishes_grid_formatting_context());
    assert!(Display::InlineGridLanes.establishes_grid_formatting_context());
    assert!(!Display::InlineBlock.establishes_grid_formatting_context());
}
```

- [ ] Run the failing tests.

```bash
cargo test -p surgeist --lib inline_display
cargo test -p surgeist --lib grid_formatting_context_values_include_inline_grid_variants
```

Expected: compile failure because the inline layout display variants and helper methods do not exist.

- [ ] Add inline display variants and helpers to `crates/surgeist/src/layout/node_input.rs`.

Expected code shape:

```rust
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Display {
    Block,
    #[default]
    Flex,
    Grid,
    GridLanes,
    InlineBlock,
    InlineGrid,
    InlineGridLanes,
    None,
}

impl Display {
    #[must_use]
    pub const fn is_inline_level(self) -> bool {
        matches!(self, Self::InlineBlock | Self::InlineGrid | Self::InlineGridLanes)
    }

    #[must_use]
    pub const fn inner_display(self) -> Self {
        match self {
            Self::InlineBlock => Self::Block,
            Self::InlineGrid => Self::Grid,
            Self::InlineGridLanes => Self::GridLanes,
            display => display,
        }
    }

    #[must_use]
    pub const fn establishes_grid_formatting_context(self) -> bool {
        matches!(self.inner_display(), Self::Grid | Self::GridLanes)
    }
}
```

- [ ] Run the tests again.

```bash
cargo test -p surgeist --lib inline_display
cargo test -p surgeist --lib grid_formatting_context_values_include_inline_grid_variants
```

Expected: pass.

- [ ] Commit.

```bash
git add crates/surgeist/src/layout/node_input.rs crates/surgeist/src/layout/tests.rs
git commit -m "Add inline display vocabulary"
```

---

## Task 2: Parse And Lower Inline-Block Only

**Files:**
- Modify: `crates/surgeist/src/style/value.rs`
- Modify: `crates/surgeist/src/css/mod.rs`
- Modify: `crates/surgeist/src/style/adapters/layout.rs`
- Modify: `crates/surgeist/tests/css.rs`
- Modify: `crates/surgeist/tests/style.rs`
- Modify: `crates/surgeist/tests/layout_browser_parity/support.rs`

- [ ] Add failing CSS/style tests for `inline-block`.

Add to `crates/surgeist/tests/css.rs`:

```rust
#[test]
fn parses_inline_block_display() {
    let sheet = css::parse_sheet(".panel { display: inline-block; }").unwrap();
    let declarations = sheet.rules()[0].declarations();

    assert_eq!(
        declarations.get(s::Property::Display),
        Some(&s::Value::Display(s::Display::InlineBlock))
    );
}
```

Add to `crates/surgeist/tests/style.rs` near the existing layout adapter display tests:

```rust
#[test]
fn lowers_inline_block_to_layout_inline_block() {
    let tree = FixtureTree::new(vec![FixtureNode::new("root")]);
    let mut resolver = s::Resolver::new(s::Sheet::new());
    let resolved = resolver
        .resolve(s::Context::new(&tree, 0).local(
            &s::Declarations::new().display(s::Display::InlineBlock),
        ))
        .unwrap();
    let layout = s::adapters::layout::lower(&resolved).unwrap();

    assert_eq!(layout.display, l::Display::InlineBlock);
}
```

Add to `crates/surgeist/tests/layout_browser_parity/support.rs` tests:

```rust
#[test]
fn parse_display_preserves_inline_block() {
    assert_eq!(parse_display("inline-block").unwrap(), layout::Display::InlineBlock);
}
```

- [ ] Run the failing tests.

```bash
cargo test -p surgeist --test css parses_inline_block_display
cargo test -p surgeist --test style lowers_inline_block_to_layout_inline_block
cargo test -p surgeist --test layout_browser_parity parse_display_preserves_inline_block
```

Expected: `inline-block` parse/lower failures and XML parser alias failure.

- [ ] Add `InlineBlock` to `style::Display` and CSS parsing. Leave existing `InlineGrid` and `InlineGridLanes` style values in place, but do not newly expose them through `style::adapters::layout` until Task 7.

Expected enum shape in `crates/surgeist/src/style/value.rs`:

```rust
pub enum Display {
    Block,
    #[default]
    Flex,
    Grid,
    InlineBlock,
    InlineGrid,
    GridLanes,
    InlineGridLanes,
    None,
}
```

Expected CSS parser branch in `crates/surgeist/src/css/mod.rs`:

```rust
"inline-block" => Ok(Display::InlineBlock),
```

- [ ] Lower only `InlineBlock` to a layout inline display in `crates/surgeist/src/style/adapters/layout.rs`.

Expected `lower_display` shape:

```rust
fn lower_display(display: Display) -> Result<layout::Display> {
    match display {
        Display::Block => Ok(layout::Display::Block),
        Display::Flex => Ok(layout::Display::Flex),
        Display::Grid => Ok(layout::Display::Grid),
        Display::GridLanes => Ok(layout::Display::GridLanes),
        Display::InlineBlock => Ok(layout::Display::InlineBlock),
        Display::InlineGrid => Err(unsupported("inline grid display")),
        Display::InlineGridLanes => Err(unsupported("inline grid-lanes display")),
        Display::None => Ok(layout::Display::None),
    }
}
```

- [ ] Preserve `inline-block` in the parity XML parser and update `to_style_display` for all layout display variants.

Expected `parse_display` shape in `crates/surgeist/tests/layout_browser_parity/support.rs`:

```rust
fn parse_display(raw: &str) -> Result<layout::Display, Error> {
    match raw {
        "block" => Ok(layout::Display::Block),
        "inline-block" => Ok(layout::Display::InlineBlock),
        "flex" => Ok(layout::Display::Flex),
        "grid" => Ok(layout::Display::Grid),
        "grid-lanes" => Ok(layout::Display::GridLanes),
        "none" => Ok(layout::Display::None),
        _ => Err(Error::new(format!("unsupported display `{raw}`"))),
    }
}
```

Expected `to_style_display` shape:

```rust
fn to_style_display(value: layout::Display) -> s::Display {
    match value {
        layout::Display::Block => s::Display::Block,
        layout::Display::Flex => s::Display::Flex,
        layout::Display::Grid => s::Display::Grid,
        layout::Display::GridLanes => s::Display::GridLanes,
        layout::Display::InlineBlock => s::Display::InlineBlock,
        layout::Display::InlineGrid => s::Display::InlineGrid,
        layout::Display::InlineGridLanes => s::Display::InlineGridLanes,
        layout::Display::None => s::Display::None,
    }
}
```

- [ ] Update any exhaustive matches that now fail to compile by using `display.inner_display()` only at dispatch boundaries, not at parse/lower boundaries. Do not make `inline-grid` or `inline-grid-lanes` reach ordinary layout through style lowering yet.

- [ ] Run targeted tests.

```bash
cargo test -p surgeist --test css parses_inline_block_display
cargo test -p surgeist --test style lowers_inline_block_to_layout_inline_block
cargo test -p surgeist --test layout_browser_parity parse_display_preserves_inline_block
cargo test -p surgeist --test layout_browser_parity parses_all_checked_in_browser_parity_xml
```

Expected: pass.

- [ ] Commit.

```bash
git add crates/surgeist/src/style/value.rs crates/surgeist/src/css/mod.rs crates/surgeist/src/style/adapters/layout.rs crates/surgeist/tests/css.rs crates/surgeist/tests/style.rs crates/surgeist/tests/layout_browser_parity/support.rs
git commit -m "Parse inline-block display"
```

---

## Task 3: Make Display Dispatch Use Inner Formatting Contexts

**Files:**
- Modify: `crates/surgeist/tests/support/oracle_tree.rs`
- Modify: `crates/surgeist/tests/layout_browser_parity/support.rs`
- Modify: `crates/surgeist/tests/layout/flex.rs`
- Modify: `crates/surgeist/tests/layout/grid.rs`
- Modify: `crates/surgeist/tests/layout.rs`

- [ ] Add failing dispatch tests that prove empty grid-formatting-context boxes are grid, not leaf.

Add to `crates/surgeist/tests/layout.rs`:

```rust
#[test]
fn empty_inline_grid_uses_grid_tracks_instead_of_leaf_measurement() {
    let mut tree = support::oracle_tree::OracleTree::new()
        .style(
            0,
            NodeInput {
                display: Display::InlineGrid,
                grid_template_columns: vec![TrackComponent::px(40.0)],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        );

    let output = tree.compute_child(
        0,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::NONE,
            available: Size::splat(Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size, Size::new(40.0, 20.0));
}
```

- [ ] Run the failing test.

```bash
cargo test -p surgeist --test layout empty_inline_grid_uses_grid_tracks_instead_of_leaf_measurement
```

Expected: fail until internal dispatch treats `InlineGrid` as grid for direct layout API calls.

- [ ] Update every test-tree dispatch match to inspect `node_input.display.inner_display()`.

Expected match shape:

```rust
match node_input.display.inner_display() {
    Display::Block => compute_block(self, node, input),
    Display::Flex => compute_flex(self, node, input),
    Display::Grid | Display::GridLanes => compute_grid(self, node, input),
    Display::None => compute_hidden(self, node),
    Display::InlineBlock | Display::InlineGrid | Display::InlineGridLanes => unreachable!(),
}
```

- [ ] Replace leaf fast paths with a helper predicate.

Expected helper:

```rust
fn can_use_leaf_measurement(display: Display, child_count: usize) -> bool {
    child_count == 0 && !display.establishes_grid_formatting_context()
}
```

Use it where current code says `if self.nodes[node].children.is_empty() { compute_leaf(...) }`.

- [ ] Run dispatch-focused tests.

```bash
cargo test -p surgeist --test layout empty_inline_grid_uses_grid_tracks_instead_of_leaf_measurement
cargo test -p surgeist --test layout -- grid
cargo test -p surgeist --test layout_browser_parity parses_all_checked_in_browser_parity_xml
```

Expected: pass.

- [ ] Commit.

```bash
git add crates/surgeist/tests/support/oracle_tree.rs crates/surgeist/tests/layout_browser_parity/support.rs crates/surgeist/tests/layout/flex.rs crates/surgeist/tests/layout/grid.rs crates/surgeist/tests/layout.rs
git commit -m "Dispatch inline displays by inner context"
```

---

## Task 4: Add Atomic Inline Line Layout Primitives

**Files:**
- Create: `crates/surgeist/src/layout/inline.rs`
- Modify: `crates/surgeist/src/layout/mod.rs`
- Modify: `crates/surgeist/src/layout/inline.rs` with inline `#[cfg(test)]` tests. If this grows large enough to split into `crates/surgeist/src/layout/inline/tests.rs`, update the commit staging list in this task before committing.

- [ ] Add failing pure tests for line metrics, wrapping, and intrinsic widths.

Expected tests:

```rust
#[test]
fn atomic_inline_line_aligns_items_to_max_baseline() {
    let report = layout_atomic_inline_items(AtomicInlineInput {
        available_width: Available::definite(200.0),
        items: vec![
            AtomicInlineItem::new(0, Size::new(20.0, 10.0), Edges::ZERO, Some(7.0)),
            AtomicInlineItem::new(1, Size::new(10.0, 20.0), Edges::ZERO, Some(12.0)),
        ],
    });

    assert_eq!(report.size, Size::new(30.0, 20.0));
    assert_eq!(report.first_baseline, Some(12.0));
    assert_eq!(report.items[0].location, Point::new(0.0, 5.0));
    assert_eq!(report.items[1].location, Point::new(20.0, 0.0));
}

#[test]
fn atomic_inline_items_wrap_between_items_for_definite_width() {
    let report = layout_atomic_inline_items(AtomicInlineInput {
        available_width: Available::definite(25.0),
        items: vec![
            AtomicInlineItem::new(0, Size::new(20.0, 10.0), Edges::ZERO, Some(10.0)),
            AtomicInlineItem::new(1, Size::new(20.0, 10.0), Edges::ZERO, Some(10.0)),
        ],
    });

    assert_eq!(report.size, Size::new(20.0, 20.0));
    assert_eq!(report.items[1].location, Point::new(0.0, 10.0));
}

#[test]
fn atomic_inline_intrinsic_widths_use_max_item_and_sum() {
    let items = vec![
        AtomicInlineItem::new(0, Size::new(25.0, 10.0), Edges::ZERO, Some(10.0)),
        AtomicInlineItem::new(1, Size::new(100.0, 10.0), Edges::ZERO, Some(10.0)),
        AtomicInlineItem::new(2, Size::new(50.0, 10.0), Edges::ZERO, Some(10.0)),
    ];

    assert_eq!(atomic_inline_min_content_width(&items), 100.0);
    assert_eq!(atomic_inline_max_content_width(&items), 175.0);
}
```

- [ ] Run the failing tests.

```bash
cargo test -p surgeist --lib atomic_inline
```

Expected: compile failure because `layout::inline` does not exist.

- [ ] Implement the pure primitives.

Expected public-to-crate API:

```rust
pub(super) struct AtomicInlineInput {
    pub available_width: Available,
    pub items: Vec<AtomicInlineItem>,
}

pub(super) struct AtomicInlineItem {
    pub order: u32,
    pub size: Size,
    pub content_size: Size,
    pub margin: Edges,
    pub padding: Edges,
    pub border: Edges,
    pub scrollbar_size: Size,
    pub first_baseline: Option<Scalar>,
}

impl AtomicInlineItem {
    #[cfg(test)]
    pub(super) fn new(
        order: u32,
        size: Size,
        margin: Edges,
        first_baseline: Option<Scalar>,
    ) -> Self {
        Self {
            order,
            size,
            content_size: size,
            margin,
            padding: Edges::ZERO,
            border: Edges::ZERO,
            scrollbar_size: Size::ZERO,
            first_baseline,
        }
    }
}

pub(super) struct AtomicInlineLayoutItem {
    pub order: u32,
    pub location: Point,
    pub size: Size,
    pub content_size: Size,
    pub margin: Edges,
    pub padding: Edges,
    pub border: Edges,
    pub scrollbar_size: Size,
}

pub(super) struct AtomicInlineReport {
    pub size: Size,
    pub content_size: Size,
    pub first_baseline: Option<Scalar>,
    pub last_baseline: Option<Scalar>,
    pub items: Vec<AtomicInlineLayoutItem>,
}
```

Algorithm requirements:

```text
advance = margin.left + size.width + margin.right
baseline = first_baseline.unwrap_or(size.height)
descent = size.height - baseline
line starts a new row if available width is definite, current line is non-empty, and current_width + advance > available width
min-content width = max advance
max-content width = sum advances
content_size.width = max line width
content_size.height = sum line heights
first_baseline = first line baseline
last_baseline = distance from top to last line baseline
```

- [ ] Run focused tests.

```bash
cargo test -p surgeist --lib atomic_inline
```

Expected: pass.

- [ ] Commit.

```bash
git add crates/surgeist/src/layout/inline.rs crates/surgeist/src/layout/mod.rs
git commit -m "Add atomic inline layout primitives"
```

---

## Task 5: Integrate Inline Runs Into Block Layout

**Files:**
- Modify: `crates/surgeist/src/layout/block.rs`
- Modify: `crates/surgeist/tests/layout/block.rs`

- [ ] Add failing block layout tests for an inline run between block children.

Add tests:

```rust
#[test]
fn block_lays_out_atomic_inline_children_on_one_line() {
    let mut tree = support::oracle_tree::OracleTree::new()
        .children(0, [1, 2])
        .style(0, NodeInput { display: Display::Block, ..NodeInput::DEFAULT })
        .style(1, NodeInput {
            display: Display::InlineBlock,
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
            ..NodeInput::DEFAULT
        })
        .style(2, NodeInput {
            display: Display::InlineBlock,
            size: Size::new(Dimension::px(30.0), Dimension::px(20.0)),
            ..NodeInput::DEFAULT
        });

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0)));
    round_layout(&mut tree, 0);

    assert_eq!(tree.final_layout(1).unwrap().location, Point::new(0.0, 10.0));
    assert_eq!(tree.final_layout(2).unwrap().location, Point::new(20.0, 0.0));
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(100.0, 20.0));
}

#[test]
fn block_wraps_atomic_inline_children_between_items() {
    let mut tree = support::oracle_tree::OracleTree::new()
        .children(0, [1, 2])
        .style(0, NodeInput { display: Display::Block, ..NodeInput::DEFAULT })
        .style(1, NodeInput {
            display: Display::InlineBlock,
            size: Size::new(Dimension::px(30.0), Dimension::px(10.0)),
            ..NodeInput::DEFAULT
        })
        .style(2, NodeInput {
            display: Display::InlineBlock,
            size: Size::new(Dimension::px(30.0), Dimension::px(10.0)),
            ..NodeInput::DEFAULT
        });

    compute_root(&mut tree, 0, Size::new(Available::definite(40.0), Available::MAX_CONTENT));
    round_layout(&mut tree, 0);

    assert_eq!(tree.final_layout(1).unwrap().location, Point::new(0.0, 0.0));
    assert_eq!(tree.final_layout(2).unwrap().location, Point::new(0.0, 10.0));
    assert_eq!(tree.final_layout(0).unwrap().size.height, 20.0);
}
```

These tests use `support::oracle_tree::OracleTree`, already available from `crates/surgeist/tests/layout.rs`.

- [ ] Run failing tests.

```bash
cargo test -p surgeist --test layout block_lays_out_atomic_inline_children_on_one_line
cargo test -p surgeist --test layout block_wraps_atomic_inline_children_between_items
```

Expected: fail because block layout still places all in-flow children as block-level boxes.

- [ ] Refactor `layout_in_flow_children` to process child runs.

Expected structure:

```rust
enum InFlowRun<Node> {
    Block(Node),
    Inline(Vec<Node>),
}

fn collect_in_flow_runs<Tree>(
    tree: &Tree,
    children: &[<Tree as Traverse>::Node],
) -> Vec<InFlowRun<<Tree as Traverse>::Node>>
where
    Tree: Compute,
{
    // Absolute-positioned, display:none, and floats remain handled by the caller.
}
```

Implementation rule:

```text
Consecutive normal-flow children whose display.is_inline_level() form one Inline run.
All other normal-flow children form one Block run.
```

- [ ] Add a block-local helper that computes one inline run into `AtomicInlineItem`s, calls `layout_atomic_inline_items`, and writes `NodeOutput` for each child when `set_layout` is true.

Expected helper shape:

```rust
fn layout_atomic_inline_run<Tree>(
    tree: &mut Tree,
    run: &[<Tree as Traverse>::Node],
    order_start: u32,
    cursor_y: Scalar,
    constants: &Constants,
    input: ComputeInput,
    node_inner_size: Size<Option<Scalar>>,
    set_layout: bool,
) -> InlineRunPlacement
where
    Tree: Compute,
```

The helper must compute each child with:

```rust
ComputeInput {
    run_mode: input.run_mode.for_child(),
    sizing_mode: SizingMode::InherentSize,
    axis: RequestedAxis::Both,
    known: Size::NONE,
    parent: Size::new(node_inner_size.width, None),
    available: Size::new(
        node_inner_size
            .width
            .map(Available::definite)
            .unwrap_or(input.available.width),
        Available::MAX_CONTENT,
    ),
}
```

- [ ] Make inline runs contribute to block content size and baselines.

Rules:

```text
run top starts at cursor_y after active collapsed margin resolves
run has no own margins and does not collapse with parent or siblings
run content width contributes like a child content width
run content height advances cursor_y by report.size.height
block first baseline becomes first inline run baseline when no earlier in-flow block establishes one
block last baseline becomes last inline run baseline when it is the last baseline-contributing in-flow content
```

- [ ] Run focused block tests.

```bash
cargo test -p surgeist --test layout block_lays_out_atomic_inline_children_on_one_line
cargo test -p surgeist --test layout block_wraps_atomic_inline_children_between_items
cargo test -p surgeist --test layout -- block
```

Expected: pass.

- [ ] Commit.

```bash
git add crates/surgeist/src/layout/block.rs crates/surgeist/tests/layout/block.rs
git commit -m "Lay out atomic inline runs in blocks"
```

---

## Task 6: Complete Inline-Block Sizing And Baseline Behavior

**Files:**
- Modify: `crates/surgeist/src/layout/block.rs`
- Modify: `crates/surgeist/src/layout/compute.rs`
- Modify: `crates/surgeist/tests/layout/block.rs`
- Modify: `crates/surgeist/tests/layout_browser_parity.rs`

- [ ] Add failing inline-block intrinsic tests.

Expected tests:

```rust
#[test]
fn inline_block_intrinsic_width_shrink_wraps_children() {
    let mut tree = support::oracle_tree::OracleTree::new()
        .children(0, [1])
        .children(1, [2])
        .style(0, NodeInput { display: Display::Block, ..NodeInput::DEFAULT })
        .style(1, NodeInput { display: Display::InlineBlock, ..NodeInput::DEFAULT })
        .style(2, NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(70.0), Dimension::px(20.0)),
            ..NodeInput::DEFAULT
        });

    compute_root(&mut tree, 0, Size::splat(Available::MAX_CONTENT));
    round_layout(&mut tree, 0);

    assert_eq!(tree.final_layout(1).unwrap().size, Size::new(70.0, 20.0));
    assert_eq!(tree.final_layout(0).unwrap().size.width, 70.0);
}

#[test]
fn inline_block_uses_bottom_synthesized_baseline_when_child_has_no_baseline() {
    let mut tree = support::oracle_tree::OracleTree::new()
        .children(0, [1, 2])
        .style(0, NodeInput { display: Display::Block, ..NodeInput::DEFAULT })
        .style(1, NodeInput {
            display: Display::InlineBlock,
            size: Size::new(Dimension::px(10.0), Dimension::px(10.0)),
            ..NodeInput::DEFAULT
        })
        .style(2, NodeInput {
            display: Display::InlineBlock,
            size: Size::new(Dimension::px(10.0), Dimension::px(20.0)),
            ..NodeInput::DEFAULT
        });

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0)));
    round_layout(&mut tree, 0);

    assert_eq!(tree.final_layout(1).unwrap().location.y, 10.0);
    assert_eq!(tree.final_layout(2).unwrap().location.y, 0.0);
}
```

- [ ] Run failing tests.

```bash
cargo test -p surgeist --test layout inline_block_intrinsic_width_shrink_wraps_children
cargo test -p surgeist --test layout inline_block_uses_bottom_synthesized_baseline_when_child_has_no_baseline
```

Expected: fail until inline-block wrappers fully use inner block layout and synthesized baselines.

- [ ] Verify leaf `inline-block` computes like a leaf with inline outer participation.

`compute_leaf` should not need a special branch: an inline-block leaf with explicit width/height is measured like a block leaf internally, while the parent block places it as an atomic inline item.

- [ ] Ensure `compute_block` can be called for a node whose own `display` is `InlineBlock`.

Audit conditions such as:

```rust
style.display == super::Display::Block
style.display != super::Display::Block
```

When the check is about the node's inner formatting context, use:

```rust
style.display.inner_display() == super::Display::Block
```

When the check is about parent participation, use:

```rust
style.display.is_inline_level()
```

- [ ] Update the unsupported inline classifier so it only buckets fixtures containing inline displays that still fail before reaching comparison. For this task, keep the bucket but add a narrower name.

Expected test update:

```rust
assert_eq!(
    classified_error_kind(&golden, "root/0: width mismatch, expected 10, got 0"),
    "InlineFormattingContextMismatch"
);
```

- [ ] Run inline-block-focused parity slices.

```bash
SURGEIST_PARITY_FILTER=subgrid_baseline cargo test -p surgeist --test layout_browser_parity -- runs_all_checked_in_browser_parity_xml --ignored
SURGEIST_PARITY_FILTER=subgrid_auto_track_sizing_min_content_text_runs cargo test -p surgeist --test layout_browser_parity -- runs_all_checked_in_browser_parity_xml --ignored
```

Expected: failures may remain, but they should no longer be classified as unsupported inline formatting context solely because `inline-block` exists.

- [ ] Run focused tests.

```bash
cargo test -p surgeist --test layout -- block
cargo test -p surgeist --test layout_browser_parity parses_all_checked_in_browser_parity_xml
```

Expected: pass.

- [ ] Commit.

```bash
git add crates/surgeist/src/layout/block.rs crates/surgeist/src/layout/compute.rs crates/surgeist/tests/layout/block.rs crates/surgeist/tests/layout_browser_parity.rs
git commit -m "Complete inline-block layout behavior"
```

---

## Task 7: Add Inline-Grid Atomic Wrapper Behavior

**Files:**
- Modify: `crates/surgeist/src/style/adapters/layout.rs`
- Modify: `crates/surgeist/src/layout/grid/mod.rs`
- Modify: `crates/surgeist/src/layout/grid/subgrid.rs`
- Modify: `crates/surgeist/src/layout/grid/lanes.rs`
- Modify: `crates/surgeist/tests/style.rs`
- Modify: `crates/surgeist/tests/layout/grid.rs`
- Modify: `crates/surgeist/tests/layout_browser_parity/support.rs`
- Modify: `crates/surgeist/tests/layout_browser_parity.rs`
- Modify: `crates/surgeist/tests/support/grid_layout_comparison.rs`

- [ ] Add failing style and parity parser tests for `inline-grid` and `inline-grid-lanes`.

Add to `crates/surgeist/tests/style.rs`:

```rust
#[test]
fn lowers_inline_grid_displays_to_layout_inline_variants() {
    for (style_display, layout_display) in [
        (s::Display::InlineGrid, l::Display::InlineGrid),
        (s::Display::InlineGridLanes, l::Display::InlineGridLanes),
    ] {
        let tree = FixtureTree::new(vec![FixtureNode::new("root")]);
        let mut resolver = s::Resolver::new(s::Sheet::new());
        let resolved = resolver
            .resolve(s::Context::new(&tree, 0).local(
                &s::Declarations::new().display(style_display),
            ))
            .unwrap();
        let layout = s::adapters::layout::lower(&resolved).unwrap();
        assert_eq!(layout.display, layout_display);
    }
}
```

Add to `crates/surgeist/tests/layout_browser_parity/support.rs` tests:

```rust
#[test]
fn parse_display_preserves_inline_grid_variants() {
    assert_eq!(parse_display("inline-grid").unwrap(), layout::Display::InlineGrid);
    assert_eq!(
        parse_display("inline-grid-lanes").unwrap(),
        layout::Display::InlineGridLanes
    );
}
```

- [ ] Run the failing parser/lowering tests.

```bash
cargo test -p surgeist --test style lowers_inline_grid_displays_to_layout_inline_variants
cargo test -p surgeist --test layout_browser_parity parse_display_preserves_inline_grid_variants
```

Expected: fail because Task 2 intentionally delayed exposing `inline-grid` and `inline-grid-lanes`.

- [ ] Lower inline-grid style values and preserve inline-grid XML values.

Expected `lower_display` branches in `crates/surgeist/src/style/adapters/layout.rs`:

```rust
Display::InlineGrid => Ok(layout::Display::InlineGrid),
Display::InlineGridLanes => Ok(layout::Display::InlineGridLanes),
```

Expected `parse_display` branches in `crates/surgeist/tests/layout_browser_parity/support.rs`:

```rust
"inline-grid" => Ok(layout::Display::InlineGrid),
"inline-grid-lanes" => Ok(layout::Display::InlineGridLanes),
```

Keep the `to_style_display` mappings added in Task 2:

```rust
layout::Display::InlineGrid => s::Display::InlineGrid,
layout::Display::InlineGridLanes => s::Display::InlineGridLanes,
```

- [ ] Add failing inline-grid tests that prove inner grid behavior and outer inline placement.

Expected tests:

```rust
#[test]
fn inline_grid_uses_grid_tracks_and_participates_as_atomic_inline() {
    let mut tree = support::oracle_tree::OracleTree::new()
        .children(0, [1, 2])
        .style(0, NodeInput { display: Display::Block, ..NodeInput::DEFAULT })
        .style(1, NodeInput {
            display: Display::InlineGrid,
            grid_template_columns: vec![TrackComponent::px(40.0)],
            grid_template_rows: vec![TrackComponent::px(20.0)],
            ..NodeInput::DEFAULT
        })
        .style(2, NodeInput {
            display: Display::InlineGrid,
            grid_template_columns: vec![TrackComponent::px(10.0)],
            grid_template_rows: vec![TrackComponent::px(30.0)],
            ..NodeInput::DEFAULT
        });

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0)));
    round_layout(&mut tree, 0);

    assert_eq!(tree.final_layout(1).unwrap().size, Size::new(40.0, 20.0));
    assert_eq!(tree.final_layout(2).unwrap().size, Size::new(10.0, 30.0));
    assert_eq!(tree.final_layout(1).unwrap().location.y, 10.0);
    assert_eq!(tree.final_layout(2).unwrap().location.y, 0.0);
}

#[test]
fn inline_grid_can_host_subgrid_descendant() {
    let subgrid_track = || {
        TrackComponent::Subgrid(surgeist::layout::SubgridTrack {
            line_names: Vec::new(),
        })
    };
    let mut tree = support::oracle_tree::OracleTree::new()
        .children(0, [1])
        .children(1, [2])
        .style(0, NodeInput {
            display: Display::InlineGrid,
            grid_template_columns: vec![TrackComponent::px(80.0)],
            grid_template_rows: vec![TrackComponent::px(30.0)],
            ..NodeInput::DEFAULT
        })
        .style(1, NodeInput {
            display: Display::Grid,
            grid_template_columns: vec![subgrid_track()],
            grid_template_rows: vec![subgrid_track()],
            ..NodeInput::DEFAULT
        })
        .style(2, NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(80.0), Dimension::px(30.0)),
            ..NodeInput::DEFAULT
        });

    let output = tree.compute_child(
        0,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::NONE,
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );
    assert_eq!(output.size, Size::new(80.0, 30.0));
}
```

These tests use `support::oracle_tree::OracleTree` and the existing `TrackComponent::Subgrid` constructor style from `crates/surgeist/tests/layout/grid.rs`.

- [ ] Add browser parity smoke tests for empty `inline-grid` and `inline-grid-lanes` fixtures.

Add to `crates/surgeist/tests/layout_browser_parity/support.rs` tests:

```rust
#[test]
fn empty_inline_grid_fixture_uses_grid_tracks_instead_of_leaf_measurement() {
    let golden = Golden::parse(
        r#"
        <test name="empty-inline-grid" use-rounding="true">
            <viewport width="max-content" height="max-content" />
            <input>
                <div display="inline-grid" grid-template-columns="40px" grid-template-rows="20px" />
            </input>
            <expectations>
                <node x="0" y="0" width="40" height="20" />
            </expectations>
        </test>
        "#,
    )
    .unwrap();

    assert_surgeist_matches(&golden).unwrap();
}

#[test]
fn empty_inline_grid_lanes_fixture_uses_grid_lanes_tracks_instead_of_leaf_measurement() {
    let golden = Golden::parse(
        r#"
        <test name="empty-inline-grid-lanes" use-rounding="true">
            <viewport width="max-content" height="max-content" />
            <input>
                <div display="inline-grid-lanes" grid-template-columns="40px" grid-template-rows="20px" />
            </input>
            <expectations>
                <node x="0" y="0" width="40" height="20" />
            </expectations>
        </test>
        "#,
    )
    .unwrap();

    assert_surgeist_matches(&golden).unwrap();
}
```

These tests belong in the browser parity support module because that is where XML display parsing, style lowering, and the childless-node leaf shortcut meet.

- [ ] Run failing tests.

```bash
cargo test -p surgeist --test layout inline_grid_uses_grid_tracks_and_participates_as_atomic_inline
cargo test -p surgeist --test layout inline_grid_can_host_subgrid_descendant
cargo test -p surgeist --test layout_browser_parity empty_inline_grid_fixture_uses_grid_tracks_instead_of_leaf_measurement
cargo test -p surgeist --test layout_browser_parity empty_inline_grid_lanes_fixture_uses_grid_lanes_tracks_instead_of_leaf_measurement
```

Expected: fail until inline-grid is fully routed to grid internally and atomic inline externally.

- [ ] Audit grid/subgrid display checks across production and test support.

Replace inner-context checks:

```rust
style.display == Display::Grid
style.display == Display::GridLanes
matches!(style.display, Display::Grid | Display::GridLanes)
```

with:

```rust
style.display.inner_display() == Display::Grid
style.display.inner_display() == Display::GridLanes
matches!(style.display.inner_display(), Display::Grid | Display::GridLanes)
```

Keep outer display checks only where inline-level participation matters.

Audit at least:

```text
crates/surgeist/src/layout/grid/mod.rs
crates/surgeist/src/layout/grid/subgrid.rs
crates/surgeist/src/layout/grid/lanes.rs
crates/surgeist/tests/layout/grid.rs
crates/surgeist/tests/support/grid_layout_comparison.rs
crates/surgeist/tests/support/oracle_tree.rs
crates/surgeist/tests/support/oracle/grid/*.rs
crates/surgeist/tests/layout_browser_parity/support.rs
```

If an oracle helper intentionally only models block-level grid containers, document that in the helper's assertion instead of silently accepting inline values.

- [ ] Ensure subgrid eligibility accepts inline-grid containers by inner display.

Expected helper shape in `crates/surgeist/src/layout/grid/subgrid.rs`:

```rust
const fn subgrid_container_display_supported(display: Display) -> bool {
    matches!(display.inner_display(), Display::Grid | Display::GridLanes)
}
```

- [ ] Ensure grid-lanes accepts `InlineGridLanes` internally.

Expected lane-axis checks:

```rust
if style.display.inner_display() == Display::GridLanes {
    // existing grid-lanes behavior
}
```

- [ ] Run focused grid tests.

```bash
cargo test -p surgeist --test layout -- grid
cargo test -p surgeist --test layout_oracle
cargo test -p surgeist --test style lowers_inline_grid_displays_to_layout_inline_variants
cargo test -p surgeist --test layout_browser_parity parse_display_preserves_inline_grid_variants
```

Expected: pass.

- [ ] Run inline-grid-heavy parity slices.

```bash
SURGEIST_PARITY_FILTER=subgrid_standalone_axis cargo test -p surgeist --test layout_browser_parity -- runs_all_checked_in_browser_parity_xml --ignored
SURGEIST_PARITY_FILTER=subgrid_alignment_002 cargo test -p surgeist --test layout_browser_parity -- runs_all_checked_in_browser_parity_xml --ignored
SURGEIST_PARITY_FILTER=grid_lanes_not_inhibited cargo test -p surgeist --test layout_browser_parity -- runs_all_checked_in_browser_parity_xml --ignored
```

Expected: failures may remain as real geometry mismatches, but `inline-grid` and `inline-grid-lanes` should no longer be unsupported parse/lower/dispatch blockers.

- [ ] Commit.

```bash
git add crates/surgeist/src/style/adapters/layout.rs crates/surgeist/src/layout/grid/mod.rs crates/surgeist/src/layout/grid/subgrid.rs crates/surgeist/src/layout/grid/lanes.rs crates/surgeist/tests/style.rs crates/surgeist/tests/layout/grid.rs crates/surgeist/tests/layout_browser_parity/support.rs crates/surgeist/tests/layout_browser_parity.rs crates/surgeist/tests/support/grid_layout_comparison.rs
git commit -m "Support inline-grid atomic wrappers"
```

---

## Task 8: Update Browser Parity Classification And Documentation

**Files:**
- Modify: `crates/surgeist/tests/layout_browser_parity.rs`
- Modify: `crates/surgeist/tests/layout_browser_parity/README.md`

- [ ] Change the classifier so inline displays are not automatically treated as unsupported.

Expected classifier behavior:

```rust
fn classified_error_kind(golden: &support::Golden, error: &str) -> String {
    if has_unimplemented_inline_feature(&golden.root) {
        return "UnsupportedNonAtomicInlineFormattingContext".to_string();
    }
    error_kind(error)
}
```

Expected helper scope:

```rust
fn has_unimplemented_inline_feature(node: &support::Node) -> bool {
    node.style
        .display()
        .is_some_and(|display| matches!(display.as_str(), "inline"))
        || node.children.iter().any(has_unimplemented_inline_feature)
}
```

Do not classify `inline-block`, `inline-grid`, or `inline-grid-lanes` as unsupported after Task 7.

- [ ] Add a regression test.

```rust
#[test]
fn atomic_inline_display_failures_are_not_bucketed_as_unsupported_inline_context() {
    let golden = support::Golden::parse(
        r#"
        <test name="inline-bucket" use-rounding="true">
            <viewport width="max-content" height="max-content" />
            <input>
                <div display="block">
                    <div display="inline-grid" />
                </div>
            </input>
            <expectations>
                <node x="0" y="0" width="0" height="0">
                    <node x="0" y="0" width="0" height="0" />
                </node>
            </expectations>
        </test>
        "#,
    )
    .unwrap();

    assert_eq!(
        classified_error_kind(&golden, "root/0: width mismatch, expected 10, got 0"),
        "width mismatch"
    );
}

#[test]
fn text_leaf_failures_are_not_bucketed_as_unsupported_inline_context() {
    let golden = support::Golden::parse(
        r#"
        <test name="text-bucket" use-rounding="true">
            <viewport width="max-content" height="max-content" />
            <input>
                <text display="block">hello</text>
            </input>
            <expectations>
                <node x="0" y="0" width="0" height="0" />
            </expectations>
        </test>
        "#,
    )
    .unwrap();

    assert_eq!(
        classified_error_kind(&golden, "root: width mismatch, expected 0, got 50"),
        "width mismatch"
    );
}
```

- [ ] Update the README with the new meaning of inline buckets.

Required README text:

```markdown
Atomic inline displays (`inline-block`, `inline-grid`, and `inline-grid-lanes`) are expected to reach normal geometry comparison. The unsupported inline bucket is reserved for non-atomic inline text/span behavior that the tree layout engine does not model yet.
```

- [ ] Run parser/classifier tests.

```bash
cargo test -p surgeist --test layout_browser_parity parse_display_preserves_inline_block
cargo test -p surgeist --test layout_browser_parity parse_display_preserves_inline_grid_variants
cargo test -p surgeist --test layout_browser_parity atomic_inline_display_failures_are_not_bucketed_as_unsupported_inline_context
cargo test -p surgeist --test layout_browser_parity text_leaf_failures_are_not_bucketed_as_unsupported_inline_context
cargo test -p surgeist --test layout_browser_parity parses_all_checked_in_browser_parity_xml
```

Expected: pass.

- [ ] Commit.

```bash
git add crates/surgeist/tests/layout_browser_parity.rs crates/surgeist/tests/layout_browser_parity/README.md
git commit -m "Reclassify atomic inline parity failures"
```

---

## Task 9: Measure Parity And Close Regression Gaps

**Files:**
- Modify only files needed by failures discovered in this task.

- [ ] Run the full focused Surgeist suite.

```bash
cargo test -p surgeist --lib
cargo test -p surgeist --test layout
cargo test -p surgeist --test layout_oracle
cargo test -p surgeist --test oracle
cargo test -p surgeist --test layout_browser_parity parses_all_checked_in_browser_parity_xml
```

Expected: pass.

- [ ] Run the ignored subgrid parity corpus and capture the new failure count.

```bash
SURGEIST_PARITY_FILTER=subgrid cargo test -p surgeist --test layout_browser_parity -- runs_all_checked_in_browser_parity_xml --ignored
```

Expected: the command may still fail, but the failure summary should no longer contain `UnsupportedInlineFormattingContext` for `inline-block`/`inline-grid` fixtures. Record:

```text
before this plan: 704 failing / 840, UnsupportedInlineFormattingContext: 484
after this plan: <copy exact failing count and buckets from test output>
```

- [ ] Run the ignored grid-lanes parity slice because `inline-grid-lanes` currently appears in checked-in grid-lanes XML fixtures.

```bash
SURGEIST_PARITY_FILTER=grid-lanes cargo test -p surgeist --test layout_browser_parity -- runs_all_checked_in_browser_parity_xml --ignored
```

Expected: failures may remain, but `inline-grid-lanes` must not fail at parse/lower/dispatch.

- [ ] Fix only regressions directly caused by this implementation.

Use this rule:

```text
If a failure is caused by atomic inline outer participation, fix it in inline/block code.
If a failure is caused by grid/subgrid/grid-lanes geometry that was previously hidden, classify it as a newly exposed grid issue and do not refactor grid algorithms in this plan unless the fix is smaller than the classification.
If a failure requires non-atomic inline text/span behavior, leave it in the unsupported non-atomic inline bucket.
```

- [ ] Run final verification.

```bash
cargo test -p surgeist
cargo clippy -p surgeist --all-targets --all-features -- -D warnings
cargo fmt --check
git diff --check
git status --short --branch
```

Expected: all commands pass except the intentionally ignored parity command may still report geometry failures when run manually.

- [ ] Commit final fixes and parity documentation.

```bash
git add crates/surgeist
git commit -m "Measure atomic inline parity impact"
```

---

## Execution Notes

- Prefer implementing `inline-block` through Task 6 before touching inline-grid internals. Task 7 should mostly be display dispatch and inner-grid eligibility work if Tasks 1-6 were done correctly.
- The smallest useful first milestone is Tasks 1-4. That milestone should compile and test without changing block layout behavior.
- The first meaningful parity milestone is Task 6. That should reduce or reclassify many `inline-block` subgrid baseline fixtures.
- The second meaningful parity milestone is Task 7. That should expose real subgrid/grid-lanes geometry failures currently hidden by `inline-grid` wrappers.
- If a change starts requiring full text inline layout, stop and write a separate non-atomic inline plan. That is a different subsystem.

---

## Self-Review

- Spec coverage: the plan covers style parsing, layout display vocabulary, dispatch, atomic inline line layout, inline-block, inline-grid, inline-grid-lanes, parity classification, and final verification.
- Placeholder scan: the plan contains no unfinished placeholder steps. Each code-changing task includes exact paths, test snippets or required code shapes, commands, and expected outcomes.
- Type consistency: `Display::InlineBlock`, `Display::InlineGrid`, `Display::InlineGridLanes`, `Display::is_inline_level`, `Display::inner_display`, and `Display::establishes_grid_formatting_context` are introduced in Task 1 and reused consistently afterward.
