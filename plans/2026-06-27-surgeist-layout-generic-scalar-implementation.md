# Surgeist Layout Generic Scalar Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `surgeist-layout` generic over a coherent layout scalar precision so the crate can be used with either `f32` or `f64` without mixing precisions inside one layout computation.

**Architecture:** Introduce a sealed `LayoutScalar` contract implemented for `f32` and `f64`, then thread that scalar through public value, input, output, cache, and algorithm types as a crate-wide generic parameter. Keep ergonomic `f32` aliases for the current default while adding explicit `f64` aliases and verification lanes, and coordinate sibling crate adapter updates through the root or owning crate projects.

**Tech Stack:** Rust 2024, standard library numeric primitives, crate-local public API generator, existing browser parity XML corpus, existing `surgeist-style` dev adapter tests, and crate-local worker/reviewer process from `AGENTS.md`.

---

## Requirements

- Support at least `f32` and `f64` as layout scalar precisions.
- A single layout run must use one scalar type end-to-end.
- Do not duplicate retained tree ownership in this crate; `surgeist-retained` remains the tree/identity owner.
- Preserve the current `f32` path as the default ergonomic path unless the root coordinator chooses otherwise.
- Expose the scalar choice as an intentional public contract, not as an incidental `pub type Scalar = f32`.
- Keep invalid numeric states at least as constrained as today: non-finite aspect ratios still fail construction, missing calc expressions remain typed failures, and placement/span validation is unchanged.
- Keep browser parity XML generation and comparison on the default `f32` lane initially, then add focused `f64` unit/integration coverage around the scalar-generic API.
- Record cross-crate requirements instead of editing sibling crates from this repo.

## Non-Goals

- Do not make one layout tree facade that owns retained identity.
- Do not implement mixed-precision layout inside a single tree.
- Do not introduce a third scalar backend such as fixed point in this plan.
- Do not change the default browser parity fixture format unless a later generator plan requires it.
- Do not optimize memory layout beyond preserving the `f32` default path and confirming that `f64` increases scalar-bearing type sizes as expected.

## File Structure

- Modify `src/scalar.rs`: new sealed `LayoutScalar` trait and helper conversion functions.
- Modify `src/lib.rs`: expose `LayoutScalar`, `DefaultScalar`, `Scalar`, and generic/default aliases from a single front door.
- Modify `src/geometry.rs`: make `Point`, `Size`, and `Edges` generic over scalar-compatible values without hard-coded `Scalar` impls.
- Modify `src/value.rs`: make layout numeric values generic: `Available`, `Length`, `LengthAuto`, `Dimension`, calc expressions/resolution/store, aspect ratio, and track sizing.
- Modify `src/node_input.rs`: introduce `NodeInputOf<S>` and make the existing `NodeInput` alias default to `DefaultScalar`.
- Modify `src/output.rs`: introduce `ComputeInputOf<S>`, `ComputeOutputOf<S>`, `NodeOutputOf<S>`, `BaselinesOf<S>`, and default aliases.
- Modify `src/traits.rs`: make `Compute`, `Traverse`, `CacheAccess`, and `compute_cached` scalar-aware without allowing one tree view to return mixed scalar inputs and outputs.
- Modify `src/cache.rs`: make cache entries and keys scalar-aware where they contain scalar-valued context.
- Modify `src/block.rs`, `src/flex.rs`, `src/inline.rs`, `src/grid/*.rs`, and `src/compute.rs`: thread `S: LayoutScalar` through algorithm internals.
- Modify `tests/layout/unit/contract.rs`: replace the single-precision contract with default-scalar and f64-lane contract tests.
- Modify `tests/support/oracle_tree.rs` and `tests/support/grid_layout_comparison.rs`: make test trees generic where they use public layout values directly.
- Modify `tests/layout/browser_parity/support.rs`: keep XML parsing/comparison on `Scalar` default while isolating conversions at the support boundary.
- Modify `tests/bin/surgeist-layout-generate/generator.rs`: keep browser numeric serialization f32-compatible and document the intentional generator precision boundary.
- Modify `README.md`: document default scalar precision, `f64` support, and root/sibling coordination.
- Modify `api/public-api.txt`: regenerate after source changes.
- Create `plans/2026-06-27-surgeist-layout-generic-scalar-cross-crate-ledger.md`: record required changes in `surgeist-style`, `surgeist-css`, root `surgeist`, and any other crate discovered during implementation.

## Public API Target

The source-derived public API should make the default and generic paths visible:

```rust
pub trait LayoutScalar:
    private::Sealed
    + Copy
    + Clone
    + core::fmt::Debug
    + Default
    + PartialEq
    + PartialOrd
    + core::ops::Add<Output = Self>
    + core::ops::Sub<Output = Self>
    + core::ops::Mul<Output = Self>
    + core::ops::Div<Output = Self>
    + core::ops::Neg<Output = Self>
    + 'static
{
    const ZERO: Self;
    const ONE: Self;
    const INFINITY: Self;
    const NAN: Self;
    const EPSILON: Self;

    fn from_f32(value: f32) -> Self;
    fn from_f64(value: f64) -> Self;
    fn from_usize(value: usize) -> Self;
    fn abs(self) -> Self;
    fn min(self, other: Self) -> Self;
    fn max(self, other: Self) -> Self;
    fn floor(self) -> Self;
    fn ceil(self) -> Self;
    fn round(self) -> Self;
    fn is_finite(self) -> bool;
    fn floor_to_usize_saturating(self) -> usize;
    fn to_f64(self) -> f64;
}

pub type DefaultScalar = f32;
pub type Scalar = DefaultScalar;

pub type NodeInput = NodeInputOf<DefaultScalar>;
pub type ComputeInput = ComputeInputOf<DefaultScalar>;
pub type ComputeOutput = ComputeOutputOf<DefaultScalar>;
pub type NodeOutput = NodeOutputOf<DefaultScalar>;
```

Generic public structs should use `Of` suffix names when the existing unsuffixed name remains as a default alias:

```rust
pub enum LengthOf<S: LayoutScalar> {
    Px(S),
    Percent(S),
    Calc(CalcId),
    Normal,
}

pub type Length = LengthOf<DefaultScalar>;
```

Geometry is intentionally different from scalar-owned layout values. `Point`,
`Size`, and `Edges` remain generic containers over any component type because
the existing API uses forms such as `Size<Option<S>>`, `Edges<LengthOf<S>>`,
and `Point<Overflow>`. Scalar-owned layout values, inputs, outputs, reports,
and algorithm contracts use the `Of<S>` generic type plus an unsuffixed default
alias.

## Compatibility Classification

This is a pre-release breaking public API change with compatibility aliases.
Workers should preserve the existing ergonomic default path through unsuffixed
aliases such as `Scalar`, `NodeInput`, `ComputeInput`, `Length`, `Dimension`,
and `TrackComponent`, all using `DefaultScalar = f32`. Intentional breaking
changes are limited to public type shapes that must become scalar-generic, such
as introducing `*Of<S>` forms, generic calc resolver contracts, and scalar-aware
tree/cache traits.

Do not introduce unrelated traversal, cache, or retained-tree API breakage while
implementing scalar precision. Existing trait method names and storage-shape
flexibility should remain unless a reviewer explicitly accepts a narrower
follow-up change. In particular, `surgeist-layout` must not require retained or
adapter tree owners to expose children as a slice if they currently satisfy the
iterator-based traversal contract.

## Cross-Crate Coordination

Implementation workers must not edit sibling crates from this repo. When a sibling adapter breaks:

1. Record the affected crate, public API, observed compiler error, and required owning change in `plans/2026-06-27-surgeist-layout-generic-scalar-cross-crate-ledger.md`.
2. Continue layout-local work where possible.
3. Ask the root coordinator to sequence sibling crate implementation plans.

Expected sibling impact:

- `surgeist-style`: layout adapter currently constructs `NodeInput`, `Length`, `Dimension`, `TrackComponent`, `AspectRatio`, grid placements, and calc resolver values through the default `Scalar` path. It likely needs explicit default aliases first, then may later expose generic style resolution.
- `surgeist-css`: CSS parser/lowering likely owns numeric parsing and may need a precision decision before it can lower directly into non-default layout scalar values.
- root `surgeist`: facade should choose the default scalar path or expose precision-specific aliases.
- `surgeist-text`: measurement APIs that feed layout may need matching scalar precision or explicit conversions.
- `surgeist-render`: output coordinates may need a scalar conversion strategy when render backends prefer `f32`.

---

## Task 1: Add The Scalar Contract

**Files:**
- Create: `src/scalar.rs`
- Modify: `src/lib.rs`
- Test: `tests/layout/unit/contract.rs`

- [ ] **Step 1: Write failing scalar contract tests**

Add tests that prove the default path is still `f32`, and that the crate exposes an accepted `f64` scalar contract:

```rust
#[test]
fn default_scalar_remains_single_precision() {
    assert_eq!(
        std::mem::size_of::<surgeist_layout::DefaultScalar>(),
        std::mem::size_of::<f32>()
    );
    assert_eq!(
        std::mem::size_of::<surgeist_layout::Scalar>(),
        std::mem::size_of::<f32>()
    );
}

#[test]
fn layout_scalar_supports_f32_and_f64() {
    fn assert_scalar<S: surgeist_layout::LayoutScalar>() {
        assert!(S::ONE.is_finite());
        assert_eq!(S::ZERO + S::ONE, S::ONE);
        assert_eq!(S::from_usize(3), S::ONE + S::ONE + S::ONE);
        assert_eq!(S::from_f64(-2.5).abs(), S::from_f64(2.5));
        assert_eq!(S::from_f64(4.75).floor_to_usize_saturating(), 4);
    }

    assert_scalar::<f32>();
    assert_scalar::<f64>();
}
```

- [ ] **Step 2: Run the focused test and confirm it fails**

Run:

```sh
cargo test -p surgeist-layout --test layout layout::contract::layout_scalar_supports_f32_and_f64 -- --nocapture
```

Expected: fail because `LayoutScalar` and `DefaultScalar` do not exist yet.

- [ ] **Step 3: Add `src/scalar.rs`**

Create the sealed scalar trait with exactly these public operations at first:

```rust
use core::fmt::Debug;
use core::ops::{Add, Div, Mul, Neg, Sub};

pub trait LayoutScalar:
    private::Sealed
    + Copy
    + Clone
    + Debug
    + Default
    + PartialEq
    + PartialOrd
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Neg<Output = Self>
    + 'static
{
    const ZERO: Self;
    const ONE: Self;
    const INFINITY: Self;
    const NAN: Self;
    const EPSILON: Self;

    fn from_f32(value: f32) -> Self;
    fn from_f64(value: f64) -> Self;
    fn from_usize(value: usize) -> Self;
    fn abs(self) -> Self;
    fn min(self, other: Self) -> Self;
    fn max(self, other: Self) -> Self;
    fn floor(self) -> Self;
    fn ceil(self) -> Self;
    fn round(self) -> Self;
    fn is_finite(self) -> bool;
    fn floor_to_usize_saturating(self) -> usize;
    fn to_f64(self) -> f64;
}

macro_rules! impl_layout_scalar {
    ($ty:ty) => {
        impl private::Sealed for $ty {}

        impl LayoutScalar for $ty {
            const ZERO: Self = 0.0;
            const ONE: Self = 1.0;
            const INFINITY: Self = <$ty>::INFINITY;
            const NAN: Self = <$ty>::NAN;
            const EPSILON: Self = <$ty>::EPSILON;

            fn from_f32(value: f32) -> Self {
                value as Self
            }

            fn from_f64(value: f64) -> Self {
                value as Self
            }

            fn from_usize(value: usize) -> Self {
                value as Self
            }

            fn abs(self) -> Self {
                <$ty>::abs(self)
            }

            fn min(self, other: Self) -> Self {
                <$ty>::min(self, other)
            }

            fn max(self, other: Self) -> Self {
                <$ty>::max(self, other)
            }

            fn floor(self) -> Self {
                <$ty>::floor(self)
            }

            fn ceil(self) -> Self {
                <$ty>::ceil(self)
            }

            fn round(self) -> Self {
                <$ty>::round(self)
            }

            fn is_finite(self) -> bool {
                <$ty>::is_finite(self)
            }

            fn floor_to_usize_saturating(self) -> usize {
                if !<$ty>::is_finite(self) || self <= 0.0 {
                    0
                } else if self >= usize::MAX as Self {
                    usize::MAX
                } else {
                    <$ty>::floor(self) as usize
                }
            }

            fn to_f64(self) -> f64 {
                self as f64
            }
        }
    };
}

impl_layout_scalar!(f32);
impl_layout_scalar!(f64);

mod private {
    pub trait Sealed {}
}
```

- [ ] **Step 4: Export the scalar contract**

Modify `src/lib.rs`:

```rust
mod scalar;

pub use scalar::LayoutScalar;

pub type DefaultScalar = f32;
pub type Scalar = DefaultScalar;
```

- [ ] **Step 5: Run the focused scalar contract test**

Run:

```sh
cargo test -p surgeist-layout --test layout layout::contract::layout_scalar_supports_f32_and_f64 -- --nocapture
```

Expected: pass.

- [ ] **Step 6: Commit**

```sh
git add src/lib.rs src/scalar.rs tests/layout/unit/contract.rs
git commit -m "Add layout scalar contract"
```

---

## Task 2: Keep Geometry Generic Containers Scalar-Friendly

**Files:**
- Modify: `src/geometry.rs`
- Modify: `src/lib.rs`
- Test: `tests/layout/unit/contract.rs`

- [ ] **Step 1: Write geometry scalar container tests**

Add:

```rust
#[test]
fn geometry_supports_default_and_f64_scalars() {
    let default_size = surgeist_layout::Size::new(2.0, 3.0);
    assert_eq!(default_size.width, 2.0);

    let f64_size = surgeist_layout::Size::<f64>::new(2.0_f64, 3.0_f64);
    assert_eq!(f64_size.height, 3.0_f64);

    let f64_edges = surgeist_layout::Edges::<f64>::new(1.0, 2.0, 3.0, 4.0);
    assert_eq!(f64_edges.horizontal_sum(), 6.0_f64);
    assert_eq!(f64_edges.vertical_sum(), 4.0_f64);
}
```

- [ ] **Step 2: Run and confirm failure**

Run:

```sh
cargo test -p surgeist-layout --test layout layout::contract::geometry_supports_default_and_f64_scalars -- --nocapture
```

Expected: fail until scalar-only geometry impls are moved behind `LayoutScalar`
where needed.

- [ ] **Step 3: Preserve existing generic geometry names**

Keep `Size<T>`, `Point<T>`, and `Edges<T>` as the real public structs. Do not
introduce `SizeOf`, `PointOf`, or `EdgesOf`; geometry is a generic container
layer, not a scalar-owned layout model. Preserve current non-scalar uses such
as `Size<Option<S>>`, `Edges<LengthOf<S>>`, and `Point<Overflow>`.

```rust
pub struct Size<T = crate::Scalar> {
    pub width: T,
    pub height: T,
}

pub struct Point<T = crate::Scalar> {
    pub x: T,
    pub y: T,
}

pub struct Edges<T = crate::Scalar> {
    pub top: T,
    pub left: T,
    pub bottom: T,
    pub right: T,
}
```

- [ ] **Step 4: Move scalar-only impls behind `LayoutScalar`**

Any impl that currently assumes arithmetic on the component type should become:

```rust
impl<S: crate::LayoutScalar> Edges<S> {
    pub const ZERO: Self = Self {
        top: S::ZERO,
        left: S::ZERO,
        bottom: S::ZERO,
        right: S::ZERO,
    };

    pub fn horizontal_sum(self) -> S {
        self.left + self.right
    }

    pub fn vertical_sum(self) -> S {
        self.top + self.bottom
    }
}
```

Keep the existing generic-container behavior for geometry in the public API
notes. Do not make `Size`, `Point`, or `Edges` default-only aliases; they are
generic containers, while scalar-owned layout models use `*Of<S>`.

- [ ] **Step 5: Run geometry and full contract tests**

Run:

```sh
cargo test -p surgeist-layout --test layout layout::contract -- --nocapture
```

Expected: pass.

- [ ] **Step 6: Commit**

```sh
git add src/geometry.rs src/lib.rs tests/layout/unit/contract.rs
git commit -m "Keep geometry scalar-friendly"
```

---

## Task 3: Make Value Types Scalar-Generic

**Files:**
- Modify: `src/value.rs`
- Modify: `src/lib.rs`
- Test: `tests/layout/unit/contract.rs`
- Test: `src/tests.rs`

- [ ] **Step 1: Add failing value generic tests**

Add:

```rust
#[test]
fn value_types_support_f64_scalar_lane() {
    let length = surgeist_layout::LengthOf::<f64>::percent(0.25);
    assert_eq!(length.resolve(400.0), 100.0);

    let dimension = surgeist_layout::DimensionOf::<f64>::px(42.5);
    assert_eq!(dimension.resolve(1000.0), Some(42.5));

    let ratio = surgeist_layout::AspectRatioOf::<f64>::new(16.0 / 9.0)
        .expect("positive finite f64 aspect ratio should be accepted");
    assert_eq!(ratio.get(), 16.0 / 9.0);

    assert!(surgeist_layout::AspectRatioOf::<f64>::new(f64::INFINITY).is_none());
}
```

- [ ] **Step 2: Run and confirm failure**

Run:

```sh
cargo test -p surgeist-layout --test layout layout::contract::value_types_support_f64_scalar_lane -- --nocapture
```

Expected: fail because `LengthOf`, `DimensionOf`, and `AspectRatioOf` do not exist.

- [ ] **Step 3: Genericize scalar-bearing values**

Rename scalar-bearing public values to `Of<S>` forms with default aliases:

```rust
pub enum LengthOf<S: LayoutScalar = DefaultScalar> {
    Px(S),
    Percent(S),
    Calc(CalcId),
    Normal,
}

pub type Length = LengthOf<DefaultScalar>;

pub enum LengthAutoOf<S: LayoutScalar = DefaultScalar> {
    Px(S),
    Percent(S),
    Calc(CalcId),
    Auto,
}

pub type LengthAuto = LengthAutoOf<DefaultScalar>;

pub enum DimensionOf<S: LayoutScalar = DefaultScalar> {
    Auto,
    Px(S),
    Percent(S),
    Calc(CalcId),
    Fr(S),
    MinContent,
    MaxContent,
    FitContent,
}

pub type Dimension = DimensionOf<DefaultScalar>;
```

Apply the same pattern to `Available`, `CalcTerm`, `CalcExpression`, `CalcResolution`, `ResolvedLengthAuto`, `AspectRatio`, `MinTrackSizing`, `MaxTrackSizing`, `TrackSizing`, `TrackComponent`, and any track/calc helper that stores or returns `Scalar`.

- [ ] **Step 4: Convert numeric literals and casts**

Replace scalar literals inside generic impls:

```rust
let zero = S::ZERO;
let one = S::ONE;
let two = S::from_usize(2);
let percent = value * basis;
```

Replace casts like `count as Scalar` with:

```rust
S::from_usize(count)
```

Keep public constructors accepting `S`, not `f32`.

When current code uses `.sum::<Scalar>()`, rewrite it as:

```rust
values.iter().copied().fold(S::ZERO, |sum, value| sum + value)
```

When current code uses `+=` or `-=`, prefer reassignment:

```rust
total = total + value;
remaining = remaining - value;
```

Do not add `core::iter::Sum`, `AddAssign`, or `SubAssign` to `LayoutScalar`
unless a reviewer confirms the extra public trait bounds are preferable to
local rewrites.

- [ ] **Step 5: Make calc resolver scalar-aware**

Change:

```rust
pub trait CalcResolver {
    fn calc_generation(&self) -> CalcGeneration;
    fn calc_depends_on_basis(&self, id: CalcId) -> bool;
    fn resolve_calc(&self, id: CalcId, basis: Option<Scalar>) -> CalcResolution;
    fn calc_percent_fraction(&self, id: CalcId) -> Option<Scalar>;
}
```

to:

```rust
pub trait CalcResolver<S: LayoutScalar = DefaultScalar> {
    fn calc_generation(&self) -> CalcGeneration;
    fn calc_depends_on_basis(&self, id: CalcId) -> bool;
    fn resolve_calc(&self, id: CalcId, basis: Option<S>) -> CalcResolutionOf<S>;
    fn calc_percent_fraction(&self, id: CalcId) -> Option<S>;
}
```

Preserve `calc_generation` and `calc_depends_on_basis` exactly as part of the
generic resolver contract. Cache invalidation and basis-dependent resolution
already rely on those methods in the current source; scalar genericity must not
drop or replace them.

Make `LayoutCalcStoreOf<S>` generic and keep:

```rust
pub type LayoutCalcStore = LayoutCalcStoreOf<DefaultScalar>;
```

Add a f64 calc resolver test:

```rust
#[test]
fn f64_calc_resolution_preserves_large_coordinate_precision() {
    let mut store = surgeist_layout::LayoutCalcStoreOf::<f64>::new();
    let id = store.push(surgeist_layout::CalcExpressionOf::sum(vec![
        surgeist_layout::CalcTermOf::px(16_777_217.0),
        surgeist_layout::CalcTermOf::percent(0.5),
    ]));

    let resolution = store.resolve_calc(id, Some(21.0));
    assert_eq!(resolution.value, Some(16_777_227.5));
    assert!(resolution.depends_on_basis);
}
```

- [ ] **Step 6: Run value-focused tests**

Run:

```sh
cargo test -p surgeist-layout --lib tests::layout_calc_store_resolves_px_and_percent_terms -- --nocapture
cargo test -p surgeist-layout --test layout layout::contract::value_types_support_f64_scalar_lane -- --nocapture
cargo test -p surgeist-layout --test layout layout::contract::f64_calc_resolution_preserves_large_coordinate_precision -- --nocapture
```

Expected: pass.

- [ ] **Step 7: Commit**

```sh
git add src/value.rs src/lib.rs src/tests.rs tests/layout/unit/contract.rs
git commit -m "Make layout values generic over scalar precision"
```

---

## Task 4: Make Node Input And Output Scalar-Generic

**Files:**
- Modify: `src/node_input.rs`
- Modify: `src/output.rs`
- Modify: `src/lib.rs`
- Test: `tests/layout/unit/contract.rs`

- [ ] **Step 1: Add failing f64 input/output tests**

Add:

```rust
#[test]
fn node_input_and_output_support_f64_scalar_lane() {
    let input = surgeist_layout::NodeInputOf::<f64> {
        size: surgeist_layout::Size::new(
            surgeist_layout::DimensionOf::px(123.5),
            surgeist_layout::DimensionOf::percent(0.25),
        ),
        margin: surgeist_layout::Edges::all(surgeist_layout::LengthAutoOf::px(2.5)),
        flex_grow: 1.0,
        ..surgeist_layout::NodeInputOf::<f64>::default()
    };

    assert_eq!(input.size.width.resolve(1000.0), Some(123.5));
    assert_eq!(input.size.height.resolve(400.0), Some(100.0));

    let output = surgeist_layout::NodeOutputOf::<f64> {
        size: surgeist_layout::Size::new(20.0, 10.0),
        ..surgeist_layout::NodeOutputOf::<f64>::default()
    };

    assert_eq!(output.size.width, 20.0);
}
```

- [ ] **Step 2: Run and confirm failure**

Run:

```sh
cargo test -p surgeist-layout --test layout layout::contract::node_input_and_output_support_f64_scalar_lane -- --nocapture
```

Expected: fail because `NodeInputOf` and `NodeOutputOf` do not exist.

- [ ] **Step 3: Genericize `NodeInput`**

Rename:

```rust
pub struct NodeInput {
```

to:

```rust
pub struct NodeInputOf<S: LayoutScalar = DefaultScalar> {
```

Change scalar-bearing fields:

```rust
pub overflow: Point<Overflow>,
pub scrollbar_width: S,
pub inset: Edges<LengthAutoOf<S>>,
pub size: Size<DimensionOf<S>>,
pub min_size: Size<DimensionOf<S>>,
pub max_size: Size<DimensionOf<S>>,
pub aspect_ratio: Option<AspectRatioOf<S>>,
pub margin: Edges<LengthAutoOf<S>>,
pub padding: Edges<LengthOf<S>>,
pub border: Edges<LengthOf<S>>,
pub gap: Size<LengthOf<S>>,
pub flex_basis: DimensionOf<S>,
pub flex_grow: S,
pub flex_shrink: S,
pub grid_template_columns: Vec<TrackComponentOf<S>>,
pub grid_template_rows: Vec<TrackComponentOf<S>>,
pub grid_auto_columns: Vec<TrackComponentOf<S>>,
pub grid_auto_rows: Vec<TrackComponentOf<S>>,
pub grid_flow_tolerance: GridFlowToleranceOf<S>,
```

Keep:

```rust
pub type NodeInput = NodeInputOf<DefaultScalar>;
```

Make scalar-taking helper methods on non-scalar enums generic instead of
narrowing through the default alias:

```rust
impl AlignItems {
    pub fn safe_fallback<S: LayoutScalar>(self, free_space: S) -> Self {
        if free_space < S::ZERO {
            // existing fallback logic
        } else {
            self.unsafe_position()
        }
    }
}

impl AlignContent {
    pub fn safe_fallback<S: LayoutScalar>(self, free_space: S) -> Self {
        if free_space < S::ZERO {
            // existing fallback logic
        } else {
            self.unsafe_position()
        }
    }
}
```

- [ ] **Step 4: Genericize output types**

Apply the same pattern in `src/output.rs`:

```rust
pub struct ComputeInputOf<S: LayoutScalar = DefaultScalar> {
    pub run_mode: RunMode,
    pub sizing_mode: SizingMode,
    pub axis: RequestedAxis,
    pub known: Size<Option<S>>,
    pub parent: Size<Option<S>>,
    pub available: Size<AvailableOf<S>>,
}

pub type ComputeInput = ComputeInputOf<DefaultScalar>;

pub struct ComputeOutputOf<S: LayoutScalar = DefaultScalar> {
    pub size: Size<S>,
    pub content_size: Size<S>,
    pub first_baselines: Point<Option<S>>,
    pub last_baselines: Point<Option<S>>,
}

pub type ComputeOutput = ComputeOutputOf<DefaultScalar>;
```

Repeat for `NodeOutput`, `Baselines`, and `CollapsibleMargin`.

Preserve `ComputeInput::HIDDEN` as a default alias constant and add an equivalent
associated constant on `ComputeInputOf<S>`:

```rust
impl<S: LayoutScalar> ComputeInputOf<S> {
    pub const HIDDEN: Self = Self {
        run_mode: RunMode::PerformHiddenLayout,
        sizing_mode: SizingMode::InherentSize,
        axis: RequestedAxis::Both,
        known: Size::NONE,
        parent: Size::NONE,
        available: Size::splat(AvailableOf::MAX_CONTENT),
    };
}
```

- [ ] **Step 5: Run focused default and f64 tests**

Run:

```sh
cargo test -p surgeist-layout --test layout layout::contract -- --nocapture
```

Expected: pass.

- [ ] **Step 6: Commit**

```sh
git add src/node_input.rs src/output.rs src/lib.rs tests/layout/unit/contract.rs
git commit -m "Make layout inputs and outputs scalar generic"
```

---

## Task 5: Make Tree Traits, Cache, And Compute Dispatch Scalar-Aware

**Files:**
- Modify: `src/traits.rs`
- Modify: `src/cache.rs`
- Modify: `src/compute.rs`
- Test: `tests/support/oracle_tree.rs`
- Test: `tests/layout/unit/cache.rs`
- Test: `tests/layout/unit/root.rs`

- [ ] **Step 1: Add a generic test tree smoke test**

In `tests/support/oracle_tree.rs`, introduce:

```rust
pub type OracleTree = OracleTreeOf<surgeist_layout::DefaultScalar>;

pub struct OracleTreeOf<S: surgeist_layout::LayoutScalar = surgeist_layout::DefaultScalar> {
    styles: HashMap<u32, surgeist_layout::NodeInputOf<S>>,
    outputs: HashMap<u32, surgeist_layout::NodeOutputOf<S>>,
    calcs: surgeist_layout::LayoutCalcStoreOf<S>,
    // Keep existing non-scalar fields unchanged.
}
```

Add a builder and resolver hook:

```rust
impl<S: surgeist_layout::LayoutScalar> OracleTreeOf<S> {
    pub fn calcs(mut self, calcs: surgeist_layout::LayoutCalcStoreOf<S>) -> Self {
        self.calcs = calcs;
        self
    }
}
```

In the `Compute for OracleTreeOf<S>` implementation:

```rust
fn calc_resolver(&self) -> &dyn surgeist_layout::CalcResolver<S> {
    &self.calcs
}
```

In `tests/layout/unit/root.rs`, add:

```rust
#[test]
fn f64_tree_can_run_root_layout_smoke_test() {
    let mut tree = crate::support::oracle_tree::OracleTreeOf::<f64>::new()
        .style(
            0,
            NodeInputOf::<f64> {
                display: Display::Block,
                size: Size::new(DimensionOf::px(100.0), DimensionOf::px(50.0)),
                ..NodeInputOf::<f64>::default()
            },
        );

    surgeist_layout::compute_root(
        &mut tree,
        0,
        Size::new(AvailableOf::definite(100.0), AvailableOf::definite(50.0)),
    );

    assert_eq!(tree.output(0).size, Size::new(100.0, 50.0));
}
```

- [ ] **Step 2: Run and confirm failure**

Run:

```sh
cargo test -p surgeist-layout --test layout layout::root::f64_tree_can_run_root_layout_smoke_test -- --nocapture
```

Expected: fail because the tree traits and compute dispatch are not scalar-aware.

- [ ] **Step 3: Add scalar associated type to tree and cache traits**

Change `Compute`, `Round`, and `Traverse` to share one scalar type, and make
`CacheAccess` scalar-aware while preserving its current cache-only shape:

```rust
pub trait Traverse {
    type Node: Copy + Eq;
    type Scalar: LayoutScalar;
    type Children<'a>: Iterator<Item = Self::Node>
    where
        Self: 'a;

    fn children(&self, node: Self::Node) -> Self::Children<'_>;
    fn child_count(&self, node: Self::Node) -> usize;
    fn child(&self, node: Self::Node, index: usize) -> Self::Node;
}

pub trait Compute: Traverse {
    fn node_input(&self, node: Self::Node) -> &NodeInputOf<Self::Scalar>;
    fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutputOf<Self::Scalar>);
    fn compute_child(
        &mut self,
        node: Self::Node,
        input: ComputeInputOf<Self::Scalar>,
    ) -> ComputeOutputOf<Self::Scalar>;

    fn calc_resolver(&self) -> &dyn CalcResolver<Self::Scalar>;
}

pub trait Round: Traverse {
    fn unrounded(&self, node: Self::Node) -> NodeOutputOf<Self::Scalar>;
    fn set_final(&mut self, node: Self::Node, layout: NodeOutputOf<Self::Scalar>);
}

pub trait CacheAccess {
    type Node: Copy + Eq;
    type Scalar: LayoutScalar;

    fn cache_context(&self) -> CacheKeyContext;
    fn cache_get(
        &self,
        node: Self::Node,
        input: &ComputeInputOf<Self::Scalar>,
        context: CacheKeyContext,
    ) -> Option<ComputeOutputOf<Self::Scalar>>;
    fn cache_store(
        &mut self,
        node: Self::Node,
        input: &ComputeInputOf<Self::Scalar>,
        context: CacheKeyContext,
        output: ComputeOutputOf<Self::Scalar>,
    );
    fn cache_clear(&mut self, node: Self::Node);
}
```

Set `type Scalar = DefaultScalar;` explicitly in existing default tree implementations. Do not use associated type defaults; keep this plan compatible with stable Rust. Existing `Compute::calc_resolver` default behavior should remain available through a default method returning `&NoCalcResolverOf<Self::Scalar>` when that is representable without allocation; otherwise require test trees to implement the method explicitly.

When an algorithm requires both compute and cache access, tie the cache scalar
and node type to the traversal scalar and node type at the call site:

```rust
where
    T: Compute + CacheAccess<
        Node = <T as Traverse>::Node,
        Scalar = <T as Traverse>::Scalar,
    >,
```

Do not make `CacheAccess` inherit from `Traverse`; it is currently a cache-only
trait, and scalar genericity should not force retained or adapter owners into a
new traversal/storage shape.

- [ ] **Step 4: Add f64 calc, cache, and round tests**

In `tests/layout/unit/root.rs`, add:

```rust
#[test]
fn f64_round_layout_preserves_large_coordinates() {
    let large = 16_777_217.25_f64;
    let mut tree = OracleTreeOf::<f64>::new()
        .style(0, NodeInputOf::<f64>::default())
        .unrounded(
            0,
            NodeOutputOf::<f64> {
                location: Point::new(large, large + 0.5),
                size: Size::new(10.5, 20.25),
                ..NodeOutputOf::<f64>::default()
            },
        );

    round_layout(&mut tree, 0);

    let final_layout = tree.output(0);
    assert_eq!(final_layout.location.x, large.round());
    assert_eq!(final_layout.location.y, (large + 0.5).round());
}
```

In `tests/layout/unit/cache.rs`, add:

```rust
#[test]
fn f64_cache_context_remains_tree_context_only() {
    let context = CacheKeyContext::new(CalcGeneration::static_no_calc());

    assert_eq!(context.calc_generation(), CalcGeneration::static_no_calc());
}
```

Add a cache roundtrip test that would fail if `available` narrows through f32:

```rust
#[test]
fn f64_cache_key_distinguishes_available_values_that_collide_as_f32() {
    let mut cache = CacheOf::<f64>::new();
    let context = CacheKeyContext::static_no_calc();
    let base = ComputeInputOf {
        run_mode: RunMode::ComputeSize,
        sizing_mode: SizingMode::ContentSize,
        axis: RequestedAxis::Horizontal,
        known: Size::NONE,
        parent: Size::NONE,
        available: Size::new(
            AvailableOf::definite(16_777_216.0),
            AvailableOf::MAX_CONTENT,
        ),
    };
    let nearby = ComputeInputOf {
        available: Size::new(
            AvailableOf::definite(16_777_217.0),
            AvailableOf::MAX_CONTENT,
        ),
        ..base
    };

    let output = ComputeOutputOf::<f64>::from_outer_size(Size::new(1.0, 1.0));
    cache.store_with_context(&base, context, output);

    assert_eq!(cache.get_with_context(&base, context), Some(output));
    assert_eq!(cache.get_with_context(&nearby, context), None);
}
```

Add a memory-profile contract test that proves the default f32 lane keeps
representative scalar-bearing types smaller than the f64 lane:

```rust
#[test]
fn f32_default_keeps_representative_layout_types_smaller_than_f64_lane() {
    assert!(
        std::mem::size_of::<surgeist_layout::ComputeOutput>()
            < std::mem::size_of::<surgeist_layout::ComputeOutputOf<f64>>()
    );
    assert!(
        std::mem::size_of::<surgeist_layout::NodeOutput>()
            < std::mem::size_of::<surgeist_layout::NodeOutputOf<f64>>()
    );
    assert!(
        std::mem::size_of::<surgeist_layout::CollapsibleMargin>()
            < std::mem::size_of::<surgeist_layout::CollapsibleMarginOf<f64>>()
    );
    assert!(
        std::mem::size_of::<surgeist_layout::Cache>()
            < std::mem::size_of::<surgeist_layout::CacheOf<f64>>()
    );
}
```

- [ ] **Step 5: Genericize cache key and cache storage**

Keep cache context non-generic and limited to tree/cache context:

```rust
pub struct CacheKeyContext {
    calc_generation: CalcGeneration,
}

impl CacheKeyContext {
    pub const fn new(calc_generation: CalcGeneration) -> Self;
    pub const fn static_no_calc() -> Self;
    pub const fn calc_generation(self) -> CalcGeneration;
}
```

Move scalar-bearing input data into the private cache key:

```rust
struct CacheKeyOf<S: LayoutScalar = DefaultScalar> {
    pub run_mode: RunMode,
    pub sizing_mode: SizingMode,
    pub axis: RequestedAxis,
    pub known: Size<Option<S>>,
    pub parent: Size<Option<S>>,
    pub available: Size<AvailableOf<S>>,
    pub context: CacheKeyContext,
}

pub type Cache = CacheOf<DefaultScalar>;
```

Make `CacheOf<S>`, internal `EntryOf<S, T>`, `cache_slot`, and
`matches_output` generic over `S`. `cache_context(&self)` remains a no-argument
method because it returns only tree/cache context, not input-derived key data.

- [ ] **Step 6: Run focused cache/root tests**

Run:

```sh
cargo test -p surgeist-layout --test layout layout::cache -- --nocapture
cargo test -p surgeist-layout --test layout layout::root::f64_tree_can_run_root_layout_smoke_test -- --nocapture
cargo test -p surgeist-layout --test layout layout::root::f64_round_layout_preserves_large_coordinates -- --nocapture
cargo test -p surgeist-layout --test layout layout::cache::f64_cache_context_remains_tree_context_only -- --nocapture
cargo test -p surgeist-layout --test layout layout::cache::f64_cache_key_distinguishes_available_values_that_collide_as_f32 -- --nocapture
cargo test -p surgeist-layout --test layout layout::contract::f32_default_keeps_representative_layout_types_smaller_than_f64_lane -- --nocapture
```

Expected: pass.

- [ ] **Step 7: Commit**

```sh
git add src/traits.rs src/cache.rs src/compute.rs tests/support/oracle_tree.rs tests/layout/unit/cache.rs tests/layout/unit/root.rs
git commit -m "Make layout tree traits scalar aware"
```

---

## Task 6: Thread Scalar Through Block, Leaf, Inline, And Flex Algorithms

**Files:**
- Modify: `src/block.rs`
- Modify: `src/compute.rs`
- Modify: `src/flex.rs`
- Modify: `src/inline.rs`
- Test: `tests/layout/unit/block.rs`
- Test: `tests/layout/unit/flex.rs`
- Test: `tests/layout/unit/leaf.rs`

- [ ] **Step 1: Add f64 focused layout tests**

Add one f64 smoke test per algorithm area:

```rust
#[test]
fn f64_leaf_layout_preserves_fractional_precision() {
    let large = 16_777_217.25_f64;
    let style = NodeInputOf::<f64> {
        size: Size::new(DimensionOf::px(large), DimensionOf::px(2.5)),
        padding: Edges::all(LengthOf::px(0.125)),
        border: Edges::all(LengthOf::px(0.25)),
        ..NodeInputOf::<f64>::default()
    };

    let output = compute_leaf(
        ComputeInputOf {
            axis: RequestedAxis::Both,
            known: Size::new(None, None),
            parent: Size::new(None, None),
            available: Size::splat(AvailableOf::MAX_CONTENT),
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::ContentSize,
        },
        &style,
        |_known, _available| Size::new(0.0, 0.0),
    );

    assert_eq!(output.size.width, large);
    assert_eq!(output.size.height, 2.5);
}
```

Add the block smoke test:

```rust
#[test]
fn f64_block_layout_preserves_fractional_child_offsets() {
    let large = 16_777_217.25_f64;
    let mut tree = OracleTreeOf::<f64>::new()
        .style(
            0,
            NodeInputOf::<f64> {
                display: Display::Block,
                size: Size::new(DimensionOf::px(large + 100.0), DimensionOf::AUTO),
                padding: Edges::new(
                    LengthOf::px(large),
                    LengthOf::ZERO,
                    LengthOf::ZERO,
                    LengthOf::ZERO,
                ),
                ..NodeInputOf::<f64>::default()
            },
        )
        .style(
            1,
            NodeInputOf::<f64> {
                display: Display::Block,
                size: Size::new(DimensionOf::px(20.5), DimensionOf::px(30.75)),
                ..NodeInputOf::<f64>::default()
            },
        )
        .child(0, 1);

    compute_block(
        &mut tree,
        0,
        ComputeInputOf {
            axis: RequestedAxis::Both,
            known: Size::new(Some(large + 100.0), None),
            parent: Size::new(Some(large + 100.0), None),
            available: Size::new(AvailableOf::definite(large + 100.0), AvailableOf::MAX_CONTENT),
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::ContentSize,
        },
    );

    let child = tree.output(1);
    assert_eq!(child.location.y, large);
    assert_eq!(child.size, Size::new(20.5, 30.75));
}
```

Add the flex smoke test:

```rust
#[test]
fn f64_flex_layout_preserves_fractional_growth() {
    let large = 16_777_217.0_f64;
    let mut tree = OracleTreeOf::<f64>::new()
        .style(
            0,
            NodeInputOf::<f64> {
                display: Display::Flex,
                size: Size::new(DimensionOf::px(large + 100.5), DimensionOf::px(20.0)),
                ..NodeInputOf::<f64>::default()
            },
        )
        .style(
            1,
            NodeInputOf::<f64> {
                flex_grow: 1.0,
                flex_basis: DimensionOf::px(20.25),
                size: Size::new(DimensionOf::AUTO, DimensionOf::px(10.0)),
                ..NodeInputOf::<f64>::default()
            },
        )
        .child(0, 1);

    compute_flex(
        &mut tree,
        0,
        ComputeInputOf {
            axis: RequestedAxis::Both,
            known: Size::new(Some(large + 100.5), Some(20.0)),
            parent: Size::new(Some(large + 100.5), Some(20.0)),
            available: Size::new(
                AvailableOf::definite(large + 100.5),
                AvailableOf::definite(20.0),
            ),
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::ContentSize,
        },
    );

    let child = tree.output(1);
    assert_eq!(child.size.width, large + 100.5);
    assert_eq!(child.size.height, 10.0);
}
```

Add a hidden-layout smoke test for the public `compute_hidden` entry point:

```rust
#[test]
fn f64_compute_hidden_clears_layout_with_f64_output_type() {
    let mut tree = OracleTreeOf::<f64>::new()
        .style(
            0,
            NodeInputOf::<f64> {
                display: Display::None,
                ..NodeInputOf::<f64>::default()
            },
        );

    let output = compute_hidden(&mut tree, 0);

    assert_eq!(output, ComputeOutputOf::<f64>::HIDDEN);
    assert_eq!(tree.layout(0), Some(NodeOutputOf::<f64>::with_order(0)));
}
```

Add a real layout calc test that proves `Compute::calc_resolver` is wired into
the f64 algorithm path:

```rust
#[test]
fn f64_block_layout_resolves_calc_through_tree_resolver_without_narrowing() {
    let mut calcs = LayoutCalcStoreOf::<f64>::new();
    let width = calcs.push(CalcExpressionOf::sum(vec![
        CalcTermOf::px(16_777_217.0),
        CalcTermOf::percent(0.5),
    ]));

    let mut tree = OracleTreeOf::<f64>::new()
        .calcs(calcs)
        .style(
            0,
            NodeInputOf::<f64> {
                display: Display::Block,
                size: Size::new(DimensionOf::Calc(width), DimensionOf::px(10.0)),
                ..NodeInputOf::<f64>::default()
            },
        );

    compute_block(
        &mut tree,
        0,
        ComputeInputOf {
            axis: RequestedAxis::Both,
            known: Size::new(None, Some(10.0)),
            parent: Size::new(Some(21.0), Some(10.0)),
            available: Size::new(
                AvailableOf::definite(21.0),
                AvailableOf::definite(10.0),
            ),
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::ContentSize,
        },
    );

    let layout = tree.layout(0).expect("block should store unrounded layout");
    assert_eq!(layout.size.width, 16_777_227.5);
}
```

- [ ] **Step 2: Run and confirm failure**

Run:

```sh
cargo test -p surgeist-layout --test layout layout::leaf::f64_leaf_layout_preserves_fractional_precision -- --nocapture
cargo test -p surgeist-layout --test layout layout::block::f64_block_layout_preserves_fractional_child_offsets -- --nocapture
cargo test -p surgeist-layout --test layout layout::flex::f64_flex_layout_preserves_fractional_growth -- --nocapture
cargo test -p surgeist-layout --test layout layout::root::f64_compute_hidden_clears_layout_with_f64_output_type -- --nocapture
cargo test -p surgeist-layout --test layout layout::block::f64_block_layout_resolves_calc_through_tree_resolver_without_narrowing -- --nocapture
```

Expected: fail until `compute_leaf`, `compute_block`, `compute_flex`, and shared helpers are generic.

- [ ] **Step 3: Genericize block/leaf/inline/flex signatures**

Change public algorithm entry points from concrete scalar types to generic tree scalar types. Preserve the current root entry shape, where `compute_root` receives available space rather than a full `ComputeInput`:

```rust
pub fn compute_root<T>(
    tree: &mut T,
    root: <T as Traverse>::Node,
    available: Size<AvailableOf<<T as Traverse>::Scalar>>,
) where
    T: Compute,
{
    // implementation
}
```

For cached algorithm entry points, tie cache and traversal scalar types explicitly:

```rust
pub fn compute_hidden<T>(
    tree: &mut T,
    node: <T as Traverse>::Node,
) -> ComputeOutputOf<<T as Traverse>::Scalar>
where
    T: Compute
        + CacheAccess<
            Node = <T as Traverse>::Node,
            Scalar = <T as Traverse>::Scalar,
        >,
{
    // implementation
}

pub fn compute_flex<T>(
    tree: &mut T,
    node: <T as Traverse>::Node,
    input: ComputeInputOf<<T as Traverse>::Scalar>,
) -> ComputeOutputOf<<T as Traverse>::Scalar>
where
    T: Compute
        + CacheAccess<
            Node = <T as Traverse>::Node,
            Scalar = <T as Traverse>::Scalar,
        >,
{
    // implementation
}
```

For leaf-only helpers:

```rust
pub fn compute_leaf<S, M>(
    input: ComputeInputOf<S>,
    style: &NodeInputOf<S>,
    measure: M,
) -> ComputeOutputOf<S>
where
    S: LayoutScalar,
    M: FnOnce(Size<Option<S>>, Size<AvailableOf<S>>) -> Size<S>,
{
    // implementation
}
```

- [ ] **Step 4: Replace concrete scalar operations**

Inside generic algorithm code:

```rust
let zero = S::ZERO;
let one = S::ONE;
let child_count = S::from_usize(children.len());
let free_space = available - used;
let clamped = value.max(min).min(max);
```

Do not convert through `f32` or `f64` except at explicitly documented test/generator boundaries.

For scalar-to-count conversions such as grid auto-repeat, use the checked
scalar helper instead of ad hoc casts:

```rust
let repeat_count = ((available / repeat_size).floor().max(S::ONE))
    .floor_to_usize_saturating();
```

This is the sanctioned production scalar-to-`usize` boundary. It keeps the
integer conversion explicit and testable without routing generic layout math
through `f32` or `f64` at call sites.

- [ ] **Step 5: Run focused algorithm tests**

Run:

```sh
cargo test -p surgeist-layout --test layout layout::leaf -- --nocapture
cargo test -p surgeist-layout --test layout layout::block -- --nocapture
cargo test -p surgeist-layout --test layout layout::flex -- --nocapture
```

Expected: pass.

- [ ] **Step 6: Commit**

```sh
git add src/block.rs src/compute.rs src/flex.rs src/inline.rs tests/layout/unit/block.rs tests/layout/unit/flex.rs tests/layout/unit/leaf.rs
git commit -m "Thread scalar precision through block and flex layout"
```

---

## Task 7: Thread Scalar Through Grid Algorithms And Oracle Support

**Files:**
- Modify: `src/grid/*.rs`
- Modify: `tests/support/oracle/grid/*.rs`
- Modify: `tests/support/grid_layout_comparison.rs`
- Test: `tests/layout/unit/grid.rs`
- Test: `src/grid/tests.rs`

- [ ] **Step 1: Add f64 grid smoke tests**

In `tests/layout/unit/grid.rs`, add:

```rust
#[test]
fn f64_grid_tracks_preserve_large_coordinate_precision() {
    let large = 16_777_217.0_f64;
    let mut tree = OracleTreeOf::<f64>::new()
        .style(
            0,
            NodeInputOf::<f64> {
                display: Display::Grid,
                size: Size::new(DimensionOf::px(large + 10.5), DimensionOf::px(20.0)),
                grid_template_columns: vec![
                    TrackComponentOf::px(large),
                    TrackComponentOf::px(10.5),
                ],
                grid_template_rows: vec![TrackComponentOf::px(20.0)],
                ..NodeInputOf::<f64>::default()
            },
        )
        .style(
            1,
            NodeInputOf::<f64> {
                grid_column: GridPlacement::try_line(2).expect("valid second column"),
                grid_row: GridPlacement::try_line(1).expect("valid first row"),
                ..NodeInputOf::<f64>::default()
            },
        )
        .child(0, 1);

    compute_grid(
        &mut tree,
        0,
        ComputeInputOf {
            axis: RequestedAxis::Both,
            known: Size::new(Some(large + 10.5), Some(20.0)),
            parent: Size::new(Some(large + 10.5), Some(20.0)),
            available: Size::new(
                AvailableOf::definite(large + 10.5),
                AvailableOf::definite(20.0),
            ),
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::ContentSize,
        },
    );

    let child = tree.output(1);
    assert_eq!(child.location.x, large);
    assert_eq!(child.size.width, 10.5);
}
```

This test intentionally uses a value just above the exact integer range of `f32`; it should prove that the `f64` lane is not silently narrowed.

- [ ] **Step 2: Run and confirm failure**

Run:

```sh
cargo test -p surgeist-layout --test layout layout::grid::f64_grid_tracks_preserve_large_coordinate_precision -- --nocapture
```

Expected: fail until grid internals and test support are scalar-generic.

- [ ] **Step 3: Genericize grid reports and inputs**

Apply the `Of<S>` pattern to scalar-bearing public grid structs:

```rust
pub struct LaneContributionFactsOf<S: LayoutScalar = DefaultScalar> {
    pub min_size: S,
    pub min_content: S,
    pub max_content: S,
}

pub type LaneContributionFacts = LaneContributionFactsOf<DefaultScalar>;

pub struct LaneIntrinsicSizingInputOf<S: LayoutScalar = DefaultScalar> {
    pub available: Option<S>,
    pub gap: S,
    pub tracks: Vec<TrackComponentOf<S>>,
    pub items: Vec<LaneIntrinsicItemOf<S>>,
}

pub type LaneIntrinsicSizingInput = LaneIntrinsicSizingInputOf<DefaultScalar>;
```

Repeat for lane item/report/offset types, `GridComputation`, and any other scalar-bearing report currently exported from `src/grid/mod.rs`.

- [ ] **Step 4: Genericize grid internals**

Replace direct scalar operations in `src/grid/child.rs`, `src/grid/lanes.rs`, `src/grid/placement.rs`, `src/grid/subgrid.rs`, and `src/grid/tracks.rs` with `S: LayoutScalar` operations. Use `S::INFINITY` for infinite growth limits and `S::from_usize(...)` for count conversions.

- [ ] **Step 5: Genericize oracle support**

For test-only oracle structs that mirror layout facts, introduce `Of<S>` types only where they interact with generic public layout values. Keep default aliases for existing tests:

```rust
pub type GridLayoutComparison = GridLayoutComparisonOf<surgeist_layout::DefaultScalar>;
pub struct GridLayoutComparisonOf<S: surgeist_layout::LayoutScalar = surgeist_layout::DefaultScalar> {
    container: Size<S>,
    gap: Size<S>,
    children: Vec<GridLayoutNodeOf<S>>,
}
```

- [ ] **Step 6: Run grid checks**

Run:

```sh
cargo test -p surgeist-layout --lib grid::tests -- --nocapture
cargo test -p surgeist-layout --test layout layout::grid -- --nocapture
```

Expected: pass.

- [ ] **Step 7: Commit**

```sh
git add src/grid tests/support/oracle/grid tests/support/grid_layout_comparison.rs tests/layout/unit/grid.rs
git commit -m "Thread scalar precision through grid layout"
```

---

## Task 8: Isolate Browser Parity And Generator Precision Boundaries

**Files:**
- Modify: `tests/layout/browser_parity/support.rs`
- Modify: `tests/bin/surgeist-layout-generate/generator.rs`
- Modify: `tests/layout/browser_parity/README.md`
- Test: `tests/layout/browser_parity.rs`

- [ ] **Step 1: Add boundary comments and tests**

Add a focused test in `tests/layout/browser_parity/support.rs`:

```rust
#[test]
fn browser_parity_support_uses_default_layout_scalar() {
    assert_eq!(
        std::mem::size_of::<Scalar>(),
        std::mem::size_of::<surgeist_layout::DefaultScalar>()
    );
}
```

- [ ] **Step 2: Keep XML support default-only**

Ensure the support module keeps:

```rust
type Scalar = layout::Scalar;
```

and add a comment:

```rust
// Browser parity XML is a default-precision fixture boundary. The layout
// engine supports f64 through generic APIs, but these imported browser
// fixtures remain default-scalar until the fixture schema grows an explicit
// precision lane.
type Scalar = layout::Scalar;
```

- [ ] **Step 3: Document generator rounding**

In `tests/bin/surgeist-layout-generate/generator.rs`, keep the existing f32 cast in `layout_number_attr_value`, but add a local comment:

```rust
// The checked-in XML corpus is the default layout precision lane. Browser
// values arrive as f64 JSON numbers, then are rounded through f32 so fixture
// expectations match the default `surgeist_layout::Scalar` alias.
let value = value as f32;
```

- [ ] **Step 4: Run browser parity checks**

Run:

```sh
cargo test -p surgeist-layout --test layout layout::browser_parity::parses_all_checked_in_browser_parity_xml -- --nocapture
cargo test -p surgeist-layout --test layout layout::browser_parity::all_checked_in_browser_parity_xml_has_generator_provenance -- --nocapture
cargo run -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- check-corpus
```

Expected: pass with no generated artifact drift.

- [ ] **Step 5: Commit**

```sh
git add tests/layout/browser_parity/support.rs tests/bin/surgeist-layout-generate/generator.rs tests/layout/browser_parity/README.md
git commit -m "Document browser parity scalar boundary"
```

---

## Task 9: Update Public API, Docs, And Cross-Crate Ledger

**Files:**
- Modify: `README.md`
- Modify: `api/public-api.txt`
- Create: `plans/2026-06-27-surgeist-layout-generic-scalar-cross-crate-ledger.md`

- [ ] **Step 1: Write README scalar contract text**

Add a public API note:

```markdown
## Scalar Precision

`surgeist-layout` is generic over a crate-wide layout scalar precision. The
default public aliases use `f32` through `DefaultScalar` and `Scalar`, while
generic `*Of<S>` types support `f64` for applications that need larger
coordinate spaces or tighter accumulated precision.

One layout computation should use a single scalar type end-to-end. Sibling
crate adapters should choose either the default aliases or an explicit generic
lane rather than mixing `f32` and `f64` values inside one tree.
```

- [ ] **Step 2: Add rustdoc for scalar public contracts**

Add rustdoc to `src/scalar.rs` and `src/lib.rs` covering:

```rust
/// Numeric contract for one layout computation.
///
/// `surgeist-layout` supports `f32` and `f64` layout lanes. A tree, cache,
/// calc resolver, input set, and output set must use one scalar type
/// consistently; mixed-precision layout inside one computation is not a
/// supported contract.
pub trait LayoutScalar { /* ... */ }

/// Default layout scalar used by the unsuffixed public aliases.
///
/// The default is `f32` to preserve the compact memory profile for applications
/// that do not need a larger coordinate space.
pub type DefaultScalar = f32;

/// Backward-compatible alias for the default layout scalar.
pub type Scalar = DefaultScalar;
```

Add rustdoc to representative aliases in `src/lib.rs` or their owning modules:

```rust
/// Default-precision node input.
///
/// Use `NodeInputOf<f64>` when a layout tree needs the double-precision lane.
pub type NodeInput = NodeInputOf<DefaultScalar>;
```

- [ ] **Step 3: Create cross-crate ledger**

Create:

```markdown
# Surgeist Layout Generic Scalar Cross-Crate Ledger

This ledger records sibling/root work discovered while making `surgeist-layout`
generic over layout scalar precision. Layout workers must not edit sibling
crates from this repo.

## Entry Status

- `open`: Owning crate work is not yet available to this repo.
- `ready-to-retest`: Owning crate work is available locally; rerun layout verification.
- `closed`: Layout verification passed and the local task no longer depends on the handoff.

## Entries

### LAYOUT-SCALAR-XCRATE-0001: Style Adapter Must Choose Default Or Generic Layout Scalar

- Status: `open`
- Layout task: Task 9, `Update Public API, Docs, And Cross-Crate Ledger`
- Layout commit: `record after the layout-side commit exists`
- Owning crate: `surgeist-style`
- Required owning change: update the style layout adapter to construct either
  default `surgeist_layout` aliases or explicit `*Of<S>` generic values
  consistently, without mixing scalar precision in one lowered tree.
- Observed failure:

```text
coordination entry seeded from the scalar precision requirement; replace with
the exact compiler error when the style adapter is retested
```

- Pending layout verification:

```sh
cargo test -p surgeist-layout
```

### LAYOUT-SCALAR-XCRATE-0002: CSS Numeric Lowering Must Choose A Precision Lane

- Status: `open`
- Layout task: Task 9, `Update Public API, Docs, And Cross-Crate Ledger`
- Layout commit: `record after the layout-side commit exists`
- Owning crate: `surgeist-css`
- Required owning change: decide whether CSS numeric parsing lowers through
  default `f32` layout aliases only, or exposes a generic numeric lane that can
  feed `surgeist-layout` `f64` values without narrowing.
- Observed failure:

```text
coordination entry seeded from the scalar precision requirement; replace with
compiler/test output when the owning crate starts implementation
```

- Pending layout verification:

```sh
cargo test -p surgeist-layout
```

### LAYOUT-SCALAR-XCRATE-0003: Root Facade Must Select Or Expose Layout Precision

- Status: `open`
- Layout task: Task 9, `Update Public API, Docs, And Cross-Crate Ledger`
- Layout commit: `record after the layout-side commit exists`
- Owning crate: root `surgeist`
- Required owning change: choose whether the facade defaults to layout `f32`
  only, exposes precision-specific aliases, or makes scalar precision a root
  generic parameter shared by retained/style/layout/render integration.
- Observed failure:

```text
coordination entry seeded from the scalar precision requirement; replace with
integration failure output when root validation starts
```

- Pending layout verification:

```sh
cargo test -p surgeist-layout
```

### LAYOUT-SCALAR-XCRATE-0004: Text Measurements Must Match Or Convert Layout Precision

- Status: `open`
- Layout task: Task 9, `Update Public API, Docs, And Cross-Crate Ledger`
- Layout commit: `record after the layout-side commit exists`
- Owning crate: `surgeist-text`
- Required owning change: define whether measurement callbacks return the same
  scalar type as the layout tree or convert through an explicit boundary.
- Observed failure:

```text
coordination entry seeded from the scalar precision requirement; replace with
compiler/test output when text integration starts
```

- Pending layout verification:

```sh
cargo test -p surgeist-layout
```

### LAYOUT-SCALAR-XCRATE-0005: Render Coordinates Need A Precision Boundary

- Status: `open`
- Layout task: Task 9, `Update Public API, Docs, And Cross-Crate Ledger`
- Layout commit: `record after the layout-side commit exists`
- Owning crate: `surgeist-render`
- Required owning change: decide how `f64` layout output is accepted, stored, or
  converted when a backend uses `f32` render coordinates.
- Observed failure:

```text
coordination entry seeded from the scalar precision requirement; replace with
compiler/test output when render integration starts
```

- Pending layout verification:

```sh
cargo test -p surgeist-layout
```
```

- [ ] **Step 4: Regenerate public API artifact**

Run:

```sh
cargo run --manifest-path api/generator/Cargo.toml
```

Expected: `api/public-api.txt` reflects the new generic/default aliases and scalar trait.

- [ ] **Step 5: Run docs/API checks**

Run:

```sh
cargo test -p surgeist-layout --test layout layout::contract -- --nocapture
cargo doc -p surgeist-layout --no-deps
cargo fmt --check
git diff --check
```

Expected: pass.

- [ ] **Step 6: Commit**

```sh
git add README.md api/public-api.txt plans/2026-06-27-surgeist-layout-generic-scalar-cross-crate-ledger.md
git commit -m "Document generic layout scalar contract"
```

---

## Task 10: Final Verification

**Files:**
- No new source files expected.
- Modify: only if verification reveals a real issue.

- [ ] **Step 1: Run full crate verification**

Run:

```sh
cargo test -p surgeist-layout
cargo clippy -p surgeist-layout --all-targets -- -D warnings
cargo fmt --check
```

Expected: all pass.

- [ ] **Step 2: Run browser/golden verification**

Run:

```sh
cargo run -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- check-corpus
cargo test -p surgeist-layout --test layout layout::browser_parity::parses_all_checked_in_browser_parity_xml -- --nocapture
cargo test -p surgeist-layout --test layout layout::browser_parity::all_checked_in_browser_parity_xml_has_generator_provenance -- --nocapture
```

Expected: all pass.

- [ ] **Step 3: Run f64-specific verification block**

Run:

```sh
cargo test -p surgeist-layout --test layout layout::contract::layout_scalar_supports_f32_and_f64 -- --nocapture
cargo test -p surgeist-layout --test layout layout::contract::f32_default_keeps_representative_layout_types_smaller_than_f64_lane -- --nocapture
cargo test -p surgeist-layout --test layout layout::contract::geometry_supports_default_and_f64_scalars -- --nocapture
cargo test -p surgeist-layout --test layout layout::contract::value_types_support_f64_scalar_lane -- --nocapture
cargo test -p surgeist-layout --test layout layout::contract::f64_calc_resolution_preserves_large_coordinate_precision -- --nocapture
cargo test -p surgeist-layout --test layout layout::contract::node_input_and_output_support_f64_scalar_lane -- --nocapture
cargo test -p surgeist-layout --test layout layout::root::f64_tree_can_run_root_layout_smoke_test -- --nocapture
cargo test -p surgeist-layout --test layout layout::root::f64_round_layout_preserves_large_coordinates -- --nocapture
cargo test -p surgeist-layout --test layout layout::cache::f64_cache_context_remains_tree_context_only -- --nocapture
cargo test -p surgeist-layout --test layout layout::cache::f64_cache_key_distinguishes_available_values_that_collide_as_f32 -- --nocapture
cargo test -p surgeist-layout --test layout layout::leaf::f64_leaf_layout_preserves_fractional_precision -- --nocapture
cargo test -p surgeist-layout --test layout layout::block::f64_block_layout_preserves_fractional_child_offsets -- --nocapture
cargo test -p surgeist-layout --test layout layout::flex::f64_flex_layout_preserves_fractional_growth -- --nocapture
cargo test -p surgeist-layout --test layout layout::root::f64_compute_hidden_clears_layout_with_f64_output_type -- --nocapture
cargo test -p surgeist-layout --test layout layout::block::f64_block_layout_resolves_calc_through_tree_resolver_without_narrowing -- --nocapture
cargo test -p surgeist-layout --test layout layout::grid::f64_grid_tracks_preserve_large_coordinate_precision -- --nocapture
```

Expected: all f64/generic scalar tests pass.

- [ ] **Step 4: Inspect public API drift**

Run:

```sh
git diff -- api/public-api.txt
```

Expected: only intentional scalar genericity changes appear.

- [ ] **Step 5: Final reviewer cycle**

Request:

1. A focused reviewer for scalar genericity and precision safety.
2. A focused reviewer for public API shape and cross-crate coordination.
3. A holistic clean-context reviewer with no narrow scope other than: “as an experienced Rust developer, review whether this implementation plan and resulting commits satisfy the goal without overfitting to the named tasks.”

Proceed only after all reviewers return clean or after every finding is reconciled with code/docs/tests and re-reviewed.

- [ ] **Step 6: Commit any verification fixes**

If verification required changes, stage the exact files changed by those fixes:

```sh
git add src tests README.md api/public-api.txt plans/2026-06-27-surgeist-layout-generic-scalar-cross-crate-ledger.md
git commit -m "Finish generic layout scalar verification"
```

If no changes were required, record that in the coordinator final response.

---

## Reviewer Checklist

Focused scalar reviewer:

- Does every scalar-bearing public value have a default alias and a generic `Of<S>` form where needed?
- Can the f64 lane run real layout code without narrowing through f32?
- Are f32 and f64 prevented from mixing inside one tree computation?
- Are numeric constants and integer casts routed through `LayoutScalar` helpers?

Focused API/coordination reviewer:

- Is the default `f32` path still ergonomic?
- Are generic names clear in `api/public-api.txt`?
- Are sibling crate requirements recorded instead of edited from this repo?
- Does README explain the scalar precision contract without promising mixed precision?

Holistic Rust reviewer:

- Is the scalar abstraction idiomatic Rust rather than trait machinery for its own sake?
- Are trait bounds minimal enough for algorithms but complete enough to avoid repeated local hacks?
- Does the plan preserve layout crate boundaries with retained/style/css/render/text?
- Is the implementation sequence realistic for workers with limited context?

## Final Completion Criteria

- The plan is implemented with logical commits.
- `surgeist-layout` supports default `f32` and explicit `f64` layout lanes.
- Focused f64 tests prove large-coordinate precision is not silently narrowed.
- Default browser parity XML corpus remains green.
- Public API artifact and README reflect the scalar contract.
- Cross-crate ledger records all sibling/root follow-up work found during implementation.
- Focused reviewers and the holistic experienced Rust reviewer return clean.
