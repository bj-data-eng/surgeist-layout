# Surgeist Layout Modeling Fixes Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Resolve every verified finding in `plans/2026-06-24-surgeist-layout-modeling-review-findings.md` with typed layout contracts, explicit phase boundaries, semantic errors, and source-derived API artifacts.

**Architecture:** Work from the lowest-value contracts upward so algorithm changes can use the new types instead of adding temporary conventions. Public API hardening is intentional and may be breaking inside this pre-release crate, but each breaking surface must be documented in the API artifact and README when user-facing. Tooling-only fixes stay in fixture/oracle support and must not leak new app-facing behavior into `src/lib.rs`.

**Tech Stack:** Rust in `src/`, crate-local unit/oracle/browser parity tests under `tests/`, generator tooling under `tests/bin/surgeist-layout-generate`, API artifact tooling under `api/`, verification with `cargo test -p surgeist-layout`, `cargo clippy -p surgeist-layout --all-targets -- -D warnings`, and `cargo fmt --check`.

---

## Non-Negotiable Constraints

- This plan is crate-local to `/Users/codex/Development/surgeist-layout`.
- Do not edit sibling crates, the root `surgeist` checkout, or submodule pointers from this project.
- Do not add `unsafe`.
- Do not add new dependencies unless a worker first records why the standard library and current dependencies cannot model the invariant.
- Do not hand-edit generated XML geometry. Regenerate generated browser parity artifacts through the documented generator when provenance changes.
- Do not weaken tests, snapshots, provenance checks, or API checks to make implementation easier.
- Use source-derived API artifacts only; do not treat `api/public-api.txt` as the authority.
- Every code task must use worker/reviewer cycles per `AGENTS.md`.
- Commit at the logical checkpoints listed below only after the reviewer cycle for that task is clean.

## Finding Coverage Matrix

- `LAYOUT-MODEL-CALC-SYMBOLIC-COLLAPSE`: Tasks 1, 4, 12.
- `LAYOUT-MODEL-CACHE-KEY-CONTEXT`: Task 2.
- `LAYOUT-MODEL-ASPECT-RATIO-RAW-SCALAR`: Task 3.
- `GRID-PLACEMENT-PUBLIC-INVALID-STATES`: Task 5.
- `LAYOUT-PARITY-PROVENANCE-STYLE-HASH`: Task 10.
- `LAYOUT-MODEL-TRACK-REPEAT-INVALID-STATES`: Task 6.
- `LAYOUT-MODEL-BLOCK-MARGIN-OPTION-STATES`: Task 4.
- `LAYOUT-MODEL-FLEX-PHASE-BAG`: Task 8.
- `GRID-NAMED-ERRORS-SILENT-FALLBACK`: Task 7.
- `GRID-LANES-INTRINSIC-ITEM-PHASE-BAG`: Task 9.
- `GRID-LANES-ERROR-CONTEXT`: Task 9.
- `GRID-LANES-PUBLIC-TRACE-REPORT`: Task 9.
- `LAYOUT-PARITY-STRINGLY-CASE-STATUS`: Task 10.
- `LAYOUT-PARITY-UNTYPED-TOLERANCE-POLICY`: Task 10.
- `LINT-ALLOW-WITHOUT-REASON`: Task 12.

## File Map

- Modify: `src/value.rs`
  - Add semantic newtypes for aspect ratio, grid line/span, track repeat counts, non-empty track component lists, calc resolution status, and resolver generation.
- Modify: `src/node_input.rs`
  - Replace raw public fields for aspect ratio and grid placement with validated types or private fields plus constructors.
- Modify: `src/compute.rs`
  - Thread resolver-aware value resolution through root and leaf helpers.
  - Use `AspectRatio` instead of raw `Scalar`.
- Modify: `src/block.rs`
  - Replace resolver-free constants and ambiguous margin `Option<Scalar>` state.
- Modify: `src/flex.rs`
  - Split flex item state into phase-specific internal structs without changing external behavior.
- Modify: `src/cache.rs`
  - Replace partial cache entries with a typed cache key derived from all layout-relevant `ComputeInput` fields plus resolver generation.
- Modify: `src/traits.rs`
  - Extend resolver/cache contracts with a stable generation hook.
- Modify: `src/grid/placement.rs`
  - Resolve validated grid placement types rather than raw optional line/span bags.
- Modify: `src/grid/named.rs`
  - Preserve named-grid validation errors in typed reports instead of erasing them silently.
- Modify: `src/grid/mod.rs`
  - Consume named-grid reports and preserve current layout fallback as an explicit degraded report state.
- Modify: `src/grid/tracks.rs`
  - Consume validated track repetition data.
- Modify: `src/grid/lanes.rs`
  - Replace lane item field bags, broad error buckets, and public trace-bearing reports.
- Modify: `src/grid/axis.rs`, `src/grid/child.rs`, `src/grid/subgrid.rs`, `src/grid/named.rs`
  - Remove bare `#[allow]` attributes or replace with scoped `#[expect(..., reason = "...")]`.
- Modify: `src/lib.rs`
  - Re-export only intentional new front-door types.
- Modify: `src/tests.rs`
  - Add value, calc, aspect ratio, and track modeling tests.
- Modify: `tests/layout/unit/cache.rs`
  - Add cache key and resolver-generation tests.
- Modify: `tests/layout/unit/leaf.rs`
  - Add leaf calc/aspect ratio characterization.
- Modify: `tests/layout/unit/block.rs`
  - Add block calc and margin-state tests.
- Modify: `tests/layout/unit/flex.rs`
  - Add rerun/phase regression tests for the split flex states.
- Modify: `tests/layout/unit/grid.rs`
  - Add grid placement, named-grid report, track repetition, and lane API tests.
- Modify: `tests/support/grid_layout_comparison.rs`
  - Introduce typed comparison tolerance.
- Modify: `tests/support/oracle/grid/lanes.rs`
  - Mirror lane API/report changes only in oracle-specific types.
- Modify: `tests/layout/browser_parity/support.rs`
  - Consume typed comparison tolerance and generated provenance changes.
- Modify: `tests/bin/surgeist-layout-generate/generator.rs`
  - Add typed corpus status/generator/source-root states and base-style provenance.
- Generated: `tests/layout/browser_parity/xml/**`
  - Regenerate XML provenance comments after generator provenance changes.
- Generated: `tests/layout/browser_parity/xml/generation-reports/**`
  - Regenerate reports after generator provenance changes.
- Modify: `api/public-api.txt`
  - Refresh after public API changes using the crate-local API generator.
- Modify: `README.md`
  - Document release-facing public API hardening when the final public surface changes.

## Task 1: Make Calc Resolution Honest And Resolver-Aware

**Findings:** `LAYOUT-MODEL-CALC-SYMBOLIC-COLLAPSE`

**Files:**
- Modify: `src/value.rs`
- Modify: `src/traits.rs`
- Modify: `src/compute.rs`
- Modify: `src/block.rs`
- Modify: `src/tests.rs`
- Modify: `tests/layout/unit/leaf.rs`
- Modify: `tests/layout/unit/block.rs`

- [ ] **Step 1: Add failing tests for missing calc ids and resolver-free paths**

Add tests to `src/tests.rs`:

```rust
#[test]
fn missing_calc_id_reports_missing_expression() {
    let store = LayoutCalcStore::new();
    let missing = CalcId::from_raw_for_tests(99);
    let resolution = store.resolve_calc(missing, Some(80.0));

    assert_eq!(resolution.value, None);
    assert!(resolution.is_missing_expression());
    assert_eq!(resolution.status(), CalcResolutionStatus::MissingExpression);
}

#[test]
fn resolver_free_calc_resolution_is_visible() {
    let mut store = LayoutCalcStore::new();
    let id = store.push(CalcExpression::sum([CalcTerm::px(8.0)]));

    assert_eq!(Length::calc(id).resolve_optional(Some(40.0)), None);
    assert!(Length::calc(id).requires_resolver());
    assert_eq!(
        Length::calc(id).resolve_with_status(Some(40.0), &NoCalcResolver).status(),
        CalcResolutionStatus::MissingResolver
    );
}
```

Add a focused leaf test to `tests/layout/unit/leaf.rs`:

```rust
#[test]
fn leaf_calc_width_uses_tree_resolver() {
    let mut store = LayoutCalcStore::new();
    let width = store.push(CalcExpression::sum([CalcTerm::percent(0.5), CalcTerm::px(10.0)]));
    let style = NodeInput {
        size: Size::new(Dimension::calc(width), Dimension::AUTO),
        ..NodeInput::default()
    };
    let input = ComputeInput {
        run_mode: RunMode::PerformLayout,
        sizing_mode: SizingMode::InherentSize,
        axis: RequestedAxis::Both,
        known: Size::NONE,
        parent: Size::new(Some(100.0), None),
        available: Size::new(Available::definite(100.0), Available::MAX_CONTENT),
    };

    let output = compute_leaf_with_resolver(input, &style, &store, |_known, _available| {
        Size::new(12.0, 8.0)
    });

    assert_eq!(output.size.width, 60.0);
}
```

Add a block test to `tests/layout/unit/block.rs`:

```rust
#[test]
fn block_container_calc_padding_uses_tree_resolver() {
    let mut tree = CalcBlockTree::default();
    let padding = tree.calcs.push(CalcExpression::sum([
        CalcTerm::percent(0.1),
        CalcTerm::px(2.0),
    ]));
    tree.styles.insert(
        0,
        NodeInput {
            padding: Edges::all(Length::calc(padding)),
            ..NodeInput::default()
        },
    );

    let output = compute_block(
        &mut tree,
        0,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::new(Some(100.0), None),
            parent: Size::new(Some(100.0), None),
            available: Size::new(Available::definite(100.0), Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.content_size.width, 76.0);
}
```

Before adding this test, extract the local calc-capable tree from the existing
`block_in_flow_calc_margin_resolves_against_containing_block_width` test into a
private `CalcBlockTree` helper in `tests/layout/unit/block.rs`. The helper must
keep the existing `LayoutCalcStore` field and `Compute::calc_resolver`
implementation.

- [ ] **Step 2: Run tests to verify failure**

Run:

```sh
cargo test -p surgeist-layout tests::missing_calc_id_reports_missing_expression tests::resolver_free_calc_resolution_is_visible -- --nocapture
cargo test -p surgeist-layout --test layout layout::leaf::leaf_calc_width_uses_tree_resolver -- --nocapture
cargo test -p surgeist-layout --test layout layout::block::block_container_calc_padding_uses_tree_resolver -- --nocapture
```

Expected: tests fail because `CalcResolutionStatus`, `is_missing_expression`, `requires_resolver`, `resolve_with_status`, and `compute_leaf_with_resolver` do not exist, and block constants still use resolver-free helpers.

- [ ] **Step 3: Add calc resolution status and test-only raw id constructor**

In `src/value.rs`, remove the public `CalcId::new(index: u32)` constructor.
Only `LayoutCalcStore::push` may mint ordinary calc ids. Add a crate-private
constructor for the store and a unit-test-only raw constructor for malformed-id
tests:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CalcResolutionStatus {
    Resolved,
    MissingBasis,
    MissingResolver,
    MissingExpression,
}

impl CalcId {
    pub(crate) const fn from_store_index(index: u32) -> Self {
        Self(index)
    }

    #[cfg(test)]
    #[must_use]
    pub const fn from_raw_for_tests(index: u32) -> Self {
        Self(index)
    }
}

impl CalcResolution {
    #[must_use]
    pub const fn missing_expression() -> Self {
        Self {
            value: None,
            depends_on_basis: false,
            status: CalcResolutionStatus::MissingExpression,
        }
    }

    #[must_use]
    pub const fn missing_resolver() -> Self {
        Self {
            value: None,
            depends_on_basis: false,
            status: CalcResolutionStatus::MissingResolver,
        }
    }

    #[must_use]
    pub const fn status(self) -> CalcResolutionStatus {
        self.status
    }

    #[must_use]
    pub const fn is_missing_expression(self) -> bool {
        matches!(self.status, CalcResolutionStatus::MissingExpression)
    }
}
```

Update `CalcResolution` to include `status: CalcResolutionStatus`. Update its constructors so resolved values use `Resolved`, basis-dependent unresolved values use `MissingBasis`, and no-resolver paths use `MissingResolver`.

Update `LayoutCalcStore::push` to call `CalcId::from_store_index`. Update any
tests that currently call `CalcId::new` to use `from_raw_for_tests` if they need
an invalid id. Do not expose any public raw-id constructor from `src/lib.rs` or
`api/public-api.txt`.

- [ ] **Step 4: Make `LayoutCalcStore` report missing ids**

Change `LayoutCalcStore::resolve_calc` so missing ids return `CalcResolution::missing_expression()` instead of unresolved/non-basis-dependent.

- [ ] **Step 5: Add resolver-aware helper methods**

In `src/value.rs`, add status-preserving helpers:

```rust
impl Length {
    #[must_use]
    pub const fn requires_resolver(self) -> bool {
        matches!(self, Self::Calc(_))
    }

    #[must_use]
    pub fn resolve_with_status(
        self,
        basis: Option<Scalar>,
        resolver: &dyn CalcResolver,
    ) -> CalcResolution {
        match self {
            Self::Normal => CalcResolution::definite(0.0, false),
            Self::Px(value) => CalcResolution::definite(value, false),
            Self::Percent(value) => basis.map_or(
                CalcResolution::unresolved(true),
                |basis| CalcResolution::definite(value * basis, true),
            ),
            Self::Calc(id) => resolver.resolve_calc(id, basis),
        }
    }
}
```

Add equivalent `requires_resolver` and `resolve_with_status` methods for `LengthAuto` and `Dimension`.

- [ ] **Step 6: Thread resolver-aware helpers through root, leaf, and block constants**

Update `src/compute.rs` so root and leaf resolution use resolver-aware helpers. If `compute_leaf` cannot take a resolver without a public signature break, add an internal `compute_leaf_with_resolver` and keep `compute_leaf` delegating with `NoCalcResolver`.

Update `src/block.rs` so `Constants::new` receives `tree.calc_resolver()` or a resolver parameter, and its padding, border, size, min-size, max-size, and own margin helpers use resolver-aware methods.

- [ ] **Step 7: Run focused tests**

Run:

```sh
cargo test -p surgeist-layout tests::missing_calc_id_reports_missing_expression tests::resolver_free_calc_resolution_is_visible -- --nocapture
cargo test -p surgeist-layout --test layout layout::leaf -- --nocapture
cargo test -p surgeist-layout --test layout layout::block -- --nocapture
```

Expected: all selected tests pass.

- [ ] **Step 8: Commit**

```sh
git add src/value.rs src/traits.rs src/compute.rs src/block.rs src/tests.rs tests/layout/unit/leaf.rs tests/layout/unit/block.rs
git commit -m "Make layout calc resolution explicit"
```

## Task 2: Expand Cache Keys To Full Compute Context

**Findings:** `LAYOUT-MODEL-CACHE-KEY-CONTEXT`

**Files:**
- Modify: `src/cache.rs`
- Modify: `src/output.rs`
- Modify: `src/traits.rs`
- Modify: `src/lib.rs`
- Modify: `tests/layout/unit/cache.rs`

- [ ] **Step 1: Add failing cache tests for mode, axis, parent, and resolver generation**

Add to `tests/layout/unit/cache.rs`:

```rust
fn cache_test_input() -> ComputeInput {
    ComputeInput {
        run_mode: RunMode::PerformLayout,
        sizing_mode: SizingMode::InherentSize,
        axis: RequestedAxis::Both,
        known: Size::new(None, None),
        parent: Size::new(Some(300.0), Some(200.0)),
        available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    }
}

#[test]
fn cache_miss_when_run_mode_changes() {
    let mut cache = Cache::new();
    let base = cache_test_input();
    cache.store(&base, ComputeOutput::from_outer_size(Size::new(20.0, 10.0)));

    let mut changed = base;
    changed.run_mode = RunMode::ComputeSize;

    assert_eq!(cache.get(&changed), None);
}

#[test]
fn cache_miss_when_sizing_mode_changes() {
    let mut cache = Cache::new();
    let base = cache_test_input();
    cache.store(&base, ComputeOutput::from_outer_size(Size::new(20.0, 10.0)));

    let mut changed = base;
    changed.sizing_mode = SizingMode::ContentSize;

    assert_eq!(cache.get(&changed), None);
}

#[test]
fn cache_miss_when_requested_axis_changes() {
    let mut cache = Cache::new();
    let base = cache_test_input();
    cache.store(&base, ComputeOutput::from_outer_size(Size::new(20.0, 10.0)));

    let mut changed = base;
    changed.axis = RequestedAxis::Horizontal;

    assert_eq!(cache.get(&changed), None);
}

#[test]
fn cache_miss_when_parent_size_changes() {
    let mut cache = Cache::new();
    let base = cache_test_input();
    cache.store(&base, ComputeOutput::from_outer_size(Size::new(20.0, 10.0)));

    let mut changed = base;
    changed.parent = Size::new(Some(200.0), Some(40.0));

    assert_eq!(cache.get(&changed), None);
}

#[test]
fn cache_miss_when_calc_generation_changes() {
    let mut cache = Cache::new();
    let base = cache_test_input();
    cache.store_with_context(
        &base,
        CacheKeyContext::new(CalcGeneration::new(1)),
        ComputeOutput::from_outer_size(Size::new(20.0, 10.0)),
    );

    assert_eq!(
        cache.get_with_context(&base, CacheKeyContext::new(CalcGeneration::new(2))),
        None
    );
}

#[test]
fn compute_cached_uses_cache_access_context_generation() {
    struct Probe {
        cache: Cache,
        generation: CalcGeneration,
        calls: usize,
    }

    impl CacheAccess for Probe {
        type Node = u32;

        fn cache_context(&self) -> CacheKeyContext {
            CacheKeyContext::new(self.generation)
        }

        fn cache_get(&self, _node: Self::Node, input: &ComputeInput) -> Option<ComputeOutput> {
            self.cache.get_with_context(input, self.cache_context())
        }

        fn cache_store(&mut self, _node: Self::Node, input: &ComputeInput, output: ComputeOutput) {
            self.cache
                .store_with_context(input, self.cache_context(), output);
        }

        fn cache_clear(&mut self, _node: Self::Node) {
            self.cache.clear();
        }
    }

    let input = cache_test_input();
    let mut probe = Probe {
        cache: Cache::new(),
        generation: CalcGeneration::new(1),
        calls: 0,
    };

    let first = compute_cached(&mut probe, 7, input, |tree, _node, _input| {
        tree.calls += 1;
        ComputeOutput::from_outer_size(Size::new(20.0, 10.0))
    });
    probe.generation = CalcGeneration::new(2);
    let second = compute_cached(&mut probe, 7, input, |tree, _node, _input| {
        tree.calls += 1;
        ComputeOutput::from_outer_size(Size::new(30.0, 10.0))
    });

    assert_eq!(first.size.width, 20.0);
    assert_eq!(second.size.width, 30.0);
    assert_eq!(probe.calls, 2);
}
```

- [ ] **Step 2: Run tests to verify failure**

Run:

```sh
cargo test -p surgeist-layout --test layout layout::cache -- --nocapture
```

Expected: new tests fail because cache keys ignore these fields and `CalcGeneration`, `CacheKeyContext`, `get_with_context`, `store_with_context`, and `CacheAccess::cache_context` do not exist.

- [ ] **Step 3: Add `CalcGeneration` and `CacheKeyContext`**

In `src/value.rs` or `src/output.rs`, add:

```rust
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct CalcGeneration(u64);

impl CalcGeneration {
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}
```

In `src/cache.rs`, add:

```rust
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct CacheKeyContext {
    calc_generation: CalcGeneration,
}

impl CacheKeyContext {
    #[must_use]
    pub const fn new(calc_generation: CalcGeneration) -> Self {
        Self { calc_generation }
    }
}
```

Do not add `calc_generation` as a required field on `ComputeInput`; there are
many existing `ComputeInput` literals and the generation is cache context, not
layout input.

Re-export `CalcGeneration` and `CacheKeyContext` from `src/lib.rs` so
integration tests and downstream cache users can name the cache context
contract intentionally.

- [ ] **Step 4: Add resolver and cache context hooks**

Extend `CalcResolver` with:

```rust
fn calc_generation(&self) -> CalcGeneration {
    CalcGeneration::default()
}
```

Make `LayoutCalcStore` return a generation that changes when expressions are pushed. If the store is append-only, the expression count is acceptable:

```rust
fn calc_generation(&self) -> CalcGeneration {
    CalcGeneration::new(self.len() as u64)
}
```

Extend `CacheAccess` with a default context hook:

```rust
fn cache_context(&self) -> CacheKeyContext {
    CacheKeyContext::default()
}
```

Update real tree implementations that own a calc store, including browser
parity `TestTree`, so `cache_context()` returns
`CacheKeyContext::new(self.calc_resolver().calc_generation())` or the direct
store generation when the resolver is the store.

- [ ] **Step 5: Replace cache entry fields with a typed key**

In `src/cache.rs`, replace `known` and `available` in `Entry<T>` with:

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
struct CacheKey {
    run_mode: RunMode,
    sizing_mode: SizingMode,
    axis: RequestedAxis,
    known: Size<Option<Scalar>>,
    parent: Size<Option<Scalar>>,
    available: Size<Available>,
    context: CacheKeyContext,
}
```

Implement `CacheKey::from_input(input: &ComputeInput, context: CacheKeyContext) -> Self`, store it in entries, and make `matches_output` compare the full key except for the existing known-size shortcut only when all other fields are equal.

Add `Cache::get_with_context` and `Cache::store_with_context`. Keep
`Cache::get` and `Cache::store` as compatibility wrappers that use
`CacheKeyContext::default()`.

- [ ] **Step 6: Stamp cache context in production cache paths**

Update `compute_cached` in `src/traits.rs` so it asks the tree for
`tree.cache_context()` before cache lookup/store. Update direct cache users such
as `tests/layout/browser_parity/support.rs` so they call
`get_with_context`/`store_with_context` or route through `CacheAccess` methods
that apply the context.

- [ ] **Step 7: Run focused cache tests**

Run:

```sh
cargo test -p surgeist-layout --test layout layout::cache -- --nocapture
```

Expected: cache tests pass.

- [ ] **Step 8: Commit**

```sh
git add src/cache.rs src/output.rs src/traits.rs src/value.rs src/lib.rs tests/layout/unit/cache.rs tests/layout/browser_parity/support.rs
git commit -m "Key layout cache by compute context"
```

## Task 3: Model Aspect Ratio As A Validated Semantic Type

**Findings:** `LAYOUT-MODEL-ASPECT-RATIO-RAW-SCALAR`

**Files:**
- Modify: `src/value.rs`
- Modify: `src/node_input.rs`
- Modify: `src/compute.rs`
- Modify: `src/block.rs`
- Modify: `src/flex.rs`
- Modify: `src/grid/child.rs`
- Modify: `src/tests.rs`
- Modify: `tests/layout/unit/leaf.rs`
- Modify: `api/public-api.txt`

- [ ] **Step 1: Add failing aspect ratio tests**

Add to `src/tests.rs`:

```rust
#[test]
fn aspect_ratio_rejects_non_positive_or_non_finite_values() {
    assert!(AspectRatio::new(1.5).is_some());
    assert_eq!(AspectRatio::new(0.0), None);
    assert_eq!(AspectRatio::new(-1.0), None);
    assert_eq!(AspectRatio::new(Scalar::NAN), None);
    assert_eq!(AspectRatio::new(Scalar::INFINITY), None);
}
```

Add to `tests/layout/unit/leaf.rs`:

```rust
#[test]
fn leaf_uses_validated_aspect_ratio() {
    let input = ComputeInput {
        run_mode: RunMode::PerformLayout,
        sizing_mode: SizingMode::InherentSize,
        axis: RequestedAxis::Both,
        known: Size::NONE,
        parent: Size::new(Some(120.0), Some(80.0)),
        available: Size::new(Available::definite(120.0), Available::MAX_CONTENT),
    };
    let style = NodeInput {
        size: Size::new(Dimension::px(60.0), Dimension::AUTO),
        aspect_ratio: AspectRatio::new(2.0),
        ..NodeInput::default()
    };

    let output = compute_leaf(input, &style, |_known, _available| Size::new(10.0, 10.0));

    assert_eq!(output.size, Size::new(60.0, 30.0));
}
```

- [ ] **Step 2: Run tests to verify failure**

Run:

```sh
cargo test -p surgeist-layout tests::aspect_ratio_rejects_non_positive_or_non_finite_values -- --nocapture
cargo test -p surgeist-layout --test layout layout::leaf::leaf_uses_validated_aspect_ratio -- --nocapture
```

Expected: tests fail because `AspectRatio` does not exist.

- [ ] **Step 3: Add `AspectRatio`**

In `src/value.rs`, add:

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AspectRatio(Scalar);

impl AspectRatio {
    #[must_use]
    pub fn new(value: Scalar) -> Option<Self> {
        (value.is_finite() && value > 0.0).then_some(Self(value))
    }

    #[must_use]
    pub const fn get(self) -> Scalar {
        self.0
    }
}
```

- [ ] **Step 4: Replace raw aspect ratio fields and helpers**

Change `NodeInput::aspect_ratio` from `Option<Scalar>` to `Option<AspectRatio>`. Update all `.apply_aspect_ratio` helpers to take `Option<AspectRatio>` and use `ratio.get()`.

- [ ] **Step 5: Update constructors/tests/call sites**

Replace direct `Some(1.5)` aspect ratio construction with `AspectRatio::new(1.5)`. Do not add unchecked public constructors.

- [ ] **Step 6: Run focused tests**

Run:

```sh
cargo test -p surgeist-layout tests::aspect_ratio_rejects_non_positive_or_non_finite_values -- --nocapture
cargo test -p surgeist-layout --test layout layout::leaf -- --nocapture
cargo test -p surgeist-layout --test layout layout::block -- --nocapture
cargo test -p surgeist-layout --test layout layout::flex -- --nocapture
cargo test -p surgeist-layout --test layout layout::grid -- --nocapture
```

Expected: all selected tests pass.

- [ ] **Step 7: Commit**

```sh
git add src/value.rs src/node_input.rs src/compute.rs src/block.rs src/flex.rs src/grid/child.rs src/tests.rs tests/layout/unit/leaf.rs
git commit -m "Model aspect ratio as a validated value"
```

## Task 4: Replace Resolver-Free Algorithm Helpers

**Findings:** `LAYOUT-MODEL-CALC-SYMBOLIC-COLLAPSE`, `LAYOUT-MODEL-BLOCK-MARGIN-OPTION-STATES`

**Files:**
- Modify: `src/compute.rs`
- Modify: `src/block.rs`
- Modify: `src/flex.rs`
- Modify: `src/grid/mod.rs`
- Modify: `src/grid/tracks.rs`
- Modify: `src/grid/child.rs`
- Modify: `src/grid/subgrid.rs`
- Modify: `tests/layout/unit/block.rs`
- Modify: `tests/layout/unit/flex.rs`
- Modify: `tests/layout/unit/grid.rs`

- [ ] **Step 1: Add a shared resolved length state**

In `src/value.rs`, add:

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ResolvedLengthAuto {
    Auto,
    Resolved(Scalar),
    Unresolved(CalcResolutionStatus),
}
```

Add `LengthAuto::resolve_auto_with_status(basis, resolver) -> ResolvedLengthAuto`.

- [ ] **Step 2: Replace ambiguous block margin state**

In `src/block.rs`, change in-flow margin resolution from `Edges<Option<Scalar>>` to `Edges<ResolvedLengthAuto>`. Update `resolve_in_flow_margin` to distribute only `ResolvedLengthAuto::Auto`; treat `Unresolved(_)` as a visible unresolved state that follows the same fallback rules currently used by the algorithm, with a local comment naming the fallback.

- [ ] **Step 3: Add focused block margin regression tests**

Add to `tests/layout/unit/block.rs`:

```rust
#[test]
fn unresolved_symbolic_vertical_margin_is_not_treated_as_auto_margin() {
    let mut tree = CalcBlockTree::default();
    let margin = tree.calcs.push(CalcExpression::sum([CalcTerm::percent(0.25)]));
    tree.styles.insert(
        1,
        NodeInput {
            margin: Edges {
                top: LengthAuto::calc(margin),
                ..Edges::ZERO.map(|_| LengthAuto::px(0.0))
            },
            ..NodeInput::default()
        },
    );

    let resolved = resolve_in_flow_margin_for_tests(
        tree.styles[&1].margin,
        Size::new(10.0, 10.0),
        Size::new(None, None),
        &tree.calcs,
    );

    assert_eq!(resolved.top, 0.0);
}
```

Expose the helper only under tests:

```rust
#[cfg(test)]
fn resolve_in_flow_margin_for_tests(
    margin: Edges<LengthAuto>,
    child_size: Size,
    container_size: Size<Option<Scalar>>,
    resolver: &dyn CalcResolver,
) -> Edges {
    let resolved = margin.zip_inline_size(container_size, |length, basis| {
        length.resolve_auto_with_status(basis, resolver)
    });
    resolve_in_flow_margin(resolved, child_size, container_size.width)
}
```

- [ ] **Step 4: Replace exact percent/calc checks in flex/grid**

Update flex and grid helper paths so basis-dependent checks use `depends_on_basis` and `requires_resolver` methods from `src/value.rs`, not direct `Percent` or resolver-free `Calc` matches.

- [ ] **Step 5: Run focused algorithm tests**

Run:

```sh
cargo test -p surgeist-layout --test layout layout::block -- --nocapture
cargo test -p surgeist-layout --test layout layout::flex -- --nocapture
cargo test -p surgeist-layout --test layout layout::grid -- --nocapture
```

Expected: all selected tests pass.

- [ ] **Step 6: Commit**

```sh
git add src/value.rs src/compute.rs src/block.rs src/flex.rs src/grid/mod.rs src/grid/tracks.rs src/grid/child.rs src/grid/subgrid.rs tests/layout/unit/block.rs tests/layout/unit/flex.rs tests/layout/unit/grid.rs
git commit -m "Use resolver-aware layout value states"
```

## Task 5: Validate Public Grid Placement

**Findings:** `GRID-PLACEMENT-PUBLIC-INVALID-STATES`

**Files:**
- Modify: `src/value.rs`
- Modify: `src/node_input.rs`
- Modify: `src/grid/placement.rs`
- Modify: `src/grid/tests.rs`
- Modify: `tests/layout/unit/grid.rs`
- Modify: `src/lib.rs`
- Modify: `api/public-api.txt`

- [ ] **Step 1: Add failing grid placement tests**

Add to `src/grid/tests.rs`:

```rust
#[test]
fn public_grid_placement_rejects_zero_line_and_span() {
    assert_eq!(GridLine::new(0), None);
    assert_eq!(GridSpan::new(0), None);
    assert!(GridLine::new(1).is_some());
    assert!(GridSpan::new(1).is_some());
}

#[test]
fn grid_placement_fields_are_constructed_through_validated_values() {
    let placement = GridPlacement::line_span(
        GridLine::new(2).expect("valid line"),
        GridSpan::new(3).expect("valid span"),
    );

    assert_eq!(placement.start(), Some(GridLine::new(2).unwrap()));
    assert_eq!(placement.span(), Some(GridSpan::new(3).unwrap()));
}
```

- [ ] **Step 2: Run tests to verify failure**

Run:

```sh
cargo test -p surgeist-layout grid::tests::public_grid_placement_rejects_zero_line_and_span grid::tests::grid_placement_fields_are_constructed_through_validated_values -- --nocapture
```

Expected: tests fail because `GridLine`, `GridSpan`, and accessor-based `GridPlacement` do not exist.

- [ ] **Step 3: Add `GridLine` and `GridSpan`**

In `src/value.rs` or `src/node_input.rs`, add:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GridLine(isize);

impl GridLine {
    #[must_use]
    pub const fn new(value: isize) -> Option<Self> {
        if value == 0 { None } else { Some(Self(value)) }
    }

    #[must_use]
    pub const fn get(self) -> isize {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GridSpan(core::num::NonZeroUsize);

impl GridSpan {
    #[must_use]
    pub const fn new(value: usize) -> Option<Self> {
        match core::num::NonZeroUsize::new(value) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }

    #[must_use]
    pub const fn get(self) -> usize {
        self.0.get()
    }
}
```

- [ ] **Step 4: Make `GridPlacement` fields private**

Change `GridPlacement` to:

```rust
pub struct GridPlacement {
    start: Option<GridLine>,
    end: Option<GridLine>,
    span: Option<GridSpan>,
}
```

Update constructors to take validated values. Add `try_line`, `try_lines`, `try_line_span`, `try_span_line`, and `try_span` helpers for raw callers that need validation results.

- [ ] **Step 5: Update placement normalization**

Change `src/grid/placement.rs` to use `GridLine::get()` and `GridSpan::get()` and remove `span.max(1)` fallback for validated placements.

- [ ] **Step 6: Run grid tests**

Run:

```sh
cargo test -p surgeist-layout grid::tests::public_grid_placement_rejects_zero_line_and_span grid::tests::grid_placement_fields_are_constructed_through_validated_values -- --nocapture
cargo test -p surgeist-layout --test layout layout::grid -- --nocapture
```

Expected: all selected tests pass.

- [ ] **Step 7: Commit**

```sh
git add src/value.rs src/node_input.rs src/grid/placement.rs src/grid/tests.rs tests/layout/unit/grid.rs src/lib.rs
git commit -m "Validate public grid placement values"
```

## Task 6: Validate Track Repetition Values

**Findings:** `LAYOUT-MODEL-TRACK-REPEAT-INVALID-STATES`

**Files:**
- Modify: `src/value.rs`
- Modify: `src/grid/tracks.rs`
- Modify: `tests/layout/unit/grid.rs`
- Modify: `src/tests.rs`
- Modify: `api/public-api.txt`

- [ ] **Step 1: Add failing track repetition tests**

Add to `src/tests.rs`:

```rust
#[test]
fn track_repetition_rejects_zero_count_and_empty_components() {
    assert!(TrackRepeatCount::new(0).is_none());
    assert!(TrackRepeatCount::new(2).is_some());
    assert!(TrackComponentList::try_from(Vec::<TrackComponent>::new()).is_err());
}
```

- [ ] **Step 2: Add validated repeat types**

In `src/value.rs`, introduce:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TrackRepeatCount(core::num::NonZeroUsize);

#[derive(Clone, Debug, PartialEq)]
pub struct TrackComponentList(Vec<TrackComponent>);
```

Add `TrackRepeatCount::new`, `TrackRepeatCount::get`, `TryFrom<Vec<TrackComponent>> for TrackComponentList`, and `TrackComponentList::as_slice`.

- [ ] **Step 3: Replace public repeat fields**

Change `TrackRepeat::Count(usize)` to `TrackRepeat::Count(TrackRepeatCount)`. Change `TrackRepetition` so fields are private and constructors return `Result<Self, TrackRepetitionError>` when raw inputs are invalid.

- [ ] **Step 4: Run focused tests**

Run:

```sh
cargo test -p surgeist-layout tests::track_repetition_rejects_zero_count_and_empty_components -- --nocapture
cargo test -p surgeist-layout --test layout layout::grid -- --nocapture
```

Expected: all selected tests pass.

- [ ] **Step 5: Commit**

```sh
git add src/value.rs src/grid/tracks.rs src/tests.rs tests/layout/unit/grid.rs
git commit -m "Validate grid track repetition values"
```

## Task 7: Preserve Named Grid Validation Reports

**Findings:** `GRID-NAMED-ERRORS-SILENT-FALLBACK`

**Files:**
- Modify: `src/grid/named.rs`
- Modify: `src/grid/mod.rs`
- Modify: `src/output.rs`
- Modify: `src/grid/tests.rs`
- Modify: `src/lib.rs`
- Modify: `tests/layout/unit/grid.rs`

- [ ] **Step 1: Add failing named-grid report test**

Add this context-report test to `tests/layout/unit/grid.rs`:

```rust
#[test]
fn invalid_named_grid_context_is_reported() {
    let mut tree = support::oracle_tree::OracleTree::new().style(
        0,
        NodeInput {
            grid_template_areas: GridTemplateAreas {
                rows: vec![
                    GridTemplateAreaRow {
                        cells: vec![Some("a".to_string()), Some("b".to_string())],
                    },
                    GridTemplateAreaRow {
                        cells: vec![Some("a".to_string())],
                    },
                ],
            },
            ..NodeInput::default()
        },
    );

    let output = compute_grid(
        &mut tree,
        0,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(200.0), Some(100.0)),
            available: Size::new(Available::definite(200.0), Available::MAX_CONTENT),
        },
    );

    assert!(output.reports.named_grid_errors().any(|error| {
        matches!(error.kind(), NamedGridErrorKind::InvalidTemplateAreas)
    }));
}
```

Add this placement-fallback test to `src/grid/tests.rs`, where the internal
named-grid helpers are visible:

```rust
#[test]
fn named_grid_placement_fallback_is_reported() {
    use super::named::{NamedGridLines, resolve_grid_placement_or_auto_with_report};

    let placement = RawGridPlacement {
        start: RawGridLine::LineName {
            name: "missing".to_string(),
            nth: 1,
        },
        end: RawGridLine::Auto,
    };
    let context = NamedGridLines::default();

    let (placement, report) =
        resolve_grid_placement_or_auto_with_report(&context, &placement, None);

    assert_eq!(placement, GridPlacement::AUTO);
    assert!(report.errors().any(|error| {
        matches!(error.kind(), NamedGridErrorKind::UnresolvedLineName)
    }));
}
```

- [ ] **Step 2: Add named grid report types**

Add a report type that records named-grid errors while preserving the current fallback layout behavior:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamedGridReport {
    errors: Vec<NamedGridErrorReport>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamedGridErrorReport {
    kind: NamedGridErrorKind,
}
```

Define `NamedGridErrorKind` from the existing `NamedGridError` variants:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NamedGridErrorKind {
    ReservedLineName,
    UnresolvedAutoRepeatNames,
    InvalidTemplateAreas,
    InvalidRepeat,
    InvalidLine,
    AutoWithoutCursor,
    LineBeforeFirst,
}

impl From<&NamedGridError> for NamedGridErrorKind {
    fn from(error: &NamedGridError) -> Self {
        match error {
            NamedGridError::ReservedLineName { .. } => Self::ReservedLineName,
            NamedGridError::UnresolvedAutoRepeatNames { .. } => Self::UnresolvedAutoRepeatNames,
            NamedGridError::EmptyTemplateAreas
            | NamedGridError::TemplateAreaRowLengthMismatch { .. }
            | NamedGridError::NonRectangularTemplateArea { .. } => Self::InvalidTemplateAreas,
            NamedGridError::ZeroRepeat { .. }
            | NamedGridError::MultipleAutoFillRepeats { .. } => Self::InvalidRepeat,
            NamedGridError::ZeroLine | NamedGridError::ZeroSpan => Self::InvalidLine,
            NamedGridError::AutoWithoutCursor => Self::AutoWithoutCursor,
            NamedGridError::LineBeforeFirst { .. } => Self::LineBeforeFirst,
        }
    }
}
```

Expose read-only accessors, not mutable fields. Add a helper that reports the
existing placement fallback instead of hiding it:

```rust
pub(super) fn resolve_grid_placement_or_auto_with_report(
    lines: &NamedGridLines,
    placement: &RawGridPlacement,
    auto_cursor_line: Option<isize>,
) -> (GridPlacement, NamedGridReport) {
    match resolve_grid_placement(lines, placement, auto_cursor_line) {
        Ok(placement) => (placement, NamedGridReport::default()),
        Err(error) => (
            GridPlacement::AUTO,
            NamedGridReport::from_error(NamedGridErrorKind::from(&error)),
        ),
    }
}
```

- [ ] **Step 3: Keep fallback explicit**

Change named context construction so `build_grid_named_context` errors are captured into `NamedGridReport` before falling back to `empty_grid_named_context`. Change placement resolution call sites to use `resolve_grid_placement_or_auto_with_report` and merge those reports into the grid output report.

Re-export the public report access types needed by integration tests from
`src/lib.rs`, including `NamedGridReport`, `NamedGridErrorReport`, and
`NamedGridErrorKind`.

- [ ] **Step 4: Run grid tests**

Run:

```sh
cargo test -p surgeist-layout --test layout layout::grid -- --nocapture
```

Expected: grid tests pass and new report test passes.

- [ ] **Step 5: Commit**

```sh
git add src/grid/named.rs src/grid/mod.rs src/output.rs src/grid/tests.rs src/lib.rs tests/layout/unit/grid.rs
git commit -m "Report named grid validation fallback"
```

## Task 8: Split Flex Item Algorithm Phases

**Findings:** `LAYOUT-MODEL-FLEX-PHASE-BAG`

**Files:**
- Modify: `src/flex.rs`
- Modify: `tests/layout/unit/flex.rs`

- [ ] **Step 1: Add a rerun regression test before refactoring**

Add to `tests/layout/unit/flex.rs`:

```rust
#[test]
fn flex_final_content_size_uses_rerun_output() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for FlexTree {
        type Node = u32;
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

    impl Compute for FlexTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput {
            self.inputs.entry(node).or_default().push(input);
            let size = if input.known.width == Some(80.0) {
                Size::new(80.0, 40.0)
            } else {
                Size::new(20.0, 10.0)
            };
            ComputeOutput::from_sizes(size, size)
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(0, vec![1]);
    tree.children.insert(1, vec![]);
    tree.styles.insert(
        0,
        NodeInput {
            display: Display::Flex,
            size: Size::new(Dimension::px(80.0), Dimension::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(1, NodeInput::default());

    let output = compute_flex(
        &mut tree,
        0,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(80.0), None),
            available: Size::new(Available::definite(80.0), Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.content_size.height, 40.0);
}
```

- [ ] **Step 2: Introduce phase structs**

In `src/flex.rs`, split the current `FlexItem` into:

```rust
struct CollectedFlexItem<Node> { /* authored style, initial measure, base facts */ }
struct ResolvedFlexItem<Node> { /* target sizes, line membership, offsets */ }
struct FinalFlexItem<Node> { /* final output, baseline, final offsets */ }
```

Keep these structs private to `src/flex.rs`.

- [ ] **Step 3: Move functions across phase boundaries**

Update collection to return `Vec<CollectedFlexItem<_>>`, line resolution to produce `Vec<ResolvedFlexItem<_>>`, and final layout to produce `Vec<FinalFlexItem<_>>`. Make `visible_content_size` accept only final items.

- [ ] **Step 4: Run focused flex tests**

Run:

```sh
cargo test -p surgeist-layout --test layout layout::flex -- --nocapture
```

Expected: flex tests pass.

- [ ] **Step 5: Commit**

```sh
git add src/flex.rs tests/layout/unit/flex.rs
git commit -m "Split flex item layout phases"
```

## Task 9: Harden Lane Intrinsic API, Errors, And Reports

**Findings:** `GRID-LANES-INTRINSIC-ITEM-PHASE-BAG`, `GRID-LANES-ERROR-CONTEXT`, `GRID-LANES-PUBLIC-TRACE-REPORT`

**Files:**
- Modify: `src/grid/lanes.rs`
- Modify: `src/lib.rs`
- Modify: `tests/support/oracle/grid/lanes.rs`
- Modify: `tests/support/grid_layout_comparison.rs`
- Modify: `tests/layout/unit/grid.rs`
- Modify: `api/public-api.txt`

- [ ] **Step 1: Add failing lane API tests**

Add to `tests/layout/unit/grid.rs`:

```rust
#[test]
fn lane_intrinsic_item_exposes_exactly_one_kind() {
    let item = LaneIntrinsicItem::indefinite(
        "a",
        LaneTrackSpanLength::new(2).expect("nonzero span"),
        LaneContributionFacts {
            min_content: 1.0,
            max_content: 2.0,
            min_size: 0.0,
            automatic_minimum_applies: false,
        },
    );

    assert!(matches!(item.kind(), LaneIntrinsicItemKind::Indefinite { .. }));
}

#[test]
fn lane_errors_carry_context() {
    let error = place_lanes(LanePlacementInput {
        grid_axis_tracks: 2,
        auto_flow: GridAutoFlow::Row,
        lane_gap: 0.0,
        tolerance: GridFlowTolerance::Normal { font_size: 16.0 },
        tolerance_basis: 16.0,
        items: vec![LaneItem {
            item: "a",
            definite_grid_axis_start: Some(0),
            grid_axis_span: 1,
            lane_axis_margin_box: 10.0,
        }],
    })
    .expect_err("zero start should fail");

    assert!(matches!(
        error,
        LanePlacementError::InvalidGridAxisStart { start: 0 }
    ));
}
```

- [ ] **Step 2: Replace lane item field bag with kind enum**

In `src/grid/lanes.rs`, introduce:

```rust
#[derive(Clone, Debug, PartialEq)]
pub enum LaneIntrinsicItemKind {
    Definite { span: LaneTrackSpan },
    Indefinite { span: LaneTrackSpanLength },
    NestedIndefiniteSubgrid { span: LaneTrackSpanLength },
}

#[derive(Clone, Debug, PartialEq)]
pub struct LaneIntrinsicItem {
    id: &'static str,
    kind: LaneIntrinsicItemKind,
    contribution: LaneContributionFacts,
}
```

Define the nonzero lane span-length type used by the kind enum:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LaneTrackSpanLength(core::num::NonZeroUsize);

impl LaneTrackSpanLength {
    #[must_use]
    pub const fn new(value: usize) -> Option<Self> {
        match core::num::NonZeroUsize::new(value) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }

    #[must_use]
    pub const fn get(self) -> usize {
        self.0.get()
    }
}
```

Add accessors and validated constructors. Remove public `span`, `definite_span`, and `nested_indefinite_subgrid` fields.

- [ ] **Step 3: Add contextual lane errors**

Replace broad `SpanOutOfRange` returns with variants such as:

```rust
pub enum LanePlacementError {
    EmptyTrackList,
    InvalidGridAxisStart { start: usize },
    InvalidGridAxisSpan { span: usize },
    GridAxisSpanOutOfRange { start: usize, span: usize, tracks: usize },
    ContentSizedTrackOutOfRange { track_index: usize, tracks: usize },
    DefiniteLaneSpanOutOfRange { span: LaneTrackSpan, tracks: usize },
    NestedGridLanesSubgridIndefiniteUnsupported,
}
```

- [ ] **Step 4: Split public report from trace**

Change `LanePlacementReport` so it exposes stable facts only. Add a private or test-support `LanePlacementTrace` for `running_positions_after_each_item` and `final_cursor`. Update oracle/comparison support to use the trace type only where tests need it.

- [ ] **Step 5: Run focused lane tests**

Run:

```sh
cargo test -p surgeist-layout --test layout grid_lanes -- --nocapture
cargo test -p surgeist-layout --test layout layout::grid -- --nocapture
```

Expected: lane and grid tests pass.

- [ ] **Step 6: Commit**

```sh
git add src/grid/lanes.rs src/lib.rs tests/support/oracle/grid/lanes.rs tests/support/grid_layout_comparison.rs tests/layout/unit/grid.rs
git commit -m "Harden grid lane modeling contracts"
```

## Task 10: Type Fixture Tooling Workflow State, Provenance, And Tolerances

**Findings:** `LAYOUT-PARITY-PROVENANCE-STYLE-HASH`, `LAYOUT-PARITY-STRINGLY-CASE-STATUS`, `LAYOUT-PARITY-UNTYPED-TOLERANCE-POLICY`

**Files:**
- Modify: `tests/bin/surgeist-layout-generate/generator.rs`
- Modify: `tests/layout/browser_parity/support.rs`
- Modify: `tests/support/grid_layout_comparison.rs`
- Generated: `tests/layout/browser_parity/xml/**`
- Generated: `tests/layout/browser_parity/xml/generation-reports/**`

- [ ] **Step 1: Add failing generator unit tests**

Add generator tests in `tests/bin/surgeist-layout-generate/generator.rs`:

```rust
#[test]
fn corpus_case_status_deserializes_to_closed_domain() {
    let case: CorpusCase = toml::from_str(
        r#"
        id = "case"
        source_root = "html"
        source = "block/example.html"
        generator = "constrained-html"
        status = "active"
        "#,
    )
    .expect("valid case");

    assert_eq!(case.status, CorpusStatus::Active);
    assert_eq!(case.generator, CorpusGenerator::ConstrainedHtml);
}

#[test]
fn generation_metadata_hashes_base_style_source() {
    let metadata = generation_report_metadata_for_tests();
    assert_eq!(
        metadata.base_style_sha256,
        sha256_bytes(TEST_BASE_STYLE_SOURCE.as_bytes())
    );
}
```

Add support tests:

```rust
#[test]
fn comparison_tolerance_is_named_policy() {
    let tolerance = ComparisonTolerance::browser_parity();
    assert!(tolerance.contains(0.05));
    assert!(!tolerance.contains(0.2));
}
```

- [ ] **Step 2: Add typed corpus state**

Replace string fields with:

```rust
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
enum CorpusStatus {
    Active,
    ExpectedFail,
    Unsupported,
    Quarantined,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
enum CorpusGenerator {
    ConstrainedHtml,
}
```

Add a typed source-root enum for the known manifest source roots.

- [ ] **Step 3: Add base-style provenance**

Add `base_style_sha256` to report metadata and XML provenance comments. Update provenance validation so checked-in XML must match the current base style hash when the HTML references `test_base_style.css`.

- [ ] **Step 4: Add comparison tolerance type**

Add:

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComparisonTolerance {
    value: Scalar,
}

impl ComparisonTolerance {
    pub const fn browser_parity() -> Self { Self { value: 0.1 } }
    pub const fn oracle_grid() -> Self { Self { value: 0.000_1 } }
    pub fn contains(self, delta: Scalar) -> bool { delta.abs() <= self.value }
}
```

Use this type in browser parity and oracle comparison helpers.

- [ ] **Step 5: Run generator and tolerance unit tests**

Run:

```sh
cargo test -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- --nocapture
cargo test -p surgeist-layout --test layout layout::browser_parity::support::tests::comparison_tolerance_is_named_policy -- --nocapture
```

Expected: generator-bin tests and the focused tolerance test pass.

- [ ] **Step 6: Regenerate generated artifacts**

Run:

```sh
cargo run -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- generate
cargo run -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- check-corpus
```

Expected: generated XML provenance comments and generation reports update to include base-style provenance; corpus check passes.

- [ ] **Step 7: Run fixture checks**

Run:

```sh
cargo test -p surgeist-layout --test layout layout::browser_parity::parses_all_checked_in_browser_parity_xml -- --nocapture
cargo test -p surgeist-layout --test layout layout::browser_parity::all_checked_in_browser_parity_xml_has_generator_provenance -- --nocapture
```

Expected: both tests pass.

- [ ] **Step 8: Commit**

```sh
git add tests/bin/surgeist-layout-generate/generator.rs tests/layout/browser_parity/support.rs tests/support/grid_layout_comparison.rs tests/layout/browser_parity/xml tests/layout/browser_parity/xml/generation-reports
git commit -m "Type browser parity fixture workflow state"
```

## Task 11: Refresh Public API Artifact And README

**Findings:** all API-hardening findings

**Files:**
- Modify: `api/public-api.txt`
- Modify: `README.md`
- Inspect: `api/generator/src/main.rs`
- Inspect: `api/generator/Cargo.toml`

- [ ] **Step 1: Regenerate public API artifact**

Run the crate-local API generator command documented in `README.md`:

```sh
cargo run --manifest-path api/generator/Cargo.toml
```

Expected: `api/public-api.txt` reflects source-derived public API changes.

- [ ] **Step 2: Review API diff**

Run:

```sh
git diff -- api/public-api.txt
```

Expected: diff shows intentional removal or hardening of raw public field bags and addition of validated semantic types.

- [ ] **Step 3: Update README public API notes**

Add a short section to `README.md`:

```markdown
## Modeling Contracts

`surgeist-layout` exposes layout-ready contracts rather than authored CSS syntax.
Public placement, aspect ratio, track repetition, lane, and calc values preserve
their invariants through typed constructors and resolver-aware APIs.
```

- [ ] **Step 4: Commit**

```sh
git add api/public-api.txt README.md
git commit -m "Refresh layout modeling API artifacts"
```

## Task 12: Remove Or Justify Lint Suppressions

**Findings:** `LINT-ALLOW-WITHOUT-REASON`

**Files:**
- Modify: `src/grid/axis.rs`
- Modify: `src/grid/child.rs`
- Modify: `src/grid/named.rs`
- Modify: `src/grid/subgrid.rs`
- Modify: `src/grid/lanes.rs`

- [ ] **Step 1: Search current lint suppressions**

Run:

```sh
rg -n '#\[allow|#\[expect' src/grid/axis.rs src/grid/child.rs src/grid/named.rs src/grid/subgrid.rs src/grid/lanes.rs
```

Expected: every remaining lint exception is visible.

- [ ] **Step 2: Remove dead-code allows made unnecessary by prior tasks**

Delete `#[allow(dead_code)]` attributes for types or functions that are now used after the modeling changes.

- [ ] **Step 3: Convert intentional exceptions to `#[expect]` with reasons**

For each remaining intentional exception, use:

```rust
#[expect(clippy::too_many_arguments, reason = "grid layout phase helper carries explicit algorithm inputs until the phase type split is complete")]
```

or:

```rust
#[expect(dead_code, reason = "subgrid report field is retained for forthcoming oracle parity coverage")]
```

The reason must name the invariant or staged plan, not merely say "needed".

- [ ] **Step 4: Run lint verification**

Run:

```sh
cargo clippy -p surgeist-layout --all-targets -- -D warnings
```

Expected: clippy passes with no unfulfilled `#[expect]` attributes.

- [ ] **Step 5: Commit**

```sh
git add src/grid/axis.rs src/grid/child.rs src/grid/named.rs src/grid/subgrid.rs src/grid/lanes.rs
git commit -m "Justify grid lint exceptions"
```

## Task 13: Final Crate Verification And Findings Closure

**Findings:** all findings

**Files:**
- Modify: `plans/2026-06-24-surgeist-layout-modeling-review-findings.md`

- [ ] **Step 1: Run full focused crate checks**

Run:

```sh
cargo test -p surgeist-layout
cargo clippy -p surgeist-layout --all-targets -- -D warnings
cargo fmt --check
```

Expected: all commands pass.

- [ ] **Step 2: Verify generated/tooling checks**

Run:

```sh
cargo run -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- check-corpus
cargo test -p surgeist-layout --test layout layout::browser_parity::parses_all_checked_in_browser_parity_xml -- --nocapture
cargo test -p surgeist-layout --test layout layout::browser_parity::all_checked_in_browser_parity_xml_has_generator_provenance -- --nocapture
```

Expected: all commands pass.

- [ ] **Step 3: Mark findings accepted**

In `plans/2026-06-24-surgeist-layout-modeling-review-findings.md`, change each finding status from `verified` to `accepted` only after the implementing task has landed and the final checks above pass. Add one reconciliation log entry:

```markdown
- 2026-06-26: Implementation plan completed. Each verified finding has a corresponding implementation commit and final crate-local verification passed.
```

- [ ] **Step 4: Request final clean-context review**

Ask a fresh reviewer to inspect the implementation commits, this plan, and the findings ledger. Required reviewer result: either no remaining coverage gaps or exact findings that must be reconciled before completion.

- [ ] **Step 5: Commit closure entry**

```sh
git add plans/2026-06-24-surgeist-layout-modeling-review-findings.md
git commit -m "Mark modeling review findings accepted"
```

## Final Completion Criteria

- Every finding in the coverage matrix has a completed task.
- Every task has a clean worker/reviewer cycle.
- All crate-local checks pass:

```sh
cargo test -p surgeist-layout
cargo clippy -p surgeist-layout --all-targets -- -D warnings
cargo fmt --check
```

- Generator/provenance checks pass after generated artifacts change.
- `api/public-api.txt` is regenerated from source after public API changes.
- The findings ledger records accepted statuses only after implementation and verification.
- A final clean-context reviewer reports no remaining plan or implementation coverage gaps.
