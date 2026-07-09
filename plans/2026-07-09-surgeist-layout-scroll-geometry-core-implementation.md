# CSS Scroll Geometry Core Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the typed scalar-generic scroll geometry core that later layout algorithms will emit and root/runtime/render will consume.

**Architecture:** Create a focused `src/scroll.rs` module for scroll geometry types, invariants, and pure clamp/range helpers. Keep Phase 1 free of block/flex/grid/root algorithm integration; it only establishes the typed contract, public front door, and tests. Preserve layout ownership: no CSS parsing, no style cascade, no root lowering, no live scroll offsets, no paint or platform behavior.

**Tech Stack:** Rust 2024, `LayoutScalar`, existing `Point`, `Size`, `Edges`, `Axis`, `WritingMode`, `Direction`, `Overflow`, crate-local unit tests, `cargo test -p surgeist-layout`, `cargo clippy -p surgeist-layout --all-targets -- -D warnings`, `cargo fmt --check`.

---

## Scope

This implements Phase 1 from:

- `plans/2026-07-09-surgeist-layout-css-scroll-geometry-sequence.md`
- `plans/2026-07-09-surgeist-layout-css-scroll-support-matrix.md`

Phase 1 must define the core model only:

- scalar-generic scroll rect/range/clamp types;
- physical-axis geometry plus writing-mode/direction metadata;
- scroll container classification per axis;
- a mixed-axis overflow policy boundary decision;
- scrollport, overflow clip rect, scrollable overflow rect, maximum offset, and scrollbar gutter rect output facts;
- typed unsupported diagnostics.

Phase 1 must not:

- integrate scroll output into block, flex, grid, inline, root, or hidden compute;
- implement semantic `overflow: auto`;
- implement stable or both-edge gutters;
- propagate nested clipping;
- implement scroll snap, `scroll-padding`, or `scroll-margin`;
- store live scroll offsets.

## Boundary Decisions For Phase 1

Use these decisions so workers do not re-open product scope while implementing:

1. Phase 1 chooses the root-pre-resolved boundary for mixed-axis overflow coupling.
   Layout records `ScrollOverflowCouplingPolicy::RootPreResolved` and exposes an unsupported diagnostic for callers that try to require layout-owned visible-to-auto coupling before Phase 6.
2. Public output is physical geometry with explicit `WritingMode` and `Direction` metadata. Logical helper methods can be added later; Phase 1 must not invent platform offset conventions.
3. `overflow: hidden` and `overflow: scroll` can expose non-zero scroll ranges. `overflow: clip` exposes clipping but no scroll range.
4. `overflow: auto`, stable gutters, both-edge gutters, scroll target geometry, and snap geometry are unsupported in Phase 1 and must be represented by typed diagnostics rather than silent no-ops.

## Files

- Create: `src/scroll.rs`
- Create: `src/scroll_tests.rs`
- Modify: `src/lib.rs`

## Task 1: Add Core Scroll Geometry Types

**Files:**

- Create: `src/scroll.rs`
- Modify: `src/lib.rs`
- Create: `src/scroll_tests.rs`

- [ ] **Step 1: Add failing tests for ranges, clamps, and f64 support**

Add `src/scroll_tests.rs` with these tests:

```rust
use crate::{
    Point, ScrollOffset, ScrollOffsetOf, ScrollRange, ScrollRangeOf, ScrollRect,
    ScrollUnsupportedFeature, Size,
};

#[test]
fn scroll_range_clamps_offsets_to_non_negative_maximum() {
    let range = ScrollRange::new(Size::new(120.0, 40.0)).unwrap();

    assert_eq!(
        range.clamp(ScrollOffset::new(Point::new(-10.0, 10.0))),
        ScrollOffset::new(Point::new(0.0, 10.0))
    );
    assert_eq!(
        range.clamp(ScrollOffset::new(Point::new(200.0, 99.0))),
        ScrollOffset::new(Point::new(120.0, 40.0))
    );
}

#[test]
fn scroll_range_rejects_negative_or_non_finite_maximum() {
    assert_eq!(
        ScrollRange::new(Size::new(-1.0, 0.0)).unwrap_err(),
        ScrollUnsupportedFeature::InvalidScrollRange
    );
    assert_eq!(
        ScrollRange::new(Size::new(f32::INFINITY, 0.0)).unwrap_err(),
        ScrollUnsupportedFeature::InvalidScrollRange
    );
}

#[test]
fn scroll_rect_rejects_negative_or_non_finite_size() {
    assert_eq!(
        ScrollRect::new(Point::ZERO, Size::new(10.0, -1.0)).unwrap_err(),
        ScrollUnsupportedFeature::InvalidScrollRect
    );
    assert_eq!(
        ScrollRect::new(Point::new(f32::NAN, 0.0), Size::new(10.0, 1.0)).unwrap_err(),
        ScrollUnsupportedFeature::InvalidScrollRect
    );
}

#[test]
fn scroll_geometry_supports_f64() {
    let range = ScrollRangeOf::<f64>::new(Size::new(1_000_000_000_000.0, 0.5)).unwrap();

    assert_eq!(
        range.clamp(ScrollOffsetOf::<f64>::new(Point::new(2_000_000_000_000.0, 1.0))),
        ScrollOffsetOf::<f64>::new(Point::new(1_000_000_000_000.0, 0.5))
    );
}
```

- [ ] **Step 2: Wire the test module so the tests fail for missing symbols**

In `src/lib.rs`, add the test module declaration near the other test modules:

```rust
#[cfg(test)]
mod scroll_tests;
```

Run:

```sh
cargo test -p surgeist-layout scroll_range -- --nocapture
```

Expected: compile failure naming missing `ScrollOffset`, `ScrollRange`, `ScrollRect`, or `ScrollUnsupportedFeature`.

- [ ] **Step 3: Implement the core types**

Create `src/scroll.rs`:

```rust
use super::{DefaultScalar, LayoutScalar, Point, Size};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScrollUnsupportedFeature {
    InvalidScrollRect,
    InvalidScrollRange,
    InvalidScrollGeometry,
    OverflowAuto,
    OverflowClipMargin,
    ScrollbarGutterStable,
    ScrollbarGutterBothEdges,
    ScrollPadding,
    ScrollMargin,
    ScrollSnap,
    LayoutOwnedMixedAxisOverflowCoupling,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollRectOf<S: LayoutScalar = DefaultScalar> {
    origin: Point<S>,
    size: Size<S>,
}

pub type ScrollRect = ScrollRectOf<DefaultScalar>;

impl<S: LayoutScalar> ScrollRectOf<S> {
    pub fn new(
        origin: Point<S>,
        size: Size<S>,
    ) -> Result<Self, ScrollUnsupportedFeature> {
        if !origin.x.is_finite()
            || !origin.y.is_finite()
            || !size.width.is_finite()
            || !size.height.is_finite()
            || size.width < S::ZERO
            || size.height < S::ZERO
        {
            return Err(ScrollUnsupportedFeature::InvalidScrollRect);
        }

        Ok(Self { origin, size })
    }

    #[must_use]
    pub const fn origin(self) -> Point<S> {
        self.origin
    }

    #[must_use]
    pub const fn size(self) -> Size<S> {
        self.size
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollOffsetOf<S: LayoutScalar = DefaultScalar> {
    position: Point<S>,
}

pub type ScrollOffset = ScrollOffsetOf<DefaultScalar>;

impl<S: LayoutScalar> ScrollOffsetOf<S> {
    #[must_use]
    pub const fn new(position: Point<S>) -> Self {
        Self { position }
    }

    #[must_use]
    pub const fn position(self) -> Point<S> {
        self.position
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollRangeOf<S: LayoutScalar = DefaultScalar> {
    maximum_offset: Size<S>,
}

pub type ScrollRange = ScrollRangeOf<DefaultScalar>;

impl<S: LayoutScalar> ScrollRangeOf<S> {
    pub fn new(maximum_offset: Size<S>) -> Result<Self, ScrollUnsupportedFeature> {
        if !maximum_offset.width.is_finite()
            || !maximum_offset.height.is_finite()
            || maximum_offset.width < S::ZERO
            || maximum_offset.height < S::ZERO
        {
            return Err(ScrollUnsupportedFeature::InvalidScrollRange);
        }

        Ok(Self { maximum_offset })
    }

    #[must_use]
    pub const fn maximum_offset(self) -> Size<S> {
        self.maximum_offset
    }

    #[must_use]
    pub fn clamp(self, offset: ScrollOffsetOf<S>) -> ScrollOffsetOf<S> {
        let position = offset.position();
        ScrollOffsetOf::new(Point::new(
            position.x.max(S::ZERO).min(self.maximum_offset.width),
            position.y.max(S::ZERO).min(self.maximum_offset.height),
        ))
    }
}
```

- [ ] **Step 4: Reexport the core types intentionally**

In `src/lib.rs`, add the module:

```rust
mod scroll;
```

Add public reexports:

```rust
pub use scroll::{
    ScrollOffset, ScrollOffsetOf, ScrollRange, ScrollRangeOf, ScrollRect, ScrollRectOf,
    ScrollUnsupportedFeature,
};
```

- [ ] **Step 5: Run focused checks**

Run:

```sh
cargo test -p surgeist-layout scroll_range -- --nocapture
cargo test -p surgeist-layout scroll_rect -- --nocapture
cargo test -p surgeist-layout scroll_geometry_supports_f64 -- --nocapture
cargo fmt --check
git diff --check
```

Expected: all pass.

## Task 2: Add Scroll Container Facts And Output Front Door

**Files:**

- Modify: `src/scroll.rs`
- Modify: `src/scroll_tests.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Add failing tests for container classification and output facts**

Append to `src/scroll_tests.rs`:

```rust
use crate::{
    Direction, Overflow, ScrollContainerAxis, ScrollContainerFacts, ScrollGeometry,
    ScrollOverflowCouplingPolicy, ScrollOverflowExposure, ScrollbarGutterRects,
    WritingMode,
};

#[test]
fn scroll_container_facts_distinguish_hidden_clip_and_scroll() {
    let hidden = ScrollContainerAxis::from_overflow(Overflow::Hidden).unwrap();
    let clip = ScrollContainerAxis::from_overflow(Overflow::Clip).unwrap();
    let scroll = ScrollContainerAxis::from_overflow(Overflow::Scroll).unwrap();
    let visible = ScrollContainerAxis::from_overflow(Overflow::Visible).unwrap();

    assert_eq!(hidden.exposure(), ScrollOverflowExposure::ScrollableClip);
    assert!(hidden.exposes_scroll_range());
    assert_eq!(clip.exposure(), ScrollOverflowExposure::ClipOnly);
    assert!(!clip.exposes_scroll_range());
    assert_eq!(scroll.exposure(), ScrollOverflowExposure::ScrollableClip);
    assert!(scroll.exposes_scroll_range());
    assert_eq!(visible.exposure(), ScrollOverflowExposure::Visible);
    assert!(!visible.exposes_scroll_range());
}

#[test]
fn scroll_geometry_front_door_preserves_physical_rects_and_flow_metadata() {
    let scrollport = ScrollRect::new(Point::new(1.0, 2.0), Size::new(80.0, 40.0)).unwrap();
    let overflow = ScrollRect::new(Point::ZERO, Size::new(120.0, 90.0)).unwrap();
    let clip = ScrollRect::new(Point::new(1.0, 2.0), Size::new(80.0, 40.0)).unwrap();
    let range = ScrollRange::new(Size::new(40.0, 50.0)).unwrap();
    let gutters = ScrollbarGutterRects::new(None, None);
    let geometry = ScrollGeometry::new(
        WritingMode::VerticalRl,
        Direction::Rtl,
        ScrollContainerFacts::new(
            ScrollContainerAxis::from_overflow(Overflow::Scroll).unwrap(),
            ScrollContainerAxis::from_overflow(Overflow::Hidden).unwrap(),
        ),
        scrollport,
        Some(clip),
        overflow,
        range,
        gutters,
    )
    .unwrap();

    assert_eq!(geometry.writing_mode(), WritingMode::VerticalRl);
    assert_eq!(geometry.direction(), Direction::Rtl);
    assert_eq!(geometry.scrollport(), scrollport);
    assert_eq!(geometry.overflow_clip(), Some(clip));
    assert_eq!(geometry.scrollable_overflow(), overflow);
    assert_eq!(geometry.range(), range);
}

#[test]
fn scroll_geometry_rejects_clip_only_axis_with_non_zero_range() {
    let scrollport = ScrollRect::new(Point::ZERO, Size::new(80.0, 40.0)).unwrap();
    let overflow = ScrollRect::new(Point::ZERO, Size::new(120.0, 90.0)).unwrap();
    let range = ScrollRange::new(Size::new(40.0, 0.0)).unwrap();
    let gutters = ScrollbarGutterRects::new(None, None);

    assert_eq!(
        ScrollGeometry::new(
            WritingMode::HorizontalTb,
            Direction::Ltr,
            ScrollContainerFacts::new(
                ScrollContainerAxis::from_overflow(Overflow::Clip).unwrap(),
                ScrollContainerAxis::from_overflow(Overflow::Visible).unwrap(),
            ),
            scrollport,
            Some(scrollport),
            overflow,
            range,
            gutters,
        )
        .unwrap_err(),
        ScrollUnsupportedFeature::InvalidScrollGeometry
    );
}

#[test]
fn scroll_geometry_rejects_visible_axis_with_non_zero_range() {
    let scrollport = ScrollRect::new(Point::ZERO, Size::new(80.0, 40.0)).unwrap();
    let overflow = ScrollRect::new(Point::ZERO, Size::new(120.0, 90.0)).unwrap();
    let range = ScrollRange::new(Size::new(0.0, 50.0)).unwrap();
    let gutters = ScrollbarGutterRects::new(None, None);

    assert_eq!(
        ScrollGeometry::new(
            WritingMode::HorizontalTb,
            Direction::Ltr,
            ScrollContainerFacts::new(
                ScrollContainerAxis::from_overflow(Overflow::Scroll).unwrap(),
                ScrollContainerAxis::from_overflow(Overflow::Visible).unwrap(),
            ),
            scrollport,
            None,
            overflow,
            range,
            gutters,
        )
        .unwrap_err(),
        ScrollUnsupportedFeature::InvalidScrollGeometry
    );
}

#[test]
fn phase_one_mixed_axis_boundary_is_root_pre_resolved() {
    assert_eq!(
        ScrollOverflowCouplingPolicy::PHASE_ONE,
        ScrollOverflowCouplingPolicy::RootPreResolved
    );
    assert_eq!(
        ScrollOverflowCouplingPolicy::LayoutOwnedVisibleToAutoCoupling.unsupported_feature(),
        Some(ScrollUnsupportedFeature::LayoutOwnedMixedAxisOverflowCoupling)
    );
}
```

Run:

```sh
cargo test -p surgeist-layout scroll_container_facts -- --nocapture
```

Expected: compile failure naming missing scroll facts/output types.

- [ ] **Step 2: Implement scroll exposure, axes, facts, gutter rects, and geometry output**

Append to `src/scroll.rs`:

```rust
use super::{Direction, Overflow, WritingMode};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScrollOverflowExposure {
    Visible,
    ClipOnly,
    ScrollableClip,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScrollContainerAxis {
    exposure: ScrollOverflowExposure,
}

impl ScrollContainerAxis {
    pub const VISIBLE: Self = Self {
        exposure: ScrollOverflowExposure::Visible,
    };

    #[must_use]
    pub const fn exposure(self) -> ScrollOverflowExposure {
        self.exposure
    }

    #[must_use]
    pub const fn exposes_scroll_range(self) -> bool {
        matches!(self.exposure, ScrollOverflowExposure::ScrollableClip)
    }

    pub const fn from_overflow(
        overflow: Overflow,
    ) -> Result<Self, ScrollUnsupportedFeature> {
        Ok(Self {
            exposure: match overflow {
                Overflow::Visible => ScrollOverflowExposure::Visible,
                Overflow::Clip => ScrollOverflowExposure::ClipOnly,
                Overflow::Hidden | Overflow::Scroll => ScrollOverflowExposure::ScrollableClip,
            },
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScrollContainerFacts {
    x: ScrollContainerAxis,
    y: ScrollContainerAxis,
}

impl ScrollContainerFacts {
    #[must_use]
    pub const fn new(x: ScrollContainerAxis, y: ScrollContainerAxis) -> Self {
        Self { x, y }
    }

    #[must_use]
    pub const fn x(self) -> ScrollContainerAxis {
        self.x
    }

    #[must_use]
    pub const fn y(self) -> ScrollContainerAxis {
        self.y
    }

    #[must_use]
    pub fn accepts_range<S: LayoutScalar>(self, range: ScrollRangeOf<S>) -> bool {
        let maximum = range.maximum_offset();
        (self.x.exposes_scroll_range() || maximum.width == S::ZERO)
            && (self.y.exposes_scroll_range() || maximum.height == S::ZERO)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScrollOverflowCouplingPolicy {
    RootPreResolved,
    LayoutOwnedVisibleToAutoCoupling,
}

impl ScrollOverflowCouplingPolicy {
    pub const PHASE_ONE: Self = Self::RootPreResolved;

    #[must_use]
    pub const fn unsupported_feature(self) -> Option<ScrollUnsupportedFeature> {
        match self {
            Self::RootPreResolved => None,
            Self::LayoutOwnedVisibleToAutoCoupling => {
                Some(ScrollUnsupportedFeature::LayoutOwnedMixedAxisOverflowCoupling)
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollbarGutterRectsOf<S: LayoutScalar = DefaultScalar> {
    horizontal: Option<ScrollRectOf<S>>,
    vertical: Option<ScrollRectOf<S>>,
}

pub type ScrollbarGutterRects = ScrollbarGutterRectsOf<DefaultScalar>;

impl<S: LayoutScalar> ScrollbarGutterRectsOf<S> {
    #[must_use]
    pub const fn new(
        horizontal: Option<ScrollRectOf<S>>,
        vertical: Option<ScrollRectOf<S>>,
    ) -> Self {
        Self { horizontal, vertical }
    }

    #[must_use]
    pub const fn horizontal(self) -> Option<ScrollRectOf<S>> {
        self.horizontal
    }

    #[must_use]
    pub const fn vertical(self) -> Option<ScrollRectOf<S>> {
        self.vertical
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollGeometryOf<S: LayoutScalar = DefaultScalar> {
    writing_mode: WritingMode,
    direction: Direction,
    container: ScrollContainerFacts,
    scrollport: ScrollRectOf<S>,
    overflow_clip: Option<ScrollRectOf<S>>,
    scrollable_overflow: ScrollRectOf<S>,
    range: ScrollRangeOf<S>,
    gutters: ScrollbarGutterRectsOf<S>,
}

pub type ScrollGeometry = ScrollGeometryOf<DefaultScalar>;

impl<S: LayoutScalar> ScrollGeometryOf<S> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        writing_mode: WritingMode,
        direction: Direction,
        container: ScrollContainerFacts,
        scrollport: ScrollRectOf<S>,
        overflow_clip: Option<ScrollRectOf<S>>,
        scrollable_overflow: ScrollRectOf<S>,
        range: ScrollRangeOf<S>,
        gutters: ScrollbarGutterRectsOf<S>,
    ) -> Result<Self, ScrollUnsupportedFeature> {
        if !container.accepts_range(range) {
            return Err(ScrollUnsupportedFeature::InvalidScrollGeometry);
        }

        Ok(Self {
            writing_mode,
            direction,
            container,
            scrollport,
            overflow_clip,
            scrollable_overflow,
            range,
            gutters,
        })
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
    pub const fn container(self) -> ScrollContainerFacts {
        self.container
    }

    #[must_use]
    pub const fn scrollport(self) -> ScrollRectOf<S> {
        self.scrollport
    }

    #[must_use]
    pub const fn overflow_clip(self) -> Option<ScrollRectOf<S>> {
        self.overflow_clip
    }

    #[must_use]
    pub const fn scrollable_overflow(self) -> ScrollRectOf<S> {
        self.scrollable_overflow
    }

    #[must_use]
    pub const fn range(self) -> ScrollRangeOf<S> {
        self.range
    }

    #[must_use]
    pub const fn gutters(self) -> ScrollbarGutterRectsOf<S> {
        self.gutters
    }
}
```

- [ ] **Step 3: Reexport the front-door scroll facts**

In `src/lib.rs`, expand the `pub use scroll::{...};` list to include:

```rust
ScrollContainerAxis, ScrollContainerFacts, ScrollGeometry, ScrollGeometryOf,
ScrollOverflowCouplingPolicy, ScrollOverflowExposure, ScrollbarGutterRects,
ScrollbarGutterRectsOf,
```

- [ ] **Step 4: Run focused checks**

Run:

```sh
cargo test -p surgeist-layout scroll_container_facts -- --nocapture
cargo test -p surgeist-layout scroll_geometry_front_door -- --nocapture
cargo test -p surgeist-layout phase_one_mixed_axis_boundary -- --nocapture
cargo fmt --check
git diff --check
```

Expected: all pass.

## Task 3: Add Unsupported Diagnostics Coverage And Public Contract Tests

**Files:**

- Modify: `src/scroll.rs`
- Modify: `src/scroll_tests.rs`
- Modify: `src/contract_tests.rs`

- [ ] **Step 1: Add tests for unsupported Phase 1 features**

Append to `src/scroll_tests.rs`:

```rust
#[test]
fn phase_one_reports_deferred_scroll_features_explicitly() {
    let deferred = [
        ScrollUnsupportedFeature::OverflowAuto,
        ScrollUnsupportedFeature::OverflowClipMargin,
        ScrollUnsupportedFeature::ScrollbarGutterStable,
        ScrollUnsupportedFeature::ScrollbarGutterBothEdges,
        ScrollUnsupportedFeature::ScrollPadding,
        ScrollUnsupportedFeature::ScrollMargin,
        ScrollUnsupportedFeature::ScrollSnap,
    ];

    for feature in deferred {
        assert!(feature.is_phase_one_deferred());
    }
}
```

Add this test to `src/contract_tests.rs`:

```rust
#[test]
fn scroll_geometry_core_is_scalar_generic() {
    fn assert_scalar<S: crate::LayoutScalar>() {
        let range = crate::ScrollRangeOf::<S>::new(crate::Size::new(S::ZERO, S::ZERO)).unwrap();
        assert_eq!(range.maximum_offset(), crate::Size::ZERO);
    }

    assert_scalar::<f32>();
    assert_scalar::<f64>();
}
```

Run:

```sh
cargo test -p surgeist-layout phase_one_reports_deferred_scroll_features -- --nocapture
```

Expected: compile failure naming missing `is_phase_one_deferred`.

- [ ] **Step 2: Implement diagnostic helper**

Add to the existing `impl ScrollUnsupportedFeature` in `src/scroll.rs`, or create that impl:

```rust
impl ScrollUnsupportedFeature {
    #[must_use]
    pub const fn is_phase_one_deferred(self) -> bool {
        matches!(
            self,
            Self::OverflowAuto
                | Self::OverflowClipMargin
                | Self::ScrollbarGutterStable
                | Self::ScrollbarGutterBothEdges
                | Self::ScrollPadding
                | Self::ScrollMargin
                | Self::ScrollSnap
                | Self::LayoutOwnedMixedAxisOverflowCoupling
        )
    }
}
```

- [ ] **Step 3: Run focused and crate checks**

Run:

```sh
cargo test -p surgeist-layout phase_one_reports_deferred_scroll_features -- --nocapture
cargo test -p surgeist-layout scroll_geometry_core_is_scalar_generic -- --nocapture
cargo test -p surgeist-layout scroll_ -- --nocapture
cargo fmt --check
git diff --check
```

Expected: all pass.

## Review And Commit Gate

After each task:

1. Worker reports changed files, tests run, and `git status --short --branch`.
2. A separate reviewer inspects the scoped task changes.
3. Coordinator reconciles findings.
4. Coordinator runs the focused checks.
5. Coordinator commits only after worker/reviewer cycle is clean.

Final implementation gate for this plan:

```sh
cargo test -p surgeist-layout
cargo clippy -p surgeist-layout --all-targets -- -D warnings
cargo fmt --check
git diff --check
git status --short --branch
```

Then assign a final clean-context holistic reviewer to inspect the full Phase 1
implementation against this plan, the sequence, the support matrix, crate
boundary, tests, and the modeling guide. Commit the final implementation only
after that review is clean.
