# CSS Scrollport And Gutter Math Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Centralize current `overflow: scroll` scrollbar reservation and scrollport/scrollbar-gutter geometry math without changing layout behavior.

**Architecture:** Extend `src/scroll.rs` with pure scalar-generic helper types/functions for current classic scrollbar reservation and derived border-box, padding-box, content-box, scrollport, and scrollbar-gutter rect facts. Migrate duplicated block, flex, grid, and child-output calculations to those helpers only after focused helper tests lock existing behavior. This phase prepares algorithm integration for later scroll geometry output but does not emit full `ScrollGeometry` from algorithms yet.

**Tech Stack:** Rust 2024, existing `LayoutScalar`, `Point`, `Size`, `Edges`, `Direction`, `Overflow`, `NodeInputOf<S>`, crate-local tests, `cargo test -p surgeist-layout`, `cargo clippy -p surgeist-layout --all-targets -- -D warnings`, `cargo fmt --check`.

---

## Scope

This implements Phase 2 from:

- `plans/2026-07-09-surgeist-layout-css-scroll-geometry-sequence.md`
- `plans/2026-07-09-surgeist-layout-css-scroll-support-matrix.md`
- `plans/2026-07-09-surgeist-layout-scroll-geometry-core-implementation.md`

Phase 2 must:

- centralize current gutter-size and content-box-inset math;
- centralize typed border-box, padding-box, content-box, scrollport, and gutter-rect derivation for current classic scrollbar reservation;
- preserve existing `overflow: scroll` behavior in block, flex, grid, and leaf-style paths;
- keep `overflow: auto`, stable gutters, both-edge gutters, nested clipping, and scroll output emission out of scope;
- keep current direction-based inline gutter placement;
- keep physical-axis behavior unchanged: vertical scrollbar width is controlled by `overflow-y`, horizontal scrollbar height is controlled by `overflow-x`;
- keep current box-sizing behavior unchanged.

Phase 2 must not:

- add CSS parsing, style cascade, root lowering, live scroll state, platform input, animation, or rendering;
- add `overflow: auto` semantics;
- add stable or both-edge `scrollbar-gutter`;
- change `overflow: clip` margin-collapse behavior;
- emit `ScrollGeometry` from block/flex/grid/root algorithms.

## Files

- Modify: `src/scroll.rs`
- Modify: `src/scroll_tests.rs`
- Modify: `src/compute.rs`
- Modify: `src/block.rs`
- Modify: `src/leaf_tests.rs`
- Modify: `src/flex.rs`
- Modify: `src/grid/mod.rs`
- Modify: `src/grid/child.rs`
- Modify: `src/grid/lanes.rs`
- Modify: `src/grid_tests.rs`

## Shared API To Add

Add `Edges` to the existing `src/scroll.rs` import list, then add these
helpers to `src/scroll.rs`:

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollbarReservationOf<S: LayoutScalar = DefaultScalar> {
    size: Size<S>,
    inset: Edges<S>,
}

pub type ScrollbarReservation = ScrollbarReservationOf<DefaultScalar>;

impl<S: LayoutScalar> ScrollbarReservationOf<S> {
    #[must_use]
    pub fn from_overflow(
        overflow: Point<Overflow>,
        scrollbar_width: S,
        direction: Direction,
    ) -> Self {
        let size = scrollbar_size_from_overflow(overflow, scrollbar_width);
        Self {
            size,
            inset: scrollbar_inset_from_size(size, direction),
        }
    }

    #[must_use]
    pub const fn size(self) -> Size<S> {
        self.size
    }

    #[must_use]
    pub const fn inset(self) -> Edges<S> {
        self.inset
    }
}

#[must_use]
pub fn scrollbar_size_from_overflow<S: LayoutScalar>(
    overflow: Point<Overflow>,
    scrollbar_width: S,
) -> Size<S> {
    Size::new(
        if overflow.y == Overflow::Scroll {
            scrollbar_width
        } else {
            S::ZERO
        },
        if overflow.x == Overflow::Scroll {
            scrollbar_width
        } else {
            S::ZERO
        },
    )
}

#[must_use]
pub fn scrollbar_inset_from_size<S: LayoutScalar>(
    size: Size<S>,
    direction: Direction,
) -> Edges<S> {
    match direction {
        Direction::Ltr => Edges {
            right: size.width,
            bottom: size.height,
            ..Edges::<S>::ZERO
        },
        Direction::Rtl => Edges {
            left: size.width,
            bottom: size.height,
            ..Edges::<S>::ZERO
        },
    }
}

#[must_use]
pub fn content_box_inset_with_scrollbar<S: LayoutScalar>(
    padding: Edges<S>,
    border: Edges<S>,
    reservation: ScrollbarReservationOf<S>,
) -> Edges<S> {
    padding + border + reservation.inset()
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollBoxRectsOf<S: LayoutScalar = DefaultScalar> {
    border_box: ScrollRectOf<S>,
    padding_box: ScrollRectOf<S>,
    content_box: ScrollRectOf<S>,
    scrollport: ScrollRectOf<S>,
    gutters: ScrollbarGutterRectsOf<S>,
}

pub type ScrollBoxRects = ScrollBoxRectsOf<DefaultScalar>;

impl<S: LayoutScalar> ScrollBoxRectsOf<S> {
    #[must_use]
    pub const fn border_box(self) -> ScrollRectOf<S> {
        self.border_box
    }

    #[must_use]
    pub const fn padding_box(self) -> ScrollRectOf<S> {
        self.padding_box
    }

    #[must_use]
    pub const fn content_box(self) -> ScrollRectOf<S> {
        self.content_box
    }

    #[must_use]
    pub const fn scrollport(self) -> ScrollRectOf<S> {
        self.scrollport
    }

    #[must_use]
    pub const fn gutters(self) -> ScrollbarGutterRectsOf<S> {
        self.gutters
    }
}

pub fn scroll_box_rects_from_border_box<S: LayoutScalar>(
    border_box: ScrollRectOf<S>,
    padding: Edges<S>,
    border: Edges<S>,
    reservation: ScrollbarReservationOf<S>,
) -> Result<ScrollBoxRectsOf<S>, ScrollUnsupportedFeature> {
    let padding_box = inset_scroll_rect(border_box, border)?;
    let content_box = inset_scroll_rect(
        border_box,
        content_box_inset_with_scrollbar(padding, border, reservation),
    )?;
    let scrollport = inset_scroll_rect(padding_box, reservation.inset())?;
    let gutters = scrollbar_gutter_rects_from_padding_box(padding_box, reservation)?;

    Ok(ScrollBoxRectsOf {
        border_box,
        padding_box,
        content_box,
        scrollport,
        gutters,
    })
}

fn inset_scroll_rect<S: LayoutScalar>(
    rect: ScrollRectOf<S>,
    inset: Edges<S>,
) -> Result<ScrollRectOf<S>, ScrollUnsupportedFeature> {
    let origin = rect.origin();
    let size = rect.size();
    ScrollRectOf::new(
        Point::new(origin.x + inset.left, origin.y + inset.top),
        Size::new(
            (size.width - inset.horizontal_sum()).max(S::ZERO),
            (size.height - inset.vertical_sum()).max(S::ZERO),
        ),
    )
}

fn scrollbar_gutter_rects_from_padding_box<S: LayoutScalar>(
    padding_box: ScrollRectOf<S>,
    reservation: ScrollbarReservationOf<S>,
) -> Result<ScrollbarGutterRectsOf<S>, ScrollUnsupportedFeature> {
    let origin = padding_box.origin();
    let size = padding_box.size();
    let gutter_size = reservation.size();
    let inset = reservation.inset();

    let vertical = if gutter_size.width > S::ZERO {
        let x = if inset.left > S::ZERO {
            origin.x
        } else {
            origin.x + (size.width - gutter_size.width).max(S::ZERO)
        };
        Some(ScrollRectOf::new(
            Point::new(x, origin.y),
            Size::new(
                gutter_size.width.min(size.width),
                (size.height - gutter_size.height).max(S::ZERO),
            ),
        )?)
    } else {
        None
    };

    let horizontal = if gutter_size.height > S::ZERO {
        let x = origin.x + inset.left.min(size.width);
        Some(ScrollRectOf::new(
            Point::new(x, origin.y + (size.height - gutter_size.height).max(S::ZERO)),
            Size::new(
                (size.width - gutter_size.width).max(S::ZERO),
                gutter_size.height.min(size.height),
            ),
        )?)
    } else {
        None
    };

    Ok(ScrollbarGutterRectsOf::new(horizontal, vertical))
}
```

Do not reexport these helpers from `lib.rs` in Phase 2 unless a reviewer finds
that root/runtime/render need them as public front doors now. They are shared
crate-internal algorithm helpers for preserving current behavior.

## Task 1: Add Shared Scrollbar Reservation Helpers

**Files:**

- Modify: `src/scroll.rs`
- Modify: `src/scroll_tests.rs`

- [ ] **Step 1: Add failing helper tests**

Append to `src/scroll_tests.rs`:

```rust
use crate::Edges;
use crate::scroll::{ScrollBoxRects, ScrollbarReservation};

#[test]
fn scrollbar_size_uses_scroll_overflow_on_opposite_physical_axis() {
    assert_eq!(
        crate::scroll::scrollbar_size_from_overflow(
            Point::new(Overflow::Visible, Overflow::Scroll),
            15.0,
        ),
        Size::new(15.0, 0.0)
    );
    assert_eq!(
        crate::scroll::scrollbar_size_from_overflow(
            Point::new(Overflow::Scroll, Overflow::Visible),
            15.0,
        ),
        Size::new(0.0, 15.0)
    );
    assert_eq!(
        crate::scroll::scrollbar_size_from_overflow(
            Point::new(Overflow::Scroll, Overflow::Scroll),
            15.0,
        ),
        Size::new(15.0, 15.0)
    );
}

#[test]
fn scrollbar_reservation_places_inline_gutter_by_direction() {
    let ltr = ScrollbarReservation::from_overflow(
        Point::new(Overflow::Visible, Overflow::Scroll),
        12.0,
        Direction::Ltr,
    );
    let rtl = ScrollbarReservation::from_overflow(
        Point::new(Overflow::Visible, Overflow::Scroll),
        12.0,
        Direction::Rtl,
    );

    assert_eq!(ltr.size(), Size::new(12.0, 0.0));
    assert_eq!(ltr.inset(), Edges::new(0.0, 12.0, 0.0, 0.0));
    assert_eq!(rtl.size(), Size::new(12.0, 0.0));
    assert_eq!(rtl.inset(), Edges::new(0.0, 0.0, 0.0, 12.0));
}

#[test]
fn content_box_inset_includes_padding_border_and_scrollbar_reservation() {
    let padding = Edges::new(1.0, 2.0, 3.0, 4.0);
    let border = Edges::new(5.0, 6.0, 7.0, 8.0);
    let reservation = ScrollbarReservation::from_overflow(
        Point::new(Overflow::Visible, Overflow::Scroll),
        9.0,
        Direction::Ltr,
    );

    assert_eq!(
        crate::scroll::content_box_inset_with_scrollbar(padding, border, reservation),
        Edges::new(6.0, 17.0, 10.0, 12.0)
    );
}

#[test]
fn scroll_box_rects_derive_ltr_scrollport_and_gutter_rects() {
    let rects = crate::scroll::scroll_box_rects_from_border_box(
        ScrollRect::new(Point::new(10.0, 20.0), Size::new(100.0, 80.0)).unwrap(),
        Edges::new(2.0, 3.0, 4.0, 5.0),
        Edges::all(1.0),
        ScrollbarReservation::from_overflow(
            Point::new(Overflow::Scroll, Overflow::Scroll),
            10.0,
            Direction::Ltr,
        ),
    )
    .unwrap();

    assert_eq!(
        rects.padding_box(),
        ScrollRect::new(Point::new(11.0, 21.0), Size::new(98.0, 78.0)).unwrap()
    );
    assert_eq!(
        rects.content_box(),
        ScrollRect::new(Point::new(16.0, 23.0), Size::new(80.0, 62.0)).unwrap()
    );
    assert_eq!(
        rects.scrollport(),
        ScrollRect::new(Point::new(11.0, 21.0), Size::new(88.0, 68.0)).unwrap()
    );
    assert_eq!(
        rects.gutters().vertical(),
        Some(ScrollRect::new(Point::new(99.0, 21.0), Size::new(10.0, 68.0)).unwrap())
    );
    assert_eq!(
        rects.gutters().horizontal(),
        Some(ScrollRect::new(Point::new(11.0, 89.0), Size::new(88.0, 10.0)).unwrap())
    );
}

#[test]
fn scroll_box_rects_shift_rtl_scrollport_after_left_gutter() {
    let rects = crate::scroll::scroll_box_rects_from_border_box(
        ScrollRect::new(Point::ZERO, Size::new(100.0, 40.0)).unwrap(),
        Edges::ZERO,
        Edges::ZERO,
        ScrollbarReservation::from_overflow(
            Point::new(Overflow::Visible, Overflow::Scroll),
            12.0,
            Direction::Rtl,
        ),
    )
    .unwrap();

    assert_eq!(
        rects.scrollport(),
        ScrollRect::new(Point::new(12.0, 0.0), Size::new(88.0, 40.0)).unwrap()
    );
    assert_eq!(
        rects.gutters().vertical(),
        Some(ScrollRect::new(Point::ZERO, Size::new(12.0, 40.0)).unwrap())
    );
    assert_eq!(rects.gutters().horizontal(), None);
}

#[test]
fn scroll_box_rects_clamp_overlarge_insets_to_empty_rects() {
    let rects: ScrollBoxRects = crate::scroll::scroll_box_rects_from_border_box(
        ScrollRect::new(Point::ZERO, Size::new(10.0, 10.0)).unwrap(),
        Edges::all(20.0),
        Edges::all(20.0),
        ScrollbarReservation::from_overflow(
            Point::new(Overflow::Scroll, Overflow::Scroll),
            20.0,
            Direction::Ltr,
        ),
    )
    .unwrap();

    assert_eq!(rects.content_box().size(), Size::ZERO);
    assert_eq!(rects.scrollport().size(), Size::ZERO);
}
```

Run:

```sh
cargo test -p surgeist-layout scrollbar_ -- --nocapture
```

Expected: compile failure naming missing `ScrollbarReservation`, `ScrollBoxRects`,
or helper functions.

- [ ] **Step 2: Implement helper types/functions**

Add the helper code from **Shared API To Add** to `src/scroll.rs`.

- [ ] **Step 3: Run focused checks**

Run:

```sh
cargo test -p surgeist-layout scrollbar_ -- --nocapture
cargo fmt --check
git diff --check
```

Expected: all pass.

## Task 2: Migrate Root/Leaf And Block Paths

**Files:**

- Modify: `src/compute.rs`
- Modify: `src/block.rs`
- Modify: `src/leaf_tests.rs`

- [ ] **Step 1: Replace root/leaf scrollbar-size calculation**

In `src/compute.rs`, import:

```rust
use crate::scroll::scrollbar_size_from_overflow;
```

In `compute_root`, replace the local `let scrollbar_size = Size::new(...)`
block with:

```rust
let scrollbar_size = scrollbar_size_from_overflow(style.overflow, style.scrollbar_width);
```

- [ ] **Step 2: Replace leaf gutter and content-box inset calculation**

In `compute_leaf_with_resolver` in `src/compute.rs`, replace the local
`scrollbar_gutter = Size::new(...)` and manual right/bottom inset updates with:

```rust
let scrollbar_reservation = ScrollbarReservationOf::from_overflow(
    style.overflow,
    style.scrollbar_width,
    Direction::Ltr,
);
let content_box_inset =
    content_box_inset_with_scrollbar(padding, border, scrollbar_reservation);
```

Use `Direction::Ltr` intentionally here to preserve current leaf behavior:
leaf layout currently reserves scrollbar gutter on the physical right and
bottom edges even when `style.direction` is RTL. Do not change that behavior in
Phase 2.

- [ ] **Step 3: Add an RTL leaf preservation test**

Add this test to `src/leaf_tests.rs`:

```rust
#[test]
fn leaf_layout_preserves_physical_end_scrollbar_gutter_for_rtl() {
    let input = ComputeInput {
        run_mode: RunMode::PerformLayout,
        sizing_mode: SizingMode::InherentSize,
        axis: RequestedAxis::Both,
        known: Size::NONE,
        parent: Size::new(Some(200.0), Some(100.0)),
        available: Size::new(Available::definite(100.0), Available::definite(50.0)),
    };
    let node_input = NodeInput {
        direction: Direction::Rtl,
        overflow: Point::new(Overflow::Visible, Overflow::Scroll),
        scrollbar_width: 15.0,
        padding: Edges::all(Length::px(2.0)),
        border: Edges::all(Length::px(1.0)),
        ..NodeInput::default()
    };

    let output = compute_leaf(input, &node_input, |_known, available| {
        assert_eq!(available.width, Available::definite(79.0));
        assert_eq!(available.height, Available::definite(44.0));
        Size::new(40.0, 12.0)
    });

    assert_eq!(output.size, Size::new(61.0, 18.0));
    assert_eq!(output.content_size, Size::new(44.0, 16.0));
}
```

- [ ] **Step 4: Replace block container gutter calculation**

In `src/block.rs`, import the helpers:

```rust
use crate::scroll::{
    ScrollbarReservationOf, content_box_inset_with_scrollbar, scrollbar_size_from_overflow,
};
```

In `src/compute.rs`, the imports for this task are:

```rust
use crate::scroll::{
    ScrollbarReservationOf, content_box_inset_with_scrollbar, scrollbar_size_from_overflow,
};
```

Also add `Direction` to the existing `super::{...}` import list in
`src/compute.rs`, because the leaf path intentionally passes
`Direction::Ltr` to preserve current physical right/bottom gutter behavior.

In `Constants::new`, replace the local `scrollbar_gutter = Size::new(...)`
and direction `match` with:

```rust
let scrollbar_reservation = ScrollbarReservationOf::from_overflow(
    style.overflow,
    style.scrollbar_width,
    style.direction,
);
let scrollbar_gutter = scrollbar_reservation.inset();
```

Then compute:

```rust
let content_box_inset = content_box_inset_with_scrollbar(
    padding,
    border,
    scrollbar_reservation,
);
```

Preserve the existing `scrollbar_gutter` field value as `Edges<S>`.

- [ ] **Step 5: Replace block child scrollbar-size helper body**

Change `child_scrollbar_size` to:

```rust
fn child_scrollbar_size<S: LayoutScalar>(style: &NodeInputOf<S>) -> Size<S> {
    scrollbar_size_from_overflow(style.overflow, style.scrollbar_width)
}
```

- [ ] **Step 6: Run block/root/leaf focused checks**

Run:

```sh
cargo test -p surgeist-layout block_rtl_scrollbar_gutter_uses_left_inset -- --nocapture
cargo test -p surgeist-layout leaf_layout_reserves_scrollbar_gutter_for_scroll_overflow -- --nocapture
cargo test -p surgeist-layout leaf_layout_preserves_physical_end_scrollbar_gutter_for_rtl -- --nocapture
cargo test -p surgeist-layout root_layout_stores_child_output_as_root_layout -- --nocapture
cargo fmt --check
git diff --check
```

Expected: all pass and no expected layout values change.

## Task 3: Migrate Flex Paths

**Files:**

- Modify: `src/flex.rs`

- [ ] **Step 1: Replace flex container gutter calculation**

In `src/flex.rs`, import:

```rust
use crate::scroll::{
    ScrollbarReservationOf, content_box_inset_with_scrollbar, scrollbar_size_from_overflow,
};
```

In `Constants::new`, replace the local `Point::new(...)` gutter calculation
and direction `match` with:

```rust
let scrollbar_reservation = ScrollbarReservationOf::from_overflow(
    style.overflow,
    style.scrollbar_width,
    style.direction,
);
let scrollbar_gutter = Point::new(
    scrollbar_reservation.size().width,
    scrollbar_reservation.size().height,
);
let content_box_inset =
    content_box_inset_with_scrollbar(padding, border, scrollbar_reservation);
```

Preserve the existing `scrollbar_gutter: Point<S>` field for now so this task
does not refactor flex internals beyond the shared helper migration.

- [ ] **Step 2: Replace `item_scrollbar_size` body**

Change `item_scrollbar_size` to:

```rust
fn item_scrollbar_size<S: LayoutScalar>(overflow: Point<Overflow>, scrollbar_width: S) -> Size<S> {
    scrollbar_size_from_overflow(overflow, scrollbar_width)
}
```

- [ ] **Step 3: Run flex focused checks**

Run:

```sh
cargo test -p surgeist-layout flex_container_reserves_scrollbar_gutter_from_inner_size -- --nocapture
cargo test -p surgeist-layout flex_scrollbar_gutter_uses_left_inset_for_rtl_containers -- --nocapture
cargo test -p surgeist-layout flex_child_layout_records_scrollbar_size_for_scroll_overflow -- --nocapture
cargo test -p surgeist-layout flex_absolute_child_layout_records_scrollbar_size_for_scroll_overflow -- --nocapture
cargo fmt --check
git diff --check
```

Expected: all pass and no expected layout values change.

## Task 4: Migrate Grid Container And Child Paths

**Files:**

- Modify: `src/grid/mod.rs`
- Modify: `src/grid/child.rs`
- Modify: `src/grid_tests.rs`

- [ ] **Step 1: Replace grid container gutter calculation**

In `src/grid/mod.rs`, import:

```rust
use crate::scroll::{
    ScrollbarReservationOf, content_box_inset_with_scrollbar,
};
```

In `Constants::new`, replace the local `Size::new(...)` gutter calculation
and direction `match` with:

```rust
let scrollbar_reservation = ScrollbarReservationOf::from_overflow(
    style.overflow,
    style.scrollbar_width,
    style.direction,
);
let content_box_inset =
    content_box_inset_with_scrollbar(padding, border, scrollbar_reservation);
```

Do not introduce a `scrollbar_gutter` local or field in `src/grid/mod.rs`;
current grid container constants only need `content_box_inset`.

- [ ] **Step 2: Replace grid child scrollbar-size calculations**

In `src/grid/child.rs`, import:

```rust
use crate::scroll::scrollbar_size_from_overflow;
```

Replace each local `Size::new(if child_style.overflow.y == Overflow::Scroll { ... })`
scrollbar-size calculation with:

```rust
let scrollbar_size =
    scrollbar_size_from_overflow(child_style.overflow, child_style.scrollbar_width);
```

Do this for normal grid children and absolute grid children.

- [ ] **Step 3: Add normal and absolute grid child scrollbar-size tests**

Add focused tests to `src/grid_tests.rs` near the existing grid child layout
tests:

```rust
#[test]
fn grid_child_layout_records_scrollbar_size_for_scroll_overflow() {
    #[derive(Default)]
    struct GridTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for GridTree {
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

    impl Compute for GridTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, _node: Self::Node, input: ComputeInput) -> ComputeOutput {
            ComputeOutput::from_sizes(input.known.map(|value| value.unwrap_or(0.0)), Size::ZERO)
        }
    }

    let mut tree = GridTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            grid_template_columns: vec![TrackComponent::px(20.0)],
            grid_template_rows: vec![TrackComponent::px(10.0)],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            overflow: Point::new(Overflow::Scroll, Overflow::Scroll),
            scrollbar_width: 11.0,
            ..NodeInput::default()
        },
    );

    compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(500.0), Some(400.0)),
            available: Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        },
    );

    assert_eq!(tree.layouts[&2].scrollbar_size, Size::new(11.0, 11.0));
}
```

Also add `grid_absolute_child_layout_records_scrollbar_size_for_scroll_overflow`
near the existing absolute grid child tests. Model it on the existing flex
absolute scrollbar-size test, but call `compute_grid`, set the child
`position: Position::Absolute`, `overflow: Point::new(Overflow::Scroll, Overflow::Scroll)`,
and `scrollbar_width: 12.0`, then assert:

```rust
assert_eq!(tree.layouts[&2].scrollbar_size, Size::new(12.0, 12.0));
```

Use the established `GridTree`/`Compute` pattern already present in
`src/grid_tests.rs` for child layout tests.

- [ ] **Step 4: Run grid focused checks**

Run:

```sh
cargo test -p surgeist-layout grid_content_box_compute_size_does_not_add_scrollbar_to_authored_size -- --nocapture
cargo test -p surgeist-layout grid_scrollbar_gutter_does_not_force_outer_size_past_authored_size -- --nocapture
cargo test -p surgeist-layout grid_absolute_child_content_box_size_includes_padding_and_border -- --nocapture
cargo test -p surgeist-layout grid_child_layout_records_scrollbar_size_for_scroll_overflow -- --nocapture
cargo test -p surgeist-layout grid_absolute_child_layout_records_scrollbar_size_for_scroll_overflow -- --nocapture
cargo fmt --check
git diff --check
```

Expected: all pass and no expected layout values change.

## Task 5: Migrate Grid Lane Child Path

**Files:**

- Modify: `src/grid/lanes.rs`
- Modify: `src/grid_tests.rs`

- [ ] **Step 1: Replace grid-lanes scrollbar-size calculation**

In `src/grid/lanes.rs`, import:

```rust
use crate::scroll::scrollbar_size_from_overflow;
```

Replace the local `scrollbar_size: Size::new(...)` calculation in the lane child
layout report with:

```rust
scrollbar_size: scrollbar_size_from_overflow(
    child_style.overflow,
    child_style.scrollbar_width,
),
```

- [ ] **Step 2: Run lane/grid regression checks**

Add or update a focused grid-lanes test in `src/grid_tests.rs` so a lane child
with `overflow: Point::new(Overflow::Scroll, Overflow::Scroll)` and
`scrollbar_width: 10.0` records:

```rust
assert_eq!(tree.layouts[&2].scrollbar_size, Size::new(10.0, 10.0));
```

It is acceptable to update `grid_lanes_display_uses_separate_placement_path_before_child_layout`
if that keeps the fixture small and still verifies the lane child layout path.

- [ ] **Step 3: Run lane/grid regression checks**

Run:

```sh
cargo test -p surgeist-layout grid_lanes -- --nocapture
cargo test -p surgeist-layout grid_child_layout_records_scrollbar_size_for_scroll_overflow -- --nocapture
cargo test -p surgeist-layout grid_absolute_child_layout_records_scrollbar_size_for_scroll_overflow -- --nocapture
cargo fmt --check
git diff --check
```

Expected: all pass and no expected layout values change.

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

Then assign a final clean-context holistic reviewer to inspect the full Phase 2
implementation against this plan, the sequence, the support matrix, crate
boundary, tests, and the modeling guide. Commit implementation work only after
the relevant reviewer gates are clean.
