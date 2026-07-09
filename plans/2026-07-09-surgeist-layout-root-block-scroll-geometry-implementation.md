# Root And Block Scroll Geometry Output Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Emit typed scroll geometry facts for root and block layout outputs while keeping layout a pure geometry engine.

**Architecture:** Add optional `ScrollGeometryOf<S>` output fields to `ComputeOutputOf<S>` and `NodeOutputOf<S>`, defaulting to `None` so algorithms outside this phase remain explicit non-emitters. Extend `src/scroll.rs` with small scalar-generic helpers that build `ScrollGeometryOf<S>` from Phase 2 box rects, resolved overflow, writing mode, direction, and computed content overflow. Wire those helpers into root and block layout only.

**Tech Stack:** Rust 2024, existing `LayoutScalar`, `ComputeOutputOf<S>`, `NodeOutputOf<S>`, `ScrollGeometryOf<S>`, `ScrollBoxRectsOf<S>`, crate-local tests, `cargo test -p surgeist-layout`, `cargo clippy -p surgeist-layout --all-targets -- -D warnings`, `cargo fmt --check`.

---

## Scope

This implements Phase 3 from:

- `plans/2026-07-09-surgeist-layout-css-scroll-geometry-sequence.md`
- `plans/2026-07-09-surgeist-layout-css-scroll-support-matrix.md`
- `plans/2026-07-09-surgeist-layout-scrollport-gutter-implementation.md`

Phase 3 must:

- expose scroll geometry through final layout outputs for root and block nodes;
- carry no live scroll position;
- preserve existing size, margin-collapse, baseline, and visible-overflow behavior;
- report geometry for `Overflow::Visible`, `Overflow::Hidden`, `Overflow::Clip`, and `Overflow::Scroll`;
- expose non-zero maximum scroll range only on `Hidden` and `Scroll` axes;
- expose clipping rects for `Hidden`, `Clip`, and `Scroll` axes;
- keep `Clip` axes non-scrollable even when content overflows;
- keep flex and grid scroll geometry output out of scope and explicit `None`;
- keep absolute/out-of-flow contribution rules unchanged for this phase.

Phase 3 must not:

- parse CSS, run cascade, lower root/style inputs, route platform input, store live offsets, animate scrolling, or paint scrollbars;
- add `overflow: auto`;
- add `scrollbar-gutter: stable` or `both-edges`;
- add `scroll-padding`, `scroll-margin`, or scroll snap;
- add inherited nested clipping propagation;
- rewrite block visible-overflow accumulation beyond naming the existing content-size behavior as the first scrollable-overflow source.

## Geometry Model Decision

All `ScrollGeometryOf<S>` rects emitted by this phase are node-local physical
rects:

- the node border box starts at `Point::ZERO`;
- padding box, content box, scrollport, gutter rects, overflow clip rect, and
  scrollable overflow rect are expressed in that node-local coordinate space;
- `writing_mode` and `direction` remain attached as metadata for runtime/render
  consumers that need logical interpretation;
- scroll ranges are physical positive-x/positive-y extents, clamped by
  `ScrollRangeOf::clamp`.

Root layout may place the root node at a non-zero parent-relative `location`
for RTL viewport alignment. That `NodeOutput.location` remains parent-relative;
root `scroll_geometry` still uses node-local rects so the contract is identical
to ordinary block outputs.

Scrollable overflow must be origin-bearing. `ComputeOutput.content_size` remains
the legacy size summary, but block scroll geometry must maintain a separate
`ScrollRectOf<S>` union so left/up overflow is not collapsed into a size with an
implicit zero origin:

- start with the Phase 2 content-box rect as the empty/no-overflow baseline;
- ordinary in-flow block margin boxes contribute to the scrollable overflow
  union in node-local coordinates;
- if a child axis is `Overflow::Visible`, that axis may contribute the child's
  own `content_size` beyond its border-box size, starting at the child border-box
  origin for this phase;
- inline runs contribute an `InlineRunPlacement.scrollable_overflow` rect,
  translated from run-local coordinates to the run's node-local origin;
- floats and absolute children keep the current contribution rules for this
  phase, but must contribute origin-bearing rects instead of size-only facts;
- later phases may refine CSS-specific out-of-flow and nested clipping rules,
  but this phase must not lose negative/left/up overflow origins.

Do not pass content-box-relative overflow rects into `ScrollGeometryOf<S>`.
`content_size` may continue using the existing content-box-relative legacy
calculation, but `scroll_geometry.scrollable_overflow()` must be node-local.

## Files

- Modify: `src/scroll.rs`
- Modify: `src/scroll_tests.rs`
- Modify: `src/output.rs`
- Modify: `src/contract_tests.rs`
- Modify: `src/compute.rs`
- Modify: `src/root_tests.rs`
- Modify: `src/block.rs`
- Modify: `src/block_tests.rs`
- Modify compiler-reported direct field initializers in: `src/flex.rs`,
  `src/grid/child.rs`, `src/grid/lanes.rs`,
  `tests/layout/browser_parity/support.rs`, and any tests with direct
  `NodeOutput` or `ComputeOutput` struct literals.

Prefer constructors (`ComputeOutputOf::from_*`, `NodeOutputOf::with_order`) when
updating tests. If a direct struct literal remains, it must set
`scroll_geometry` explicitly.

## Shared Helper API To Add

Extend `src/scroll.rs` with these helpers. Remove `#[allow(dead_code)]` from
Phase 2 helper items as they become used by production algorithms.

```rust
#[must_use]
pub fn scroll_container_facts_from_overflow(
    overflow: Point<Overflow>,
) -> Result<ScrollContainerFacts, ScrollUnsupportedFeature> {
    Ok(ScrollContainerFacts::new(
        ScrollContainerAxis::from_overflow(overflow.x)?,
        ScrollContainerAxis::from_overflow(overflow.y)?,
    ))
}

pub fn scroll_rect_union<S: LayoutScalar>(
    a: ScrollRectOf<S>,
    b: ScrollRectOf<S>,
) -> Result<ScrollRectOf<S>, ScrollUnsupportedFeature> {
    let a_origin = a.origin();
    let b_origin = b.origin();
    let a_size = a.size();
    let b_size = b.size();
    let min_x = a_origin.x.min(b_origin.x);
    let min_y = a_origin.y.min(b_origin.y);
    let max_x = (a_origin.x + a_size.width).max(b_origin.x + b_size.width);
    let max_y = (a_origin.y + a_size.height).max(b_origin.y + b_size.height);

    ScrollRectOf::new(
        Point::new(min_x, min_y),
        Size::new((max_x - min_x).max(S::ZERO), (max_y - min_y).max(S::ZERO)),
    )
}

pub fn scrollable_overflow_from_content_size<S: LayoutScalar>(
    content_box: ScrollRectOf<S>,
    content_size: Size<S>,
) -> Result<ScrollRectOf<S>, ScrollUnsupportedFeature> {
    scroll_rect_union(
        content_box,
        ScrollRectOf::new(
            content_box.origin(),
            Size::new(
                content_box.size().width.max(content_size.width),
                content_box.size().height.max(content_size.height),
            ),
        )?,
    )
}

pub fn scrollable_overflow_from_layout_content_size<S: LayoutScalar>(
    direction: Direction,
    overflow: Point<Overflow>,
    border_box_size: Size<S>,
    padding: Edges<S>,
    border: Edges<S>,
    scrollbar_width: S,
    content_size: Size<S>,
) -> Result<ScrollRectOf<S>, ScrollUnsupportedFeature> {
    let reservation = ScrollbarReservationOf::from_overflow(overflow, scrollbar_width, direction);
    let rects = scroll_box_rects_from_border_box(
        ScrollRectOf::new(Point::ZERO, border_box_size)?,
        padding,
        border,
        reservation,
    )?;
    scrollable_overflow_from_content_size(rects.content_box(), content_size)
}

pub fn scroll_range_from_overflow_rects<S: LayoutScalar>(
    container: ScrollContainerFacts,
    scrollport: ScrollRectOf<S>,
    scrollable_overflow: ScrollRectOf<S>,
) -> Result<ScrollRangeOf<S>, ScrollUnsupportedFeature> {
    let scrollport_origin = scrollport.origin();
    let scrollport_size = scrollport.size();
    let scrollable_origin = scrollable_overflow.origin();
    let scrollable_size = scrollable_overflow.size();
    ScrollRangeOf::new(Size::new(
        if container.x().exposes_scroll_range() {
            ((scrollable_origin.x + scrollable_size.width)
                - (scrollport_origin.x + scrollport_size.width))
                .max(S::ZERO)
        } else {
            S::ZERO
        },
        if container.y().exposes_scroll_range() {
            ((scrollable_origin.y + scrollable_size.height)
                - (scrollport_origin.y + scrollport_size.height))
                .max(S::ZERO)
        } else {
            S::ZERO
        },
    ))
}

#[allow(clippy::too_many_arguments)]
pub fn scroll_geometry_from_layout<S: LayoutScalar>(
    writing_mode: WritingMode,
    direction: Direction,
    overflow: Point<Overflow>,
    border_box_size: Size<S>,
    padding: Edges<S>,
    border: Edges<S>,
    scrollbar_width: S,
    scrollable_overflow: ScrollRectOf<S>,
) -> Result<ScrollGeometryOf<S>, ScrollUnsupportedFeature> {
    let container = scroll_container_facts_from_overflow(overflow)?;
    let reservation = ScrollbarReservationOf::from_overflow(overflow, scrollbar_width, direction);
    let rects = scroll_box_rects_from_border_box(
        ScrollRectOf::new(Point::ZERO, border_box_size)?,
        padding,
        border,
        reservation,
    )?;
    let range =
        scroll_range_from_overflow_rects(container, rects.scrollport(), scrollable_overflow)?;
    let overflow_clip = container
        .requires_overflow_clip()
        .then_some(rects.scrollport());

    ScrollGeometryOf::new(
        writing_mode,
        direction,
        container,
        rects.scrollport(),
        overflow_clip,
        scrollable_overflow,
        range,
        rects.gutters(),
    )
}

pub fn round_scroll_geometry<S: LayoutScalar>(
    geometry: ScrollGeometryOf<S>,
    cumulative_origin: Point<S>,
) -> Result<ScrollGeometryOf<S>, ScrollUnsupportedFeature> {
    let scrollport = round_scroll_rect(geometry.scrollport(), cumulative_origin)?;
    let overflow_clip = geometry
        .overflow_clip()
        .map(|rect| round_scroll_rect(rect, cumulative_origin))
        .transpose()?;
    let scrollable_overflow =
        round_scroll_rect(geometry.scrollable_overflow(), cumulative_origin)?;
    let gutters = ScrollbarGutterRectsOf::new(
        geometry
            .gutters()
            .horizontal()
            .map(|rect| round_scroll_rect(rect, cumulative_origin))
            .transpose()?,
        geometry
            .gutters()
            .vertical()
            .map(|rect| round_scroll_rect(rect, cumulative_origin))
            .transpose()?,
    );
    let range =
        scroll_range_from_overflow_rects(geometry.container(), scrollport, scrollable_overflow)?;

    ScrollGeometryOf::new(
        geometry.writing_mode(),
        geometry.direction(),
        geometry.container(),
        scrollport,
        overflow_clip,
        scrollable_overflow,
        range,
        gutters,
    )
}

fn round_scroll_rect<S: LayoutScalar>(
    rect: ScrollRectOf<S>,
    cumulative_origin: Point<S>,
) -> Result<ScrollRectOf<S>, ScrollUnsupportedFeature> {
    let origin = rect.origin();
    let size = rect.size();
    let rounded_origin = Point::new(
        round(cumulative_origin.x + origin.x) - round(cumulative_origin.x),
        round(cumulative_origin.y + origin.y) - round(cumulative_origin.y),
    );
    let rounded_end = Point::new(
        round(cumulative_origin.x + origin.x + size.width) - round(cumulative_origin.x),
        round(cumulative_origin.y + origin.y + size.height) - round(cumulative_origin.y),
    );
    ScrollRectOf::new(
        rounded_origin,
        Size::new(
            (rounded_end.x - rounded_origin.x).max(S::ZERO),
            (rounded_end.y - rounded_origin.y).max(S::ZERO),
        ),
    )
}

fn round<S: LayoutScalar>(value: S) -> S {
    (value + S::from_f64(0.5)).floor()
}
```

Keep these helpers inside the existing private `scroll` module unless a reviewer
requires public reexports. `ScrollGeometryOf<S>` is already public; this helper
is an algorithm construction utility, not a root/runtime API surface.

## Task 1: Add Scroll Output Fields

**Files:**

- Modify: `src/output.rs`
- Modify: `src/contract_tests.rs`
- Modify direct struct literal sites reported by the compiler across crate
  tests and support code.

- [ ] **Step 1: Add failing output-contract tests**

Append or update tests in `src/contract_tests.rs`:

```rust
#[test]
fn compute_output_defaults_to_no_scroll_geometry() {
    let output = ComputeOutput::from_outer_size(Size::new(10.0, 20.0));

    assert_eq!(output.scroll_geometry, None);
}

#[test]
fn node_output_defaults_to_no_scroll_geometry() {
    let output = NodeOutput::with_order(7);

    assert_eq!(output.scroll_geometry, None);
}
```

Run:

```sh
cargo test -p surgeist-layout scroll_geometry -- --nocapture
```

Expected: compile failure naming missing `scroll_geometry` fields.

- [ ] **Step 2: Add output fields**

In `src/output.rs`, add `ScrollGeometryOf` to the import list:

```rust
use super::{AvailableOf, DefaultScalar, Edges, LayoutScalar, Point, ScrollGeometryOf, Size};
```

Add this field to `ComputeOutputOf<S>`:

```rust
pub scroll_geometry: Option<ScrollGeometryOf<S>>,
```

Set it to `None` in `ComputeOutputOf::HIDDEN` and
`ComputeOutputOf::from_sizes_and_baselines`.

Add this field to `NodeOutputOf<S>`:

```rust
pub scroll_geometry: Option<ScrollGeometryOf<S>>,
```

Set it to `None` in `NodeOutputOf::with_order`.

- [ ] **Step 3: Fix direct struct literals**

Run:

```sh
cargo test -p surgeist-layout scroll_geometry -- --nocapture
```

Expected: compile errors at any remaining direct `ComputeOutputOf` or
`NodeOutputOf` struct literals that need `scroll_geometry: None`.

For every direct output struct literal that is not part of root/block scroll
emission, add:

```rust
scroll_geometry: None,
```

Do not add placeholder scroll geometry to flex, grid, hidden, browser fixture
support, or unrelated tests in this task.

- [ ] **Step 4: Run focused output checks**

Run:

```sh
cargo test -p surgeist-layout compute_output_defaults_to_no_scroll_geometry -- --nocapture
cargo test -p surgeist-layout node_output_defaults_to_no_scroll_geometry -- --nocapture
cargo fmt --check
git diff --check
```

Expected: all pass.

## Task 2: Add Scroll Geometry Construction Helpers

**Files:**

- Modify: `src/scroll.rs`
- Modify: `src/scroll_tests.rs`

- [ ] **Step 1: Add failing helper tests**

Append to `src/scroll_tests.rs`:

```rust
#[test]
fn scroll_geometry_from_layout_exposes_hidden_range_and_clip() {
    let scrollable_overflow = crate::scroll::scrollable_overflow_from_content_size(
        ScrollRect::new(Point::ZERO, Size::new(100.0, 40.0)).unwrap(),
        Size::new(140.0, 70.0),
    )
    .unwrap();
    let geometry = crate::scroll::scroll_geometry_from_layout(
        WritingMode::HorizontalTb,
        Direction::Ltr,
        Point::new(Overflow::Hidden, Overflow::Hidden),
        Size::new(100.0, 40.0),
        Edges::ZERO,
        Edges::ZERO,
        0.0,
        scrollable_overflow,
    )
    .unwrap();

    assert_eq!(geometry.scrollport(), ScrollRect::new(Point::ZERO, Size::new(100.0, 40.0)).unwrap());
    assert_eq!(geometry.overflow_clip(), Some(geometry.scrollport()));
    assert_eq!(geometry.scrollable_overflow(), ScrollRect::new(Point::ZERO, Size::new(140.0, 70.0)).unwrap());
    assert_eq!(geometry.range().maximum_offset(), Size::new(40.0, 30.0));
}

#[test]
fn scroll_geometry_from_layout_keeps_clip_range_zero() {
    let scrollable_overflow = crate::scroll::scrollable_overflow_from_content_size(
        ScrollRect::new(Point::ZERO, Size::new(100.0, 40.0)).unwrap(),
        Size::new(140.0, 70.0),
    )
    .unwrap();
    let geometry = crate::scroll::scroll_geometry_from_layout(
        WritingMode::HorizontalTb,
        Direction::Ltr,
        Point::new(Overflow::Clip, Overflow::Clip),
        Size::new(100.0, 40.0),
        Edges::ZERO,
        Edges::ZERO,
        0.0,
        scrollable_overflow,
    )
    .unwrap();

    assert_eq!(geometry.overflow_clip(), Some(geometry.scrollport()));
    assert_eq!(geometry.range().maximum_offset(), Size::ZERO);
}

#[test]
fn scroll_geometry_from_layout_keeps_visible_range_zero_with_visible_overflow() {
    let scrollable_overflow = crate::scroll::scrollable_overflow_from_content_size(
        ScrollRect::new(Point::ZERO, Size::new(100.0, 40.0)).unwrap(),
        Size::new(140.0, 70.0),
    )
    .unwrap();
    let geometry = crate::scroll::scroll_geometry_from_layout(
        WritingMode::HorizontalTb,
        Direction::Ltr,
        Point::new(Overflow::Visible, Overflow::Visible),
        Size::new(100.0, 40.0),
        Edges::ZERO,
        Edges::ZERO,
        0.0,
        scrollable_overflow,
    )
    .unwrap();

    assert_eq!(geometry.overflow_clip(), None);
    assert_eq!(geometry.scrollable_overflow(), ScrollRect::new(Point::ZERO, Size::new(140.0, 70.0)).unwrap());
    assert_eq!(geometry.range().maximum_offset(), Size::ZERO);
}

#[test]
fn scroll_geometry_from_layout_accounts_for_scrollbar_gutter() {
    let scrollable_overflow = crate::scroll::scrollable_overflow_from_content_size(
        ScrollRect::new(Point::new(10.0, 0.0), Size::new(90.0, 40.0)).unwrap(),
        Size::new(120.0, 40.0),
    )
    .unwrap();
    let geometry = crate::scroll::scroll_geometry_from_layout(
        WritingMode::HorizontalTb,
        Direction::Rtl,
        Point::new(Overflow::Hidden, Overflow::Scroll),
        Size::new(100.0, 40.0),
        Edges::ZERO,
        Edges::ZERO,
        10.0,
        scrollable_overflow,
    )
    .unwrap();

    assert_eq!(geometry.scrollport(), ScrollRect::new(Point::new(10.0, 0.0), Size::new(90.0, 40.0)).unwrap());
    assert_eq!(geometry.gutters().vertical(), Some(ScrollRect::new(Point::ZERO, Size::new(10.0, 40.0)).unwrap()));
    assert_eq!(geometry.range().maximum_offset(), Size::new(30.0, 0.0));
}

#[test]
fn scroll_geometry_from_layout_keeps_visible_axis_range_zero_when_other_axis_scrolls() {
    let scrollable_overflow = crate::scroll::scrollable_overflow_from_content_size(
        ScrollRect::new(Point::ZERO, Size::new(100.0, 40.0)).unwrap(),
        Size::new(140.0, 70.0),
    )
    .unwrap();
    let geometry = crate::scroll::scroll_geometry_from_layout(
        WritingMode::HorizontalTb,
        Direction::Ltr,
        Point::new(Overflow::Visible, Overflow::Hidden),
        Size::new(100.0, 40.0),
        Edges::ZERO,
        Edges::ZERO,
        0.0,
        scrollable_overflow,
    )
    .unwrap();

    assert_eq!(geometry.overflow_clip(), Some(geometry.scrollport()));
    assert_eq!(geometry.range().maximum_offset(), Size::new(0.0, 30.0));
}
```

Run:

```sh
cargo test -p surgeist-layout scroll_geometry_from_layout -- --nocapture
```

Expected: compile failure naming missing helper functions.

- [ ] **Step 2: Implement helper functions**

Add the helper code from **Shared Helper API To Add** to `src/scroll.rs`.

Remove `#[allow(dead_code)]` from Phase 2 helper items that are now called by
the new helpers. If `ScrollbarReservation` or `ScrollBoxRects` aliases remain
test-only, either leave the narrow allowance on the alias or move test imports
to the generic type name.

- [ ] **Step 3: Run focused helper checks**

Run:

```sh
cargo test -p surgeist-layout scroll_geometry_from_layout -- --nocapture
cargo test -p surgeist-layout scroll_geometry_core_is_scalar_generic -- --nocapture
cargo fmt --check
git diff --check
```

Expected: all pass.

## Task 3: Emit Root Scroll Geometry

**Files:**

- Modify: `src/compute.rs`
- Modify: `src/root_tests.rs`

- [ ] **Step 1: Add failing root tests**

Add focused tests to `src/root_tests.rs` near existing root layout tests:

```rust
#[test]
fn root_layout_emits_scroll_geometry_for_scroll_overflow() {
    let mut tree = SingleRootTree::new(NodeInput {
        overflow: Point::new(Overflow::Scroll, Overflow::Scroll),
        scrollbar_width: 10.0,
        size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
        ..NodeInput::default()
    });
    tree.output = ComputeOutput::from_sizes(Size::new(100.0, 40.0), Size::new(130.0, 70.0));

    compute_root(
        &mut tree,
        1,
        Size::new(Available::definite(100.0), Available::definite(40.0)),
    );

    let geometry = tree.layouts[&1].scroll_geometry.unwrap();
    assert_eq!(geometry.scrollport(), ScrollRect::new(Point::ZERO, Size::new(90.0, 30.0)).unwrap());
    assert_eq!(geometry.range().maximum_offset(), Size::new(40.0, 40.0));
    assert_eq!(
        geometry.range().clamp(ScrollOffset::new(Point::new(99.0, -5.0))),
        ScrollOffset::new(Point::new(40.0, 0.0))
    );
    assert_eq!(geometry.overflow_clip(), Some(geometry.scrollport()));
}

#[test]
fn root_layout_emits_visible_scroll_geometry_without_range() {
    let mut tree = SingleRootTree::new(NodeInput {
        overflow: Point::new(Overflow::Visible, Overflow::Visible),
        size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
        ..NodeInput::default()
    });
    tree.output = ComputeOutput::from_sizes(Size::new(100.0, 40.0), Size::new(130.0, 70.0));

    compute_root(
        &mut tree,
        1,
        Size::new(Available::definite(100.0), Available::definite(40.0)),
    );

    let geometry = tree.layouts[&1].scroll_geometry.unwrap();
    assert_eq!(geometry.overflow_clip(), None);
    assert_eq!(geometry.scrollable_overflow(), ScrollRect::new(Point::ZERO, Size::new(130.0, 70.0)).unwrap());
    assert_eq!(geometry.range().maximum_offset(), Size::ZERO);
}

#[test]
fn root_layout_emits_clip_geometry_without_range() {
    let mut tree = SingleRootTree::new(NodeInput {
        overflow: Point::new(Overflow::Clip, Overflow::Clip),
        size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
        ..NodeInput::default()
    });
    tree.output = ComputeOutput::from_sizes(Size::new(100.0, 40.0), Size::new(130.0, 70.0));

    compute_root(
        &mut tree,
        1,
        Size::new(Available::definite(100.0), Available::definite(40.0)),
    );

    let geometry = tree.layouts[&1].scroll_geometry.unwrap();
    assert_eq!(geometry.overflow_clip(), Some(geometry.scrollport()));
    assert_eq!(geometry.range().maximum_offset(), Size::ZERO);
}

#[test]
fn root_scroll_geometry_range_accounts_for_padding_border_and_gutter() {
    let mut tree = SingleRootTree::new(NodeInput {
        overflow: Point::new(Overflow::Hidden, Overflow::Scroll),
        scrollbar_width: 10.0,
        size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
        padding: Edges::all(Length::px(2.0)),
        border: Edges::all(Length::px(3.0)),
        ..NodeInput::default()
    });
    tree.output = ComputeOutput::from_sizes(Size::new(100.0, 40.0), Size::new(130.0, 70.0));

    compute_root(
        &mut tree,
        1,
        Size::new(Available::definite(100.0), Available::definite(40.0)),
    );

    let geometry = tree.layouts[&1].scroll_geometry.unwrap();
    assert_eq!(geometry.scrollport(), ScrollRect::new(Point::new(3.0, 3.0), Size::new(84.0, 34.0)).unwrap());
    assert_eq!(geometry.scrollable_overflow(), ScrollRect::new(Point::new(5.0, 5.0), Size::new(130.0, 70.0)).unwrap());
    assert_eq!(geometry.range().maximum_offset(), Size::new(48.0, 38.0));
    assert_eq!(
        geometry.range().clamp(ScrollOffset::new(Point::new(99.0, 99.0))),
        ScrollOffset::new(Point::new(48.0, 38.0))
    );
}

#[test]
fn root_scroll_geometry_preserves_child_origin_bearing_scrollable_overflow() {
    let mut tree = SingleRootTree::new(NodeInput {
        overflow: Point::new(Overflow::Hidden, Overflow::Hidden),
        size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
        ..NodeInput::default()
    });
    let child_overflow =
        ScrollRect::new(Point::new(-12.0, -4.0), Size::new(160.0, 74.0)).unwrap();
    let child_geometry = crate::scroll::scroll_geometry_from_layout(
        WritingMode::HorizontalTb,
        Direction::Ltr,
        Point::new(Overflow::Hidden, Overflow::Hidden),
        Size::new(100.0, 40.0),
        Edges::ZERO,
        Edges::ZERO,
        0.0,
        child_overflow,
    )
    .unwrap();
    tree.output = ComputeOutput::from_sizes(Size::new(100.0, 40.0), Size::new(130.0, 70.0));
    tree.output.scroll_geometry = Some(child_geometry);

    compute_root(
        &mut tree,
        1,
        Size::new(Available::definite(100.0), Available::definite(40.0)),
    );

    let geometry = tree.layouts[&1].scroll_geometry.unwrap();
    assert_eq!(geometry.scrollable_overflow(), child_overflow);
    assert_eq!(geometry.range().maximum_offset(), Size::new(48.0, 30.0));
}
```

Create this reusable helper in `src/root_tests.rs` near the root layout tests:

```rust
#[derive(Default)]
struct SingleRootTree {
    style: NodeInput,
    output: ComputeOutput,
    layouts: HashMap<u32, NodeOutput>,
    input: Option<ComputeInput>,
}

impl SingleRootTree {
    fn new(style: NodeInput) -> Self {
        Self {
            style,
            output: ComputeOutput::from_outer_size(Size::ZERO),
            layouts: HashMap::new(),
            input: None,
        }
    }
}

impl Traverse for SingleRootTree {
    type Node = u32;
    type Scalar = Scalar;
    type Children<'a> = std::iter::Empty<u32>;

    fn children(&self, _node: Self::Node) -> Self::Children<'_> {
        std::iter::empty()
    }

    fn child_count(&self, _node: Self::Node) -> usize {
        0
    }

    fn child(&self, _node: Self::Node, _index: usize) -> Self::Node {
        unreachable!("root test tree has no children")
    }
}

impl Compute for SingleRootTree {
    fn node_input(&self, _node: Self::Node) -> &NodeInput {
        &self.style
    }

    fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
        LayoutInputOf::box_input(self.node_input(node).clone())
    }

    fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
        self.layouts.insert(node, layout);
    }

    fn compute_child(&mut self, _node: Self::Node, input: ComputeInput) -> ComputeOutput {
        self.input = Some(input);
        self.output
    }
}
```

Run:

```sh
cargo test -p surgeist-layout root_layout_emits_scroll_geometry -- --nocapture
cargo test -p surgeist-layout root_layout_emits_clip_geometry_without_range -- --nocapture
cargo test -p surgeist-layout root_scroll_geometry_range_accounts_for_padding_border_and_gutter -- --nocapture
cargo test -p surgeist-layout root_scroll_geometry_preserves_child_origin_bearing_scrollable_overflow -- --nocapture
```

Expected: tests compile but fail because `compute_root` does not set
`scroll_geometry`.

- [ ] **Step 2: Emit root geometry**

In `src/compute.rs`, import:

```rust
use crate::scroll::{
    ScrollbarReservationOf, content_box_inset_with_scrollbar, scroll_geometry_from_layout,
    scrollable_overflow_from_layout_content_size, scrollbar_size_from_overflow,
};
```

In `compute_root`, after resolving `padding`, `border`, and `scrollbar_size`,
construct:

```rust
let scrollable_overflow = scrollable_overflow_from_layout_content_size(
    style.direction,
    style.overflow,
    output.size,
    padding,
    border,
    style.scrollbar_width,
    output.content_size,
)
.expect("root scrollable overflow is derived from finite non-negative layout output");
let scrollable_overflow = output
    .scroll_geometry
    .map(|geometry| {
        crate::scroll::scroll_rect_union(scrollable_overflow, geometry.scrollable_overflow())
            .expect("root scrollable overflow union remains valid")
    })
    .unwrap_or(scrollable_overflow);
let scroll_geometry = Some(
    scroll_geometry_from_layout(
        style.writing_mode,
        style.direction,
        style.overflow,
        output.size,
        padding,
        border,
        style.scrollbar_width,
        scrollable_overflow,
    )
    .expect("root scroll geometry is derived from finite non-negative layout output"),
);
```

Set this field in the root `NodeOutputOf` literal:

```rust
scroll_geometry,
```

- [ ] **Step 3: Run root checks**

Run:

```sh
cargo test -p surgeist-layout root_layout_emits_scroll_geometry -- --nocapture
cargo test -p surgeist-layout root_layout_emits_clip_geometry_without_range -- --nocapture
cargo test -p surgeist-layout root_scroll_geometry_range_accounts_for_padding_border_and_gutter -- --nocapture
cargo test -p surgeist-layout root_scroll_geometry_preserves_child_origin_bearing_scrollable_overflow -- --nocapture
cargo test -p surgeist-layout root_layout_stores_child_output_as_root_layout -- --nocapture
cargo fmt --check
git diff --check
```

Expected: all pass.

## Task 4: Round Scroll Geometry With Final NodeOutput

**Files:**

- Modify: `src/scroll.rs`
- Modify: `src/scroll_tests.rs`
- Modify: `src/compute.rs`
- Modify: `src/root_tests.rs`

- [ ] **Step 1: Add failing rounding tests**

Append to `src/scroll_tests.rs`:

```rust
#[test]
fn round_scroll_geometry_rounds_rects_with_cumulative_origin() {
    let scrollable_overflow =
        ScrollRect::new(Point::new(0.25, 0.25), Size::new(10.5, 20.5)).unwrap();
    let geometry = ScrollGeometry::new(
        WritingMode::HorizontalTb,
        Direction::Ltr,
        ScrollContainerFacts::new(
            ScrollContainerAxis::from_overflow(Overflow::Hidden).unwrap(),
            ScrollContainerAxis::from_overflow(Overflow::Hidden).unwrap(),
        ),
        ScrollRect::new(Point::new(0.25, 0.25), Size::new(5.5, 6.5)).unwrap(),
        Some(ScrollRect::new(Point::new(0.25, 0.25), Size::new(5.5, 6.5)).unwrap()),
        scrollable_overflow,
        ScrollRange::new(Size::new(5.0, 14.0)).unwrap(),
        ScrollbarGutterRects::new(
            None,
            Some(ScrollRect::new(Point::new(5.75, 0.25), Size::new(1.0, 6.5)).unwrap()),
        ),
    )
    .unwrap();

    let rounded =
        crate::scroll::round_scroll_geometry(geometry, Point::new(10.25, 20.25)).unwrap();

    assert_eq!(rounded.scrollport(), ScrollRect::new(Point::ZERO, Size::new(6.0, 7.0)).unwrap());
    assert_eq!(rounded.overflow_clip(), Some(rounded.scrollport()));
    assert_eq!(
        rounded.scrollable_overflow(),
        ScrollRect::new(Point::ZERO, Size::new(11.0, 21.0)).unwrap()
    );
    assert_eq!(
        rounded.gutters().vertical(),
        Some(ScrollRect::new(Point::new(6.0, 0.0), Size::new(1.0, 7.0)).unwrap())
    );
    assert_eq!(rounded.range().maximum_offset(), Size::new(5.0, 14.0));
}
```

Add this root integration test to `src/root_tests.rs`:

```rust
#[test]
fn round_layout_rounds_scroll_geometry_with_node_output() {
    let mut tree = OracleTreeOf::<f64>::new().unrounded(
        0,
        NodeOutputOf::<f64> {
            location: Point::new(10.25, 20.25),
            size: Size::new(100.5, 40.5),
            content_size: Size::new(120.5, 70.5),
            scroll_geometry: Some(
                crate::scroll::scroll_geometry_from_layout(
                    WritingMode::HorizontalTb,
                    Direction::Ltr,
                    Point::new(Overflow::Hidden, Overflow::Hidden),
                    Size::new(100.5, 40.5),
                    Edges::ZERO,
                    Edges::ZERO,
                    0.0,
                    ScrollRectOf::new(Point::ZERO, Size::new(120.5, 70.5)).unwrap(),
                )
                .unwrap(),
            ),
            ..NodeOutputOf::<f64>::default()
        },
    );

    round_layout(&mut tree, 0);

    let geometry = tree.output(0).scroll_geometry.unwrap();
    assert_eq!(geometry.scrollport().size(), Size::new(101.0, 41.0));
    assert_eq!(geometry.scrollable_overflow().size(), Size::new(121.0, 71.0));
    assert_eq!(geometry.range().maximum_offset(), Size::new(20.0, 30.0));
}
```

Run:

```sh
cargo test -p surgeist-layout round_scroll_geometry -- --nocapture
cargo test -p surgeist-layout round_layout_rounds_scroll_geometry_with_node_output -- --nocapture
```

Expected: compile failure naming missing `round_scroll_geometry`, then failing
integration behavior until `round_layout` calls it.

- [ ] **Step 2: Implement rounding helper**

Add `round_scroll_geometry`, `round_scroll_rect`, and private `round` from
**Shared Helper API To Add** to `src/scroll.rs`.

- [ ] **Step 3: Wire `round_layout`**

In `src/compute.rs`, import:

```rust
use crate::scroll::{
    ScrollbarReservationOf, content_box_inset_with_scrollbar, round_scroll_geometry,
    scroll_geometry_from_layout, scrollable_overflow_from_layout_content_size,
    scrollbar_size_from_overflow,
};
```

In `round_layout_inner`, after rounding `layout.padding.bottom` and before
`tree.set_final(node, layout)`, add:

```rust
layout.scroll_geometry = unrounded
    .scroll_geometry
    .map(|geometry| round_scroll_geometry(geometry, Point::new(cumulative_x, cumulative_y)))
    .transpose()
    .expect("rounded scroll geometry remains finite and non-negative");
```

- [ ] **Step 4: Run focused rounding checks**

Run:

```sh
cargo test -p surgeist-layout round_scroll_geometry -- --nocapture
cargo test -p surgeist-layout round_layout_rounds_scroll_geometry_with_node_output -- --nocapture
cargo test -p surgeist-layout f64_round_layout_preserves_large_coordinates -- --nocapture
cargo fmt --check
git diff --check
```

Expected: all pass.

## Task 5: Emit Block Scroll Geometry Through ComputeOutput

**Files:**

- Modify: `src/block.rs`
- Modify: `src/block_tests.rs`

- [ ] **Step 1: Add failing block compute-output tests**

Add tests to `src/block_tests.rs`:

```rust
#[test]
fn block_layout_emits_scroll_geometry_for_scroll_overflow() {
    #[derive(Default)]
    struct BlockTree {
        style: NodeInput,
    }

    impl Traverse for BlockTree {
        type Node = u32;
        type Scalar = Scalar;
        type Children<'a> = std::iter::Empty<u32>;

        fn children(&self, _node: Self::Node) -> Self::Children<'_> {
            std::iter::empty()
        }

        fn child_count(&self, _node: Self::Node) -> usize {
            0
        }

        fn child(&self, _node: Self::Node, _index: usize) -> Self::Node {
            unreachable!()
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, _node: Self::Node) -> &NodeInput {
            &self.style
        }

        fn layout_input(&self, _node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.style.clone())
        }

        fn set_unrounded(&mut self, _node: Self::Node, _layout: NodeOutput) {}

        fn compute_child(&mut self, _node: Self::Node, _input: ComputeInput) -> ComputeOutput {
            unreachable!()
        }
    }

    let mut tree = BlockTree {
        style: NodeInput {
            overflow: Point::new(Overflow::Scroll, Overflow::Hidden),
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            ..NodeInput::default()
        },
    };

    let output = compute_block(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(100.0), Some(40.0)),
            available: Size::new(Available::definite(100.0), Available::definite(40.0)),
        },
    );

    let geometry = output.scroll_geometry.unwrap();
    assert_eq!(geometry.overflow_clip(), Some(geometry.scrollport()));
    assert_eq!(geometry.range().maximum_offset(), Size::ZERO);
}
```

Add a second test using one visible overflowing child so block scrollable
overflow is non-zero:

```rust
#[test]
fn block_scroll_geometry_uses_visible_child_overflow_content_size() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;
        type Scalar = Scalar;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, _input: ComputeInput) -> ComputeOutput {
            self.outputs[&node]
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            overflow: Point::new(Overflow::Hidden, Overflow::Hidden),
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            overflow: Point::new(Overflow::Visible, Overflow::Visible),
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(50.0, 20.0), Size::new(130.0, 70.0)),
    );

    let output = compute_block(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(100.0), Some(40.0)),
            available: Size::new(Available::definite(100.0), Available::definite(40.0)),
        },
    );

    let geometry = output.scroll_geometry.unwrap();
    assert_eq!(geometry.scrollable_overflow(), ScrollRect::new(Point::ZERO, Size::new(130.0, 70.0)).unwrap());
    assert_eq!(geometry.range().maximum_offset(), Size::new(30.0, 30.0));
}
```

Add a third test that proves the scrollable overflow rect preserves a negative
origin:

```rust
#[test]
fn block_scroll_geometry_preserves_negative_child_overflow_origin() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;
        type Scalar = Scalar;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, _input: ComputeInput) -> ComputeOutput {
            self.outputs[&node]
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            overflow: Point::new(Overflow::Hidden, Overflow::Hidden),
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            overflow: Point::new(Overflow::Visible, Overflow::Visible),
            inset: Edges {
                left: LengthAuto::px(-20.0),
                top: LengthAuto::px(-5.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            position: Position::Relative,
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(50.0, 20.0), Size::new(50.0, 20.0)),
    );

    let output = compute_block(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(100.0), Some(40.0)),
            available: Size::new(Available::definite(100.0), Available::definite(40.0)),
        },
    );

    let geometry = output.scroll_geometry.unwrap();
    assert_eq!(geometry.scrollable_overflow().origin(), Point::new(-20.0, -5.0));
    assert_eq!(geometry.scrollable_overflow().size(), Size::new(120.0, 45.0));
    assert_eq!(geometry.range().maximum_offset(), Size::ZERO);
}
```

Add block keyword and coordinate tests:

```rust
#[test]
fn block_scroll_geometry_distinguishes_visible_hidden_clip_and_scroll() {
    fn run(overflow: Point<Overflow>) -> ScrollGeometry {
        #[derive(Default)]
        struct BlockTree {
            style: NodeInput,
        }

        impl Traverse for BlockTree {
            type Node = u32;
            type Scalar = Scalar;
            type Children<'a> = std::iter::Empty<u32>;

            fn children(&self, _node: Self::Node) -> Self::Children<'_> {
                std::iter::empty()
            }

            fn child_count(&self, _node: Self::Node) -> usize {
                0
            }

            fn child(&self, _node: Self::Node, _index: usize) -> Self::Node {
                unreachable!()
            }
        }

        impl Compute for BlockTree {
            fn node_input(&self, _node: Self::Node) -> &NodeInput {
                &self.style
            }

            fn layout_input(&self, _node: Self::Node) -> LayoutInputOf<Self::Scalar> {
                LayoutInputOf::box_input(self.style.clone())
            }

            fn set_unrounded(&mut self, _node: Self::Node, _layout: NodeOutput) {}

            fn compute_child(&mut self, _node: Self::Node, _input: ComputeInput) -> ComputeOutput {
                unreachable!()
            }
        }

        let mut tree = BlockTree {
            style: NodeInput {
                display: Display::Block,
                overflow,
                size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
                ..NodeInput::default()
            },
        };

        compute_block(
            &mut tree,
            1,
            ComputeInput {
                run_mode: RunMode::PerformLayout,
                sizing_mode: SizingMode::InherentSize,
                axis: RequestedAxis::Both,
                known: Size::NONE,
                parent: Size::new(Some(100.0), Some(40.0)),
                available: Size::new(Available::definite(100.0), Available::definite(40.0)),
            },
        )
        .scroll_geometry
        .unwrap()
    }

    let visible = run(Point::new(Overflow::Visible, Overflow::Visible));
    assert_eq!(visible.overflow_clip(), None);
    assert_eq!(visible.range().maximum_offset(), Size::ZERO);

    let hidden = run(Point::new(Overflow::Hidden, Overflow::Hidden));
    assert_eq!(hidden.overflow_clip(), Some(hidden.scrollport()));
    assert_eq!(
        hidden
            .range()
            .clamp(ScrollOffset::new(Point::new(3.0, 4.0))),
        ScrollOffset::new(Point::ZERO)
    );

    let clip = run(Point::new(Overflow::Clip, Overflow::Clip));
    assert_eq!(clip.overflow_clip(), Some(clip.scrollport()));
    assert_eq!(clip.range().maximum_offset(), Size::ZERO);

    let scroll = run(Point::new(Overflow::Scroll, Overflow::Scroll));
    assert_eq!(scroll.overflow_clip(), Some(scroll.scrollport()));
    assert_eq!(scroll.range().maximum_offset(), Size::ZERO);
}

#[test]
fn block_scroll_geometry_uses_node_local_padding_border_and_gutter_coordinates() {
    #[derive(Default)]
    struct BlockTree {
        style: NodeInput,
    }

    impl Traverse for BlockTree {
        type Node = u32;
        type Scalar = Scalar;
        type Children<'a> = std::iter::Empty<u32>;

        fn children(&self, _node: Self::Node) -> Self::Children<'_> {
            std::iter::empty()
        }

        fn child_count(&self, _node: Self::Node) -> usize {
            0
        }

        fn child(&self, _node: Self::Node, _index: usize) -> Self::Node {
            unreachable!()
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, _node: Self::Node) -> &NodeInput {
            &self.style
        }

        fn layout_input(&self, _node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.style.clone())
        }

        fn set_unrounded(&mut self, _node: Self::Node, _layout: NodeOutput) {}

        fn compute_child(&mut self, _node: Self::Node, _input: ComputeInput) -> ComputeOutput {
            unreachable!()
        }
    }

    let mut tree = BlockTree {
        style: NodeInput {
            display: Display::Block,
            direction: Direction::Rtl,
            overflow: Point::new(Overflow::Visible, Overflow::Scroll),
            scrollbar_width: 10.0,
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            padding: Edges::all(Length::px(2.0)),
            border: Edges::all(Length::px(3.0)),
            ..NodeInput::default()
        },
    };

    let output = compute_block(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(100.0), Some(40.0)),
            available: Size::new(Available::definite(100.0), Available::definite(40.0)),
        },
    );

    let geometry = output.scroll_geometry.unwrap();
    assert_eq!(geometry.scrollport().origin(), Point::new(13.0, 3.0));
    assert_eq!(geometry.scrollport().size(), Size::new(84.0, 34.0));
    assert_eq!(
        geometry.gutters().vertical(),
        Some(ScrollRect::new(Point::new(3.0, 3.0), Size::new(10.0, 34.0)).unwrap())
    );
}
```

Add an absolute child overflow test:

```rust
#[test]
fn block_scroll_geometry_includes_absolute_child_overflow_rect() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;
        type Scalar = Scalar;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, _input: ComputeInput) -> ComputeOutput {
            self.outputs[&node]
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            overflow: Point::new(Overflow::Hidden, Overflow::Hidden),
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            position: Position::Absolute,
            overflow: Point::new(Overflow::Visible, Overflow::Visible),
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
            inset: Edges {
                left: LengthAuto::px(90.0),
                top: LengthAuto::px(35.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(45.0, 25.0)),
    );

    let output = compute_block(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(100.0), Some(40.0)),
            available: Size::new(Available::definite(100.0), Available::definite(40.0)),
        },
    );

    let geometry = output.scroll_geometry.unwrap();
    assert_eq!(geometry.scrollable_overflow().size(), Size::new(135.0, 60.0));
    assert_eq!(geometry.range().maximum_offset(), Size::new(35.0, 20.0));
}
```

Add focused tests for the newly origin-bearing inline and float paths:

- `block_scroll_geometry_includes_final_content_box_after_size_resolution`:
  build a block whose final `output_size` is larger than the child-flow
  accumulator's initial inner size because of known/min sizing. Assert
  `scrollable_overflow` includes the final Phase 2 content box derived from
  final `output_size`, padding, border, direction, overflow, and scrollbar
  gutter.
- `block_scroll_geometry_includes_inline_child_origin_bearing_overflow_rect`:
  build a block with an inline-level child whose `ComputeOutput.scroll_geometry`
  has `scrollable_overflow` origin `Point::new(-12.0, -3.0)` and size larger
  than the final inline item. Assert the parent block `scrollable_overflow`
  includes the translated origin-bearing child rect, and assert a `Visible`
  parent axis still exposes no range even when that axis overflows.
- `block_scroll_geometry_includes_segmented_inline_overflow_rects`: build an
  inline run that is split by clear/float handling and assert the parent block
  unions each segment placement's node-local `scrollable_overflow` with its
  segment y translation.
- `block_scroll_geometry_includes_float_child_overflow_rect`: build a block
  with one floated block child whose computed scrollable overflow extends past
  the float border box. Assert the parent block `scrollable_overflow` includes
  the translated float overflow, not only the float margin box.
- `block_scroll_geometry_includes_absolute_margin_box_with_area_offset`: build a
  positioned block with non-zero border/padding/gutter and an absolute child
  with non-zero margin. Assert the block scrollable overflow uses node-local
  scroll geometry for the child margin box and translated child overflow, while
  any legacy `content_size` area-offset subtraction remains isolated to the
  legacy size summary.

Run:

```sh
cargo test -p surgeist-layout block_layout_emits_scroll_geometry -- --nocapture
cargo test -p surgeist-layout block_scroll_geometry_uses_visible_child_overflow_content_size -- --nocapture
cargo test -p surgeist-layout block_scroll_geometry_preserves_negative_child_overflow_origin -- --nocapture
cargo test -p surgeist-layout block_scroll_geometry_distinguishes_visible_hidden_clip_and_scroll -- --nocapture
cargo test -p surgeist-layout block_scroll_geometry_uses_node_local_padding_border_and_gutter_coordinates -- --nocapture
cargo test -p surgeist-layout block_scroll_geometry_includes_absolute_child_overflow_rect -- --nocapture
cargo test -p surgeist-layout block_scroll_geometry_includes_final_content_box_after_size_resolution -- --nocapture
cargo test -p surgeist-layout block_scroll_geometry_includes_inline_child_origin_bearing_overflow_rect -- --nocapture
cargo test -p surgeist-layout block_scroll_geometry_includes_segmented_inline_overflow_rects -- --nocapture
cargo test -p surgeist-layout block_scroll_geometry_includes_float_child_overflow_rect -- --nocapture
cargo test -p surgeist-layout block_scroll_geometry_includes_absolute_margin_box_with_area_offset -- --nocapture
```

Expected: tests compile but fail because `compute_block` does not set
`ComputeOutput.scroll_geometry`.

- [ ] **Step 2: Add block helper**

In `src/block.rs`, extend the `crate::scroll` import:

```rust
use crate::scroll::{
    ScrollbarReservationOf, content_box_inset_with_scrollbar, scroll_geometry_from_layout,
    scrollbar_size_from_overflow,
};
```

Add a private helper near `content_size_contribution`:

```rust
fn block_scroll_geometry<S: LayoutScalar>(
    style: &NodeInputOf<S>,
    constants: &Constants<S>,
    output_size: Size<S>,
    scrollable_overflow: super::ScrollRectOf<S>,
) -> super::ScrollGeometryOf<S> {
    scroll_geometry_from_layout(
        style.writing_mode,
        style.direction,
        style.overflow,
        output_size,
        constants.padding,
        constants.border,
        style.scrollbar_width,
        scrollable_overflow,
    )
    .expect("block scroll geometry is derived from finite non-negative layout output")
}
```

If `Constants<S>` does not currently retain `padding`, add a `padding: Edges<S>`
field and initialize it in `Constants::new`.

- [ ] **Step 3: Add block scrollable-overflow accumulator**

Add a private accumulator near `content_size_contribution`:

```rust
#[derive(Clone, Copy, Debug)]
struct ScrollableOverflowAccumulator<S: LayoutScalar> {
    rect: super::ScrollRectOf<S>,
}

impl<S: LayoutScalar> ScrollableOverflowAccumulator<S> {
    fn new(content_box_origin: Point<S>, content_box_size: Size<S>) -> Self {
        Self {
            rect: super::ScrollRectOf::new(content_box_origin, content_box_size)
                .expect("content box size is non-negative"),
        }
    }

    fn include_rect(&mut self, rect: super::ScrollRectOf<S>) {
        self.rect = crate::scroll::scroll_rect_union(self.rect, rect)
            .expect("scrollable overflow union remains valid");
    }

    fn include_translated_child_overflow(
        &mut self,
        location: Point<S>,
        overflow: super::ScrollRectOf<S>,
    ) {
        self.include_rect(translate_scroll_rect(overflow, location));
    }

    fn include_child(
        &mut self,
        location: Point<S>,
        size: Size<S>,
        content_size: Size<S>,
        margin: Edges<S>,
        overflow: Point<Overflow>,
    ) {
        let margin_rect = super::ScrollRectOf::new(
            Point::new(location.x - margin.left, location.y - margin.top),
            size + margin.sum_axes(),
        )
        .expect("child margin rect is non-negative");
        self.include_rect(margin_rect);

        let visible_size = Size::new(
            if overflow.x == Overflow::Visible {
                size.width.max(content_size.width)
            } else {
                size.width
            },
            if overflow.y == Overflow::Visible {
                size.height.max(content_size.height)
            } else {
                size.height
            },
        );
        self.include_rect(
            super::ScrollRectOf::new(location, visible_size)
                .expect("visible overflow rect is non-negative"),
        );
    }

    fn finish(self) -> super::ScrollRectOf<S> {
        self.rect
    }
}
```

Add these private helpers near `block_scroll_geometry`; Task 6 reuses them for
child `NodeOutput` geometry:

```rust
fn child_scrollable_overflow<S: LayoutScalar>(
    style: &NodeInputOf<S>,
    size: Size<S>,
    content_size: Size<S>,
    padding: Edges<S>,
    border: Edges<S>,
    child_compute_geometry: Option<super::ScrollGeometryOf<S>>,
) -> super::ScrollRectOf<S> {
    let base = crate::scroll::scrollable_overflow_from_layout_content_size(
        style.direction,
        style.overflow,
        size,
        padding,
        border,
        style.scrollbar_width,
        content_size,
    )
    .expect("child scrollable overflow is derived from finite non-negative layout output");
    let Some(child_compute_geometry) = child_compute_geometry else {
        return base;
    };

    crate::scroll::scroll_rect_union(base, child_compute_geometry.scrollable_overflow())
        .expect("child scrollable overflow union remains valid")
}

fn translate_scroll_rect<S: LayoutScalar>(
    rect: super::ScrollRectOf<S>,
    offset: Point<S>,
) -> super::ScrollRectOf<S> {
    super::ScrollRectOf::new(
        Point::new(rect.origin().x + offset.x, rect.origin().y + offset.y),
        rect.size(),
    )
    .expect("translated scroll rect remains valid")
}

fn final_content_box_scroll_rect<S: LayoutScalar>(
    style: &NodeInputOf<S>,
    size: Size<S>,
    padding: Edges<S>,
    border: Edges<S>,
) -> super::ScrollRectOf<S> {
    let reservation =
        ScrollbarReservationOf::from_overflow(style.overflow, style.scrollbar_width, style.direction);
    let inset = content_box_inset_with_scrollbar(
        padding,
        border,
        reservation,
    );
    let content_size = Size::new(
        (size.width - inset.horizontal_sum()).max(S::ZERO),
        (size.height - inset.vertical_sum()).max(S::ZERO),
    );
    super::ScrollRectOf::new(Point::new(inset.left, inset.top), content_size)
        .expect("final content box scroll rect is valid")
}
```

Add this field to `InFlowResult<Node, S>`:

```rust
scrollable_overflow: super::ScrollRectOf<S>,
```

Add this field to `InlineRunPlacement<Node, S>`:

```rust
scrollable_overflow: super::ScrollRectOf<S>,
```

The inline placement field is origin-bearing in the placement caller's
coordinate space, not a plain size. For `layout_inline_run_children`, the
returned rect is in the parent block's node-local coordinates. For
`layout_inline_segments`, the returned rect is in the same coordinate space as
the segmented placement returned to `layout_inline_run_with_clear`.

Add this result type near `layout_absolute_children`:

```rust
struct AbsoluteLayoutResult<S: LayoutScalar> {
    content_size: Size<S>,
    scrollable_overflow: Option<super::ScrollRectOf<S>>,
}
```

In `layout_in_flow_children`, initialize:

```rust
let content_box_size = Size::new(
    inner_width
        .or(constants.node_inner_size.width)
        .or(input.available.width.into_option())
        .unwrap_or(S::ZERO),
    constants.node_inner_size.height.unwrap_or(S::ZERO),
);
let content_box_origin = Point::new(
    constants.content_box_inset.left,
    constants.content_box_inset.top,
);
let mut scrollable_overflow =
    ScrollableOverflowAccumulator::new(content_box_origin, content_box_size);
```

For every existing place that updates `content_size` from an origin-bearing
child/run contribution, also call `scrollable_overflow.include_child(...)` or
`scrollable_overflow.include_rect(...)` with the node-local origin used for
actual `NodeOutput.location`. For normal in-flow children, use:

```rust
scrollable_overflow.include_child(
    location,
    output.size,
    output.content_size,
    child_margin,
    child_style.overflow,
);
let child_overflow = child_scrollable_overflow(
    &child_style,
    output.size,
    output.content_size,
    child_padding,
    child_border,
    output.scroll_geometry,
);
scrollable_overflow.include_translated_child_overflow(location, child_overflow);
```

In `layout_inline_run_children`, after `report_items_by_order` is available,
build a run-local `ScrollableOverflowAccumulator` initialized with
`Point::ZERO, report.size`. For each `InlineRunChild::Box`, compute the child
location in run-local coordinates:

```rust
let run_child_location =
    Point::new(item.location.x + inset_offset.x, item.location.y + inset_offset.y);
let child_overflow = child_scrollable_overflow(
    child_style,
    item.size,
    item.content_size,
    item.padding,
    item.border,
    output.scroll_geometry,
);
run_scrollable_overflow.include_translated_child_overflow(
    run_child_location,
    child_overflow,
);
```

Translate the run-local rect before returning `InlineRunPlacement`, using the
same node-local origin used for actual child placement. Store the translated
rect in `placement.scrollable_overflow`:

```rust
scrollable_overflow: translate_scroll_rect(
    run_scrollable_overflow.finish(),
    Point::new(constants.content_box_inset.left + run_offset, cursor_y),
),
```

In `layout_inline_segments`, add a segment-level
`Option<super::ScrollRectOf<S>>`. After each segment placement, union
`placement.scrollable_overflow` into that option; each segment placement is
already node-local because `layout_inline_run_children` received that segment's
`cursor_y`. Return the union from `layout_inline_segments`; if a segmented run
contains no boxes, return `ScrollRectOf::new(Point::new(constants.content_box_inset.left,
start_y), Size::ZERO).unwrap()`.

When an `InlineRunPlacement` contributes to `layout_in_flow_children`, union
`placement.scrollable_overflow` directly into the parent accumulator:

```rust
scrollable_overflow.include_rect(placement.scrollable_overflow);
```

For floats and absolute children, include the child margin box and translated
child scrollable overflow in node-local coordinates. Extend `PendingFloat` with:

```rust
style: Box<NodeInputOf<S>>,
scrollable_overflow: super::ScrollRectOf<S>,
```

When creating a pending float, compute `scrollable_overflow` with
`child_scrollable_overflow(&child_style, output.size, output.content_size,
child_padding, child_border, output.scroll_geometry)`. After
`float_exclusions.place_float(...)` returns the final node-local border-box
location, include:

```rust
scrollable_overflow.include_child(
    float_location,
    pending_float.size,
    pending_float.content_size,
    pending_float.margin,
    pending_float.style.overflow,
);
scrollable_overflow.include_translated_child_overflow(
    float_location,
    pending_float.scrollable_overflow,
);
```

For absolute children, `location` is the node-local border-box origin because
`AbsoluteAxis::location()` already received `area_offset` as `area_start`.
Therefore scroll geometry must use `location` directly and must not subtract
`area_offset` again. The legacy `content_size` calculation may keep its current
`location - area_offset` call because that field remains the existing
content-size summary, but `absolute_scrollable_overflow` is node-local. The
absolute margin-box origin for scroll geometry is
`Point::new(location.x - margin.left, location.y - margin.top)`.

Change `layout_absolute_children` to return `AbsoluteLayoutResult<S>` instead
of `Size<S>`. Initialize:

```rust
let mut absolute_content_size = Size::ZERO;
let mut absolute_scrollable_overflow: Option<super::ScrollRectOf<S>> = None;
```

For each absolute child, after computing `location`, `final_size`, `margin`, and
`output`, build a node-local absolute scroll rect:

```rust
let margin_box_origin = Point::new(location.x - margin.left, location.y - margin.top);
let mut absolute_accumulator =
    ScrollableOverflowAccumulator::new(margin_box_origin, final_size + margin.sum_axes());
absolute_accumulator.include_child(
    location,
    final_size,
    output.content_size,
    margin,
    style.overflow,
);
let child_overflow = child_scrollable_overflow(
    &style,
    final_size,
    output.content_size,
    padding,
    border,
    output.scroll_geometry,
);
absolute_accumulator.include_translated_child_overflow(location, child_overflow);
let child_rect = absolute_accumulator.finish();
absolute_scrollable_overflow = Some(match absolute_scrollable_overflow {
    Some(existing) => crate::scroll::scroll_rect_union(existing, child_rect)
        .expect("absolute scroll overflow union remains valid"),
    None => child_rect,
});
```

Return:

```rust
AbsoluteLayoutResult {
    content_size: absolute_content_size,
    scrollable_overflow: absolute_scrollable_overflow,
}
```

Return it from `layout_in_flow_children`:

```rust
scrollable_overflow: scrollable_overflow.finish(),
```

- [ ] **Step 4: Set block compute-output geometry**

In `compute_block_inner`, after each final `ComputeOutputOf` is constructed for
non-early-return paths, union final-pass and absolute scroll overflow before
setting geometry. Replace the existing `layout_absolute_children` call with:

```rust
let absolute = layout_absolute_children(
    tree,
    &children,
    &final_pass.static_positions,
    output_size,
    &constants,
);
let content_size = max_content_size(final_pass.content_size, absolute.content_size);
let scrollable_overflow = crate::scroll::scroll_rect_union(
    final_pass.scrollable_overflow,
    final_content_box_scroll_rect(&style, output_size, constants.padding, constants.border),
)
.expect("block scrollable overflow includes final content box");
let scrollable_overflow = match absolute.scrollable_overflow {
    Some(absolute) => crate::scroll::scroll_rect_union(scrollable_overflow, absolute)
        .expect("block scrollable overflow union remains valid"),
    None => scrollable_overflow,
};
output.scroll_geometry = Some(block_scroll_geometry(
    &style,
    &constants,
    output_size,
    scrollable_overflow,
));
```

For `ComputeSize` outputs, do not set scroll geometry unless the reviewer
explicitly requires it. Phase 3 output is layout geometry, and `ComputeSize`
does not represent final layout placement.

For early `ComputeSize` returns, keep constructor defaults (`None`).

- [ ] **Step 5: Run block checks**

Run:

```sh
cargo test -p surgeist-layout block_layout_emits_scroll_geometry -- --nocapture
cargo test -p surgeist-layout block_scroll_geometry_uses_visible_child_overflow_content_size -- --nocapture
cargo test -p surgeist-layout block_scroll_geometry_preserves_negative_child_overflow_origin -- --nocapture
cargo test -p surgeist-layout block_scroll_geometry_distinguishes_visible_hidden_clip_and_scroll -- --nocapture
cargo test -p surgeist-layout block_scroll_geometry_uses_node_local_padding_border_and_gutter_coordinates -- --nocapture
cargo test -p surgeist-layout block_scroll_geometry_includes_absolute_child_overflow_rect -- --nocapture
cargo test -p surgeist-layout block_scroll_geometry_includes_final_content_box_after_size_resolution -- --nocapture
cargo test -p surgeist-layout block_scroll_geometry_includes_inline_child_origin_bearing_overflow_rect -- --nocapture
cargo test -p surgeist-layout block_scroll_geometry_includes_segmented_inline_overflow_rects -- --nocapture
cargo test -p surgeist-layout block_scroll_geometry_includes_float_child_overflow_rect -- --nocapture
cargo test -p surgeist-layout block_scroll_geometry_includes_absolute_margin_box_with_area_offset -- --nocapture
cargo test -p surgeist-layout block_content_size_includes_visible_child_overflow_content -- --nocapture
cargo test -p surgeist-layout block_rtl_scrollbar_gutter_uses_left_inset -- --nocapture
cargo fmt --check
git diff --check
```

Expected: all pass.

## Task 6: Emit Block Child NodeOutput Scroll Geometry

**Files:**

- Modify: `src/block.rs`
- Modify: `src/block_tests.rs`

- [ ] **Step 1: Add failing child-output tests**

Add tests to `src/block_tests.rs`:

```rust
#[test]
fn block_child_node_output_recomputes_child_scroll_geometry() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;
        type Scalar = Scalar;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, _input: ComputeInput) -> ComputeOutput {
            self.outputs[&node]
        }
    }

    let mut child_output =
        ComputeOutput::from_sizes(Size::new(50.0, 20.0), Size::new(80.0, 45.0));
    child_output.scroll_geometry = None;

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            overflow: Point::new(Overflow::Hidden, Overflow::Hidden),
            size: Size::new(Dimension::px(50.0), Dimension::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(2, child_output);

    compute_block(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(100.0), Some(40.0)),
            available: Size::new(Available::definite(100.0), Available::definite(40.0)),
        },
    );

    let geometry = tree.layouts[&2].scroll_geometry.unwrap();
    assert_eq!(geometry.scrollport().size(), Size::new(50.0, 20.0));
    assert_eq!(geometry.range().maximum_offset(), Size::new(30.0, 25.0));
}
```

Add an absolute-child test that proves final-size recomputation:

```rust
#[test]
fn block_absolute_child_scroll_geometry_uses_final_node_output_size() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;
        type Scalar = Scalar;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, _input: ComputeInput) -> ComputeOutput {
            self.outputs[&node]
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            position: Position::Absolute,
            overflow: Point::new(Overflow::Hidden, Overflow::Hidden),
            inset: Edges {
                left: LengthAuto::px(0.0),
                right: LengthAuto::px(0.0),
                top: LengthAuto::px(0.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(120.0, 30.0)),
    );

    compute_block(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(100.0), Some(40.0)),
            available: Size::new(Available::definite(100.0), Available::definite(40.0)),
        },
    );

    let child_layout = tree.layouts[&2];
    assert_eq!(child_layout.size.width, 100.0);
    let geometry = child_layout.scroll_geometry.unwrap();
    assert_eq!(geometry.scrollport().size().width, 100.0);
    assert_eq!(geometry.range().maximum_offset(), Size::new(20.0, 20.0));
}
```

Add a child-origin preservation test:

```rust
#[test]
fn block_child_node_output_preserves_child_scrollable_overflow_origin() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;
        type Scalar = Scalar;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, _input: ComputeInput) -> ComputeOutput {
            self.outputs[&node]
        }
    }

    let child_overflow =
        ScrollRect::new(Point::new(-15.0, -4.0), Size::new(95.0, 49.0)).unwrap();
    let child_geometry = crate::scroll::scroll_geometry_from_layout(
        WritingMode::HorizontalTb,
        Direction::Ltr,
        Point::new(Overflow::Hidden, Overflow::Hidden),
        Size::new(50.0, 20.0),
        Edges::ZERO,
        Edges::ZERO,
        0.0,
        child_overflow,
    )
    .unwrap();
    let mut child_output =
        ComputeOutput::from_sizes(Size::new(50.0, 20.0), Size::new(80.0, 45.0));
    child_output.scroll_geometry = Some(child_geometry);

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            overflow: Point::new(Overflow::Hidden, Overflow::Hidden),
            size: Size::new(Dimension::px(50.0), Dimension::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(2, child_output);

    compute_block(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(100.0), Some(40.0)),
            available: Size::new(Available::definite(100.0), Available::definite(40.0)),
        },
    );

    let geometry = tree.layouts[&2].scroll_geometry.unwrap();
    assert_eq!(geometry.scrollable_overflow().origin(), Point::new(-15.0, -4.0));
    assert_eq!(geometry.scrollable_overflow().size(), Size::new(95.0, 49.0));
}
```

Add an inline atomic child final-geometry test:

```rust
#[test]
fn block_inline_child_node_output_uses_final_inline_item_geometry() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;
        type Scalar = Scalar;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, _input: ComputeInput) -> ComputeOutput {
            self.outputs[&node]
        }
    }

    let child_overflow =
        ScrollRect::new(Point::new(-9.0, -3.0), Size::new(74.0, 34.0)).unwrap();
    let child_geometry = crate::scroll::scroll_geometry_from_layout(
        WritingMode::HorizontalTb,
        Direction::Ltr,
        Point::new(Overflow::Hidden, Overflow::Hidden),
        Size::new(40.0, 12.0),
        Edges::ZERO,
        Edges::ZERO,
        0.0,
        child_overflow,
    )
    .unwrap();
    let mut child_output =
        ComputeOutput::from_sizes(Size::new(40.0, 12.0), Size::new(65.0, 31.0));
    child_output.scroll_geometry = Some(child_geometry);

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::InlineBlock,
            overflow: Point::new(Overflow::Hidden, Overflow::Hidden),
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(2, child_output);

    compute_block(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(100.0), Some(40.0)),
            available: Size::new(Available::definite(100.0), Available::definite(40.0)),
        },
    );

    let child_layout = tree.layouts[&2];
    assert_eq!(child_layout.size, Size::new(40.0, 12.0));
    assert_eq!(child_layout.content_size, Size::new(65.0, 31.0));
    let geometry = child_layout.scroll_geometry.unwrap();
    assert_eq!(geometry.scrollport().size(), child_layout.size);
    assert_eq!(geometry.scrollable_overflow(), child_overflow);
}
```

Run:

```sh
cargo test -p surgeist-layout block_child_node_output_recomputes_child_scroll_geometry -- --nocapture
cargo test -p surgeist-layout block_absolute_child_scroll_geometry_uses_final_node_output_size -- --nocapture
cargo test -p surgeist-layout block_child_node_output_preserves_child_scrollable_overflow_origin -- --nocapture
cargo test -p surgeist-layout block_inline_child_node_output_uses_final_inline_item_geometry -- --nocapture
```

Expected: fail because block child `NodeOutputOf` values do not yet carry
scroll geometry.

- [ ] **Step 2: Add child node-output geometry helper**

Add this helper near `block_scroll_geometry`; reuse the
`child_scrollable_overflow` helper added in Task 5:

```rust
fn child_node_scroll_geometry<S: LayoutScalar>(
    style: &NodeInputOf<S>,
    size: Size<S>,
    content_size: Size<S>,
    padding: Edges<S>,
    border: Edges<S>,
    child_compute_geometry: Option<super::ScrollGeometryOf<S>>,
) -> super::ScrollGeometryOf<S> {
    let scrollable_overflow = child_scrollable_overflow(
        style,
        size,
        content_size,
        padding,
        border,
        child_compute_geometry,
    );
    scroll_geometry_from_layout(
        style.writing_mode,
        style.direction,
        style.overflow,
        size,
        padding,
        border,
        style.scrollbar_width,
        scrollable_overflow,
    )
    .expect("child scroll geometry is derived from finite non-negative layout output")
}
```

- [ ] **Step 3: Set child node-output geometry**

In every block `tree.set_unrounded(... NodeOutputOf { ... })` path that writes a
box child from a `ComputeOutput`, set `scroll_geometry` by recomputing from the
final node-output size:

```rust
scroll_geometry: Some(child_node_scroll_geometry(
    &child_style,
    output.size,
    output.content_size,
    child_padding,
    child_border,
    output.scroll_geometry,
)),
```

For absolute children, use the absolute layout `final_size`, resolved `padding`,
resolved `border`, and `output.content_size`:

```rust
scroll_geometry: Some(child_node_scroll_geometry(
    &style,
    final_size,
    output.content_size,
    padding,
    border,
    output.scroll_geometry,
)),
```

For inline atomic box children, use the final inline item fields, not the raw
`ComputeOutput` size fields:

```rust
scroll_geometry: Some(child_node_scroll_geometry(
    child_style,
    item.size,
    item.content_size,
    item.padding,
    item.border,
    output.scroll_geometry,
)),
```

For floats, carry the child style in `PendingFloat` or compute and store the
scroll geometry before pushing the pending float, then write that exact geometry
when the float is finally positioned. Do not copy a stale geometry if the stored
float size differs from the final `NodeOutput.size`.

Hidden, display-none, line-break, and inline-boundary outputs must remain
`None`.

- [ ] **Step 4: Run child-output checks**

Run:

```sh
cargo test -p surgeist-layout block_child_node_output_recomputes_child_scroll_geometry -- --nocapture
cargo test -p surgeist-layout block_absolute_child_scroll_geometry_uses_final_node_output_size -- --nocapture
cargo test -p surgeist-layout block_child_node_output_preserves_child_scrollable_overflow_origin -- --nocapture
cargo test -p surgeist-layout block_inline_child_node_output_uses_final_inline_item_geometry -- --nocapture
cargo test -p surgeist-layout block_layout_lays_out_absolute_children_without_flow_contribution_and_hides_display_none -- --nocapture
cargo test -p surgeist-layout block_atomic_inline_run_honors_line_break_child -- --nocapture
cargo fmt --check
git diff --check
```

Expected: all pass.

## Review And Commit Gate

After each task:

1. Worker reports changed files, tests run, and `git status --short --branch`.
2. A separate reviewer inspects the scoped task changes.
3. Coordinator reconciles findings.
4. Coordinator runs focused checks.
5. Coordinator commits only after the worker/reviewer cycle is clean.

Final implementation gate for this plan:

```sh
cargo test -p surgeist-layout
cargo clippy -p surgeist-layout --all-targets -- -D warnings
cargo fmt --check
git diff --check
git status --short --branch
```

Then assign a final clean-context holistic reviewer to inspect the full Phase 3
implementation against this plan, the sequence, the support matrix, crate
boundary, tests, and the modeling guide. Commit implementation work only after
the relevant reviewer gates are clean.
