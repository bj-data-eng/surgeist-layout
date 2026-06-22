# Surgeist Layout Typed Values and Calc Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Upgrade the `surgeist-layout` value boundary so calc handles and basis-dependent sizing behavior can grow without turning every extension into closed enum churn, while handing style/CSS authoring work to the owning crates.

**Architecture:** Keep layout algorithm dispatch enums private unless they become intentional extension surfaces. Style owns authored CSS values, calc AST parsing, serialization, validation, and property metadata; the style-to-layout adapter normalizes CSS percent semantics once and lowers values into typed layout handles. Layout values carry only typed concrete values plus opaque calc handles and resolve through a resolver hook that reports both numeric results and basis dependency.

**Tech Stack:** Rust in this `surgeist-layout` crate, focused module tests with `cargo test -p surgeist-layout`, formatting with `cargo fmt --check`, lint verification with `cargo clippy -p surgeist-layout --all-targets -- -D warnings`, optional read-only comparison against `../des-document/layout/src/style/dimension.rs` and `../des-document/layout/src/style/compact_length.rs` when that checkout is available.

---

## Non-Negotiable Constraints

- Plan execution must use scoped diffs and scoped commits. This crate repo may be used as a submodule by the top-level `surgeist` workspace; implementation workers must leave sibling crates and top-level submodule pointers untouched unless a coordinator explicitly hands off that work.
- Do not edit `../surgeist-style`, `../surgeist-css`, `../surgeist`, or any other sibling checkout from this crate project. Capture required sibling changes as handoff notes or execute them in the owning project.
- Do not make layout algorithm enums public unless a task explicitly promotes one as an extension surface.
- Do not add `#[allow]`, `#[expect]`, crate-level lint allow lists, or `clippy::` suppressions. Fix warnings by improving API shape, names, tests, or code structure.
- Do not copy Taffy's pointer tagging, unsafe calc storage, or lint-suppression approach. Surgeist calc storage must use typed handles and ordinary safe Rust.
- Preserve the semantic split between authored CSS percentages such as `50.0` and layout-normalized percentage factors such as `0.5`; normalization happens exactly once in the style adapter.
- Every implementation task is TDD-first: write a focused failing test, run it and observe the expected failure, implement the smallest change, verify, then commit with the scoped message listed in the task.

## Split-Repo Execution Scope

This document now lives in the standalone `surgeist-layout` crate repo. Tasks that touch `src/...`, `tests/...`, `api/...`, or this crate's `plans/...` are executable here. Tasks that mention `../surgeist-style`, `../surgeist-css`, or `../surgeist` are cross-crate handoff notes only; do not edit those sibling checkouts from this project. If a layout task depends on a style/CSS API that is not available yet, stop after capturing the exact upstream issue or issue draft for the owning crate.

The original monorepo plan included style parsing, style resolver, CSS parser, and facade export work. Those details are retained as implementation guidance for the owning crates, but this crate-local plan should commit only layout value, layout algorithm, layout tests, parity tooling, and source-derived API artifact changes.

## Revised Task Ownership

Execute these tasks in this crate project:

1. Task 1: baseline layout value resolver seam in `src/value.rs`, `src/lib.rs`, and `src/tests.rs`.
2. Task 2: centralized layout resolution helpers in `src/compute.rs`, `src/block.rs`, `src/flex.rs`, and `src/grid/**/*.rs`.
3. Task 4 layout portion: `CalcId`, `CalcResolver`, `LayoutCalcStore`, calc-bearing layout value variants, `Compute::calc_resolver`, and crate-local tests.
4. Task 5: flex/grid basis logic and resolver threading.
5. Task 9: layout lint suppression cleanup, if any suppressions exist.
6. Task 10: final crate-local verification and public API artifact refresh.

Treat these sections as upstream handoff drafts, not executable crate-local tasks:

- Task 3 belongs to `surgeist-style` and `surgeist-css`.
- The style adapter lowering portion of Task 4 belongs to `surgeist-style`.
- Task 6 belongs to `surgeist-style`.
- Task 7 belongs to `surgeist-style` and `surgeist-css`, except for any explicit `src/node_input.rs` compatibility tests requested by the style adapter contract.
- Task 8 belongs to `surgeist-style`.
- Any facade export work belongs to the top-level `surgeist` repo.

## Calc Ownership Invariant

`surgeist-style` owns authored calc ASTs and normalization into layout-facing values. `surgeist-layout` owns only typed concrete values, opaque `CalcId` handles, safe resolver/store contracts, and algorithm behavior when a value depends on a sizing basis. Every `CalcId` stored anywhere in a layout tree resolves against one layout-pass store through `Compute::calc_resolver(&self) -> &dyn CalcResolver`; no individual `NodeInput` owns or clones the store.

## Evidence From Evaluation

- The biggest extension bottleneck is the closed `style::Property` and `style::Value` lane, not private layout enums.
- `style::Display` currently mixes CSS spelling, inline participation, box generation, and layout dispatch. Split rich style descriptors from closed layout dispatch after calc/property plumbing is stable.
- `style::Length` is broad and context-rejected in `../surgeist-style/src/adapters/layout.rs`; future work should introduce narrower wrappers instead of expanding adapter rejections.
- Calc support is risky because percent-dependent behavior currently depends on duplicated resolution helpers and direct `Percent(_)` matches in flex/grid code.
- Layout modules currently duplicate helpers such as `resolve_length_or_zero`, `resolve_auto_optional`, and `resolve_dimension` in `compute.rs`, `flex.rs`, `block.rs`, and `grid/mod.rs`.
- Grid and flex rerun/intrinsic logic checks exact percent variants in places such as `src/flex.rs`, `src/grid/mod.rs`, and `src/grid/tracks.rs`; `calc(20px + 10%)` must participate in the same decisions.

## Module Evaluation Pattern

Use this pattern before upgrading each Surgeist module after this plan:

1. Map the module boundary: list the public types, private algorithm enums, adapter inputs, and tests that currently define behavior.
2. Classify each type as authored data, normalized adapter data, layout algorithm state, retained runtime state, or extension metadata.
3. Identify closed enums that block extension and private enums that merely protect implementation detail. Open only the first category.
4. Find duplicated helpers, exact variant checks, validation rejections, and percent or axis assumptions.
5. Add characterization tests that capture the current intended behavior before changing structure.
6. Introduce typed seams with the smallest possible migration surface.
7. Verify with focused module tests, then run package-level formatting and lint checks.
8. Record any deferred cleanup as a named staged task with exact files and tests.

## File Map

- Modify: `src/value.rs`
  - Add layout calc handle/resolver traits, value resolution methods, and basis-dependency methods.
- Modify: `src/lib.rs`
  - Re-export only intentional front-door layout value APIs.
- Modify: `src/node_input.rs`
  - Continue to store per-node normalized layout values; do not store layout calc arenas here.
- Modify: `src/traits.rs`
  - Expose the layout-pass calc resolver from tree/session implementations.
- Modify: `src/tests.rs`
  - Add egui-free layout value tests for resolution, calc handles, and basis dependency.
- Modify: `src/compute.rs`
  - Replace local px/percent helper bodies with layout value methods.
- Modify: `src/block.rs`
  - Replace local px/percent helper bodies with layout value methods.
- Modify: `src/flex.rs`
  - Replace local helpers and exact percent checks with layout value dependency methods.
- Modify: `src/grid/mod.rs`
  - Replace local helpers and track rerun checks with layout value dependency methods.
- Modify: `src/grid/tracks.rs`
  - Replace track percent detection and intrinsic percent math with track sizing methods that understand calc metadata.
- Modify: `src/grid/lanes.rs`
  - Replace exact percent checks where lane placement depends on basis or percent-aware track sizing.
- Modify: `src/grid/child.rs`
  - Replace local helper calls with shared layout value methods where the same semantics are already used.
- Modify: `src/grid/subgrid.rs`
  - Replace local helper calls with shared layout value methods where subgrid edge/gap semantics match existing behavior.
- Upstream handoff to `surgeist-style` (do not edit from this repo): `../surgeist-style/src/calc.rs`
  - Own authored calc AST, validation, display/debug-friendly serialization helpers, and tests.
- Upstream handoff to `surgeist-style` (do not edit from this repo): `../surgeist-style/src/lib.rs`
  - Export style calc and open property/value boundary types intentionally.
- Upstream handoff to `surgeist-style` (do not edit from this repo): `../surgeist-style/src/value.rs`
  - Add style calc-bearing length variants or typed value wrappers and grid name newtypes.
- Upstream handoff to `surgeist-style` (do not edit from this repo): `../surgeist-style/src/property.rs`
  - Add `PropertyId`, `PropertyDescriptor`, and descriptor conversion for built-in `Property`.
- Upstream handoff to `surgeist-style` (do not edit from this repo): `../surgeist-style/src/declaration.rs`
  - Preserve fingerprints and value hashing for open/custom value lanes and calc values.
- Upstream handoff to `surgeist-style` (do not edit from this repo): `../surgeist-style/src/resolver.rs`
  - Resolve and cache built-in and open property IDs without forcing custom values into the closed enum.
- Upstream handoff to `surgeist-style` (do not edit from this repo): `../surgeist-style/src/adapters/layout.rs`
  - Normalize CSS percentages once and lower authored calc ASTs into layout `CalcId` handles.
- Upstream handoff to `surgeist-css` (do not edit from this repo): `../surgeist-css/src/lib.rs`
  - Parse and serialize CSS calc expressions in style-owned values.
- Top-level handoff to `surgeist` (do not edit from this repo): `../surgeist/src/lib.rs` only if a new public module export is already mirrored by existing module policy.
- Read-only reference: `../des-document/layout/src/style/dimension.rs`
  - Compare typed layout value API shape.
- Read-only reference: `../des-document/layout/src/style/compact_length.rs`
  - Compare compact value handling and avoid copying unsafe internals.

### Task 1: Baseline Characterization and No-Op Resolver Surface

**Files:**
- Modify: `src/value.rs`
- Modify: `src/lib.rs`
- Modify: `src/tests.rs`

- [ ] **Step 1: Write failing layout value tests**

Add these tests to `src/tests.rs`:

```rust
#[test]
fn layout_lengths_report_basis_dependency() {
    assert!(!Length::NORMAL.depends_on_basis());
    assert!(!Length::px(12.0).depends_on_basis());
    assert!(Length::percent(0.25).depends_on_basis());

    assert!(!LengthAuto::AUTO.depends_on_basis());
    assert!(!LengthAuto::px(12.0).depends_on_basis());
    assert!(LengthAuto::percent(0.25).depends_on_basis());

    assert!(!Dimension::AUTO.depends_on_basis());
    assert!(!Dimension::px(12.0).depends_on_basis());
    assert!(Dimension::percent(0.25).depends_on_basis());
}

#[test]
fn layout_lengths_resolve_optional_basis_consistently() {
    assert_eq!(Length::px(12.0).resolve_or_zero(None), 12.0);
    assert_eq!(Length::percent(0.25).resolve_or_zero(None), 0.0);
    assert_eq!(Length::percent(0.25).resolve_or_zero(Some(80.0)), 20.0);
    assert_eq!(Length::percent(0.25).resolve_optional(None), None);
    assert_eq!(Length::percent(0.25).resolve_optional(Some(80.0)), Some(20.0));

    assert_eq!(LengthAuto::AUTO.resolve_or_zero(Some(80.0)), 0.0);
    assert_eq!(LengthAuto::percent(0.25).resolve_optional(Some(80.0)), Some(20.0));
    assert_eq!(Dimension::percent(0.25).resolve_optional(Some(80.0)), Some(20.0));
}

#[test]
fn no_calc_resolver_keeps_plain_values_working() {
    let resolver = NoCalcResolver;
    assert_eq!(Length::px(8.0).resolve_with(Some(40.0), &resolver), Some(8.0));
    assert_eq!(
        Length::percent(0.5).resolve_with(Some(40.0), &resolver),
        Some(20.0)
    );
}
```

- [ ] **Step 2: Run tests to verify failure**

Run:

```sh
cargo test -p surgeist-layout tests::layout_lengths_report_basis_dependency tests::layout_lengths_resolve_optional_basis_consistently tests::no_calc_resolver_keeps_plain_values_working
```

Expected: tests fail to compile because `depends_on_basis`, `resolve_or_zero`, `resolve_optional`, `NoCalcResolver`, and `resolve_with` do not exist.

- [ ] **Step 3: Add minimal no-op resolver and value methods**

In `src/value.rs`, add the resolver API near the top after `Available`:

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CalcId(u32);

impl CalcId {
    #[must_use]
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    #[must_use]
    pub const fn index(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CalcResolution {
    pub value: Option<Scalar>,
    pub depends_on_basis: bool,
}

impl CalcResolution {
    #[must_use]
    pub const fn definite(value: Scalar, depends_on_basis: bool) -> Self {
        Self {
            value: Some(value),
            depends_on_basis,
        }
    }

    #[must_use]
    pub const fn unresolved(depends_on_basis: bool) -> Self {
        Self {
            value: None,
            depends_on_basis,
        }
    }
}

pub trait CalcResolver {
    fn resolve_calc(&self, id: CalcId, basis: Option<Scalar>) -> CalcResolution;
    fn calc_depends_on_basis(&self, id: CalcId) -> bool;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NoCalcResolver;

impl CalcResolver for NoCalcResolver {
    fn resolve_calc(&self, _id: CalcId, _basis: Option<Scalar>) -> CalcResolution {
        CalcResolution::unresolved(false)
    }

    fn calc_depends_on_basis(&self, _id: CalcId) -> bool {
        false
    }
}
```

Then add these methods to existing value impls without changing enum variants yet:

```rust
impl Length {
    #[must_use]
    pub const fn depends_on_basis(self) -> bool {
        matches!(self, Self::Percent(_))
    }

    #[must_use]
    pub fn resolve_or_zero(self, basis: Option<Scalar>) -> Scalar {
        self.resolve_optional(basis).unwrap_or(0.0)
    }

    #[must_use]
    pub fn resolve_optional(self, basis: Option<Scalar>) -> Option<Scalar> {
        match self {
            Self::Normal => Some(0.0),
            Self::Px(value) => Some(value),
            Self::Percent(value) => basis.map(|basis| value * basis),
        }
    }

    #[must_use]
    pub fn resolve_with(
        self,
        basis: Option<Scalar>,
        _resolver: &(impl CalcResolver + ?Sized),
    ) -> Option<Scalar> {
        self.resolve_optional(basis)
    }
}
```

Mirror the same method names on `LengthAuto` and `Dimension` with current semantics:

```rust
impl LengthAuto {
    #[must_use]
    pub const fn depends_on_basis(self) -> bool {
        matches!(self, Self::Percent(_))
    }

    #[must_use]
    pub fn resolve_or_zero(self, basis: Option<Scalar>) -> Scalar {
        self.resolve_optional(basis).unwrap_or(0.0)
    }

    #[must_use]
    pub fn resolve_optional(self, basis: Option<Scalar>) -> Option<Scalar> {
        match self {
            Self::Px(value) => Some(value),
            Self::Percent(value) => basis.map(|basis| value * basis),
            Self::Auto => None,
        }
    }
}

impl Dimension {
    #[must_use]
    pub const fn depends_on_basis(self) -> bool {
        matches!(self, Self::Percent(_))
    }

    #[must_use]
    pub fn resolve_optional(self, basis: Option<Scalar>) -> Option<Scalar> {
        match self {
            Self::Px(value) => Some(value),
            Self::Percent(value) => basis.map(|basis| value * basis),
            Self::Fr(_) | Self::Auto | Self::MinContent | Self::MaxContent => None,
        }
    }
}
```

In `src/lib.rs`, extend the value re-export:

```rust
pub use value::{
    Available, CalcId, CalcResolution, CalcResolver, Dimension, Length, LengthAuto, NoCalcResolver,
};
```

- [ ] **Step 4: Run tests to verify pass**

Run:

```sh
cargo test -p surgeist-layout tests::layout_lengths_report_basis_dependency tests::layout_lengths_resolve_optional_basis_consistently tests::no_calc_resolver_keeps_plain_values_working
```

Expected: all three tests pass.

- [ ] **Step 5: Commit**

```sh
git add -- src/value.rs src/lib.rs src/tests.rs
git commit -m "layout: add typed calc resolver seam"
```

### Task 2: Centralize Layout Resolution Helpers

**Files:**
- Modify: `src/value.rs`
- Modify: `src/tests.rs`
- Modify: `src/compute.rs`
- Modify: `src/block.rs`
- Modify: `src/flex.rs`
- Modify: `src/grid/mod.rs`
- Modify: `src/grid/child.rs`
- Modify: `src/grid/tracks.rs`
- Modify: `src/grid/lanes.rs`
- Modify: `src/grid/subgrid.rs`

- [ ] **Step 1: Write failing track sizing dependency tests**

Add these tests to `src/tests.rs`:

```rust
#[test]
fn track_sizing_reports_basis_dependency() {
    assert!(!TrackSizing::px(12.0).depends_on_basis());
    assert!(TrackSizing::percent(0.25).depends_on_basis());
    assert!(TrackSizing::fit_content(Length::percent(0.25)).depends_on_basis());
    assert!(!TrackSizing::fr(1.0).depends_on_basis());
}

#[test]
fn track_sizing_definite_uses_shared_optional_basis_resolution() {
    let track = TrackSizing::percent(0.25);
    assert_eq!(track.min.definite(None), None);
    assert_eq!(track.min.definite(Some(80.0)), Some(20.0));
    assert_eq!(track.max.definite(None), None);
    assert_eq!(track.max.definite(Some(80.0)), Some(20.0));
}
```

- [ ] **Step 2: Run tests to verify failure**

Run:

```sh
cargo test -p surgeist-layout tests::track_sizing_reports_basis_dependency tests::track_sizing_definite_uses_shared_optional_basis_resolution
```

Expected: `track_sizing_reports_basis_dependency` fails to compile because track sizing types do not expose `depends_on_basis`.

- [ ] **Step 3: Add track sizing dependency methods**

In `src/value.rs`, update `MinTrackSizing::definite`, `MaxTrackSizing::definite`, and `MaxTrackSizing::fit_limit` to call `Length::resolve_optional`.

Add these methods:

```rust
impl MinTrackSizing {
    #[must_use]
    pub const fn depends_on_basis(self) -> bool {
        match self {
            Self::Length(length) => length.depends_on_basis(),
            Self::Auto | Self::MinContent | Self::MaxContent => false,
        }
    }
}

impl MaxTrackSizing {
    #[must_use]
    pub const fn depends_on_basis(self) -> bool {
        match self {
            Self::Length(length) | Self::FitContent(length) => length.depends_on_basis(),
            Self::Flex(_) | Self::Auto | Self::MinContent | Self::MaxContent => false,
        }
    }
}

impl TrackSizing {
    #[must_use]
    pub const fn depends_on_basis(self) -> bool {
        self.min.depends_on_basis() || self.max.depends_on_basis()
    }
}
```

- [ ] **Step 4: Replace duplicated helper bodies module by module**

For each module listed below, replace duplicated local helper bodies with calls to the layout value methods. Keep function names temporarily where that produces the smallest diff.

In `src/compute.rs`:

```rust
fn resolve_length_or_zero(length: super::Length, basis: Option<Scalar>) -> Scalar {
    length.resolve_or_zero(basis)
}

fn resolve_auto_or_zero(length: super::LengthAuto, basis: Option<Scalar>) -> Scalar {
    length.resolve_or_zero(basis)
}

fn resolve_dimension(dimension: super::Dimension, basis: Option<Scalar>) -> Option<Scalar> {
    dimension.resolve_optional(basis)
}
```

Apply the same pattern in:

```text
src/block.rs
src/flex.rs
src/grid/mod.rs
src/grid/child.rs
src/grid/lanes.rs
src/grid/subgrid.rs
```

For `src/grid/tracks.rs`, replace `resolve_length_optional(length, basis)` and direct `length.resolve(0.0)` fallback paths with `length.resolve_optional(basis)` and `length.resolve_or_zero(None)` when the current behavior explicitly treats px as definite without a basis.

- [ ] **Step 5: Run focused layout tests**

Run:

```sh
cargo test -p surgeist-layout tests::track_sizing_reports_basis_dependency tests::track_sizing_definite_uses_shared_optional_basis_resolution
cargo test -p surgeist-layout grid::tests::vertical_subgrid_percentage_gap_uses_flow_relative_axis_basis
cargo test -p surgeist-layout grid::tests::intrinsic_subgrid_context_is_needed_for_row_subgrid_with_percent_columns
```

Expected: all selected tests pass.

- [ ] **Step 6: Run broader layout tests**

Run:

```sh
cargo test -p surgeist-layout --lib
```

Expected: layout module tests pass.

- [ ] **Step 7: Commit**

```sh
git add -- src/value.rs src/tests.rs src/compute.rs src/block.rs src/flex.rs src/grid/mod.rs src/grid/child.rs src/grid/tracks.rs src/grid/lanes.rs src/grid/subgrid.rs
git commit -m "layout: centralize basis resolution"
```

### Task 3: Upstream Handoff - Style Calc AST and CSS Parsing

This task belongs to the `surgeist-style` and `surgeist-css` crate projects after the split. Do not execute or commit this task from the `surgeist-layout` repo. Use this section as an issue draft or handoff checklist for the owning crate coordinators; the layout crate should proceed only once the needed style-owned calc representation and lowering contract are available or mocked through a layout-local resolver test.

The authoritative split plans now live in:

- `../surgeist-style/plans/2026-06-21-surgeist-style-typed-calc-integration.md`
- `../surgeist-css/plans/2026-06-21-surgeist-css-calc-parsing-integration.md`

If any code snippet below disagrees with those crate-local plans, the crate-local plan wins. Layout workers should treat the rest of this task as historical monorepo source material only, not as an executable implementation recipe.

**Files:**
- Upstream handoff create: `../surgeist-style/src/calc.rs`
- Upstream handoff: `../surgeist-style/src/lib.rs`
- Upstream handoff: `../surgeist-style/src/value.rs`
- Upstream handoff: `../surgeist-style/src/declaration.rs`
- Upstream handoff: `../surgeist-css/src/lib.rs`

- [ ] **Step 1: Write failing style calc tests**

Create `../surgeist-style/src/calc.rs` with the implementation skeleton and tests in the same file:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calc_length_ast_reports_percentage_use() {
        let calc = CalcLength::sum([
            CalcLength::px(20.0),
            CalcLength::percent(10.0),
        ]);

        assert!(calc.uses_percentage());
        assert_eq!(
            calc.to_css_string(),
            "calc(20px + 10%)"
        );
    }

    #[test]
    fn calc_length_ast_rejects_non_finite_terms() {
        let error = CalcLength::try_px(f32::NAN).unwrap_err();
        assert_eq!(error.code(), crate::style::ErrorCode::InvalidValue);
    }
}
```

Add these CSS parser tests to the existing test module in `../surgeist-css/src/lib.rs`. If the file has no test module, create `#[cfg(test)] mod tests` at the end:

```rust
#[test]
fn parses_calc_width_as_style_calc_length() {
    let sheet = parse_sheet(".panel { width: calc(20px + 10%); }").unwrap();
    let declarations = &sheet.rules()[0].declarations;
    let value = declarations.get(crate::style::Property::Width).unwrap();

    match value {
        crate::style::Value::Length(crate::style::Length::Calc(calc)) => {
            assert!(calc.uses_percentage());
            assert_eq!(calc.to_css_string(), "calc(20px + 10%)");
        }
        other => panic!("expected calc length, got {other:?}"),
    }
}

#[test]
fn parses_nested_calc_width_with_subtraction() {
    let sheet = parse_sheet(".panel { width: calc(100% - calc(12px + 3%)); }").unwrap();
    let declarations = &sheet.rules()[0].declarations;
    let value = declarations.get(crate::style::Property::Width).unwrap();

    match value {
        crate::style::Value::Length(crate::style::Length::Calc(calc)) => {
            assert!(calc.uses_percentage());
            assert_eq!(calc.to_css_string(), "calc(100% - calc(12px + 3%))");
        }
        other => panic!("expected nested calc length, got {other:?}"),
    }
}
```

- [ ] **Step 2: Run tests to verify failure**

Run:

```sh
cargo test -p surgeist-style style::calc::tests::calc_length_ast_reports_percentage_use style::calc::tests::calc_length_ast_rejects_non_finite_terms
```

Expected: tests fail to compile because `style::calc`, `Length::Calc`, and CSS calc parsing do not exist.

- [ ] **Step 3: Implement authored calc AST**

In `../surgeist-style/src/calc.rs`, implement safe AST types:

```rust
use super::{Error, ErrorCode, Result};

#[derive(Clone, Debug, PartialEq)]
pub enum CalcLength {
    Px(f32),
    Percent(f32),
    Sum(Vec<CalcTerm>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum CalcTerm {
    Add(Box<CalcLength>),
    Sub(Box<CalcLength>),
}

impl CalcLength {
    #[must_use]
    pub const fn px(value: f32) -> Self {
        Self::Px(value)
    }

    pub fn try_px(value: f32) -> Result<Self> {
        if !value.is_finite() {
            return Err(Error::new(ErrorCode::InvalidValue, "calc px term must be finite"));
        }
        Ok(Self::Px(value))
    }

    #[must_use]
    pub const fn percent(value: f32) -> Self {
        Self::Percent(value)
    }

    pub fn try_percent(value: f32) -> Result<Self> {
        if !value.is_finite() {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "calc percent term must be finite",
            ));
        }
        Ok(Self::Percent(value))
    }

    #[must_use]
    pub fn sum(values: impl IntoIterator<Item = CalcLength>) -> Self {
        Self::Sum(
            values
                .into_iter()
                .map(|term| CalcTerm::Add(Box::new(term)))
                .collect(),
        )
    }

    #[must_use]
    pub fn uses_percentage(&self) -> bool {
        match self {
            Self::Px(_) => false,
            Self::Percent(_) => true,
            Self::Sum(terms) => terms.iter().any(CalcTerm::uses_percentage),
        }
    }

    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Px(value) => Self::try_px(*value).map(|_| ()),
            Self::Percent(value) => Self::try_percent(*value).map(|_| ()),
            Self::Sum(terms) => {
                for term in terms {
                    term.value().validate()?;
                }
                Ok(())
            }
        }
    }

    #[must_use]
    pub fn to_css_string(&self) -> String {
        match self {
            Self::Px(value) => format_number(*value) + "px",
            Self::Percent(value) => format_number(*value) + "%",
            Self::Sum(terms) => {
                let mut output = String::from("calc(");
                for (index, term) in terms.iter().enumerate() {
                    if index > 0 {
                        output.push(' ');
                    }
                    match term {
                        CalcTerm::Add(value) if index == 0 => output.push_str(&value.to_css_string()),
                        CalcTerm::Add(value) => {
                            output.push_str("+ ");
                            output.push_str(&value.to_css_string());
                        }
                        CalcTerm::Sub(value) => {
                            output.push_str("- ");
                            output.push_str(&value.to_css_string());
                        }
                    }
                }
                output.push(')');
                output
            }
        }
    }
}

impl CalcTerm {
    #[must_use]
    pub fn add(value: CalcLength) -> Self {
        Self::Add(Box::new(value))
    }

    #[must_use]
    pub fn sub(value: CalcLength) -> Self {
        Self::Sub(Box::new(value))
    }

    #[must_use]
    pub fn value(&self) -> &CalcLength {
        match self {
            Self::Add(value) | Self::Sub(value) => value,
        }
    }

    #[must_use]
    fn uses_percentage(&self) -> bool {
        self.value().uses_percentage()
    }
}

fn format_number(value: f32) -> String {
    let text = value.to_string();
    text.strip_suffix(".0").unwrap_or(&text).to_owned()
}
```

In `../surgeist-style/src/value.rs`, add `Calc(CalcLength)` to `style::Length`, update `validate`, and update all match expressions that must remain exhaustive.

In `../surgeist-style/src/lib.rs`, add:

```rust
mod calc;
pub use calc::{CalcLength, CalcTerm};
```

- [ ] **Step 4: Parse CSS calc into style values**

In `../surgeist-css/src/lib.rs`, extend `parse_length` to recognize the CSS `calc` function before the generic token branch:

```rust
Token::Function(name) if name.eq_ignore_ascii_case("calc") => {
    input.parse_nested_block(parse_calc_length).map(Length::Calc)
}
```

Add parser helpers near `parse_length`:

```rust
fn parse_calc_length<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<style::CalcLength, ParseError<'i, Error>> {
    let first = parse_calc_length_atom(input)?;
    let mut terms = vec![style::CalcTerm::add(first)];

    while !input.is_exhausted() {
        let location = input.current_source_location();
        let operator = input.next().map_err(basic)?;
        let value = parse_calc_length_atom(input)?;
        match operator {
            Token::Delim('+') => terms.push(style::CalcTerm::add(value)),
            Token::Delim('-') => terms.push(style::CalcTerm::sub(value)),
            token => return Err(location.new_unexpected_token_error::<Error>(token.clone())),
        }
    }

    Ok(style::CalcLength::Sum(terms))
}

fn parse_calc_length_atom<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<style::CalcLength, ParseError<'i, Error>> {
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Dimension { value, unit, .. } if unit.eq_ignore_ascii_case("px") => {
            style::CalcLength::try_px(*value).map_err(|error| error_at(location, error.to_string()))
        }
        Token::Percentage { unit_value, .. } => style::CalcLength::try_percent(*unit_value * 100.0)
            .map_err(|error| error_at(location, error.to_string())),
        Token::Number { value, .. } if *value == 0.0 => Ok(style::CalcLength::px(0.0)),
        Token::Function(name) if name.eq_ignore_ascii_case("calc") => {
            input.parse_nested_block(parse_calc_length)
        }
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}
```

If `cssparser` yields whitespace-separated delimiters differently in the current version, keep the same AST and tests, and adjust only the parser call shape.

- [ ] **Step 5: Update hashing and validation**

In `../surgeist-style/src/declaration.rs`, update `hash_length`:

```rust
fn hash_length(value: super::Length, state: &mut DefaultHasher) {
    match value {
        super::Length::Normal => 0_u8.hash(state),
        super::Length::Px(value) => {
            1_u8.hash(state);
            hash_f32(value, state);
        }
        super::Length::Percent(value) => {
            2_u8.hash(state);
            hash_f32(value, state);
        }
        super::Length::Fill => 3_u8.hash(state),
        super::Length::Fit => 4_u8.hash(state),
        super::Length::MinContent => 5_u8.hash(state),
        super::Length::MaxContent => 6_u8.hash(state),
        super::Length::Auto => 7_u8.hash(state),
        super::Length::Calc(calc) => {
            8_u8.hash(state);
            calc.to_css_string().hash(state);
        }
    }
}
```

In `../surgeist-style/src/value.rs`, ensure `Length::validate` delegates to `calc.validate()` for `Length::Calc(calc)`.

- [ ] **Step 6: Run tests**

Run:

```sh
cargo test -p surgeist-style style::calc::tests::calc_length_ast_reports_percentage_use style::calc::tests::calc_length_ast_rejects_non_finite_terms
cargo test -p surgeist-style
```

Expected: tests pass.

- [ ] **Step 7: Commit**

```sh
echo "commit upstream files in the owning crate repo"
git commit -m "style: add authored calc length values"
```

### Task 4: Coordinate Lowering Contract and Add Layout Handles

The layout-owned parts of this task are `src/value.rs`, `src/lib.rs`, `src/traits.rs`, and `src/tests.rs`. The `../surgeist-style/src/adapters/layout.rs` work belongs to the `surgeist-style` project and must be handed off; do not edit that sibling checkout from this repo. If the style adapter API is not ready, finish the layout resolver/store API with crate-local tests and report the exact adapter contract needed upstream.

**Files:**
- Modify: `src/value.rs`
- Modify: `src/lib.rs`
- Modify: `src/traits.rs`
- Modify: `src/tests.rs`
- Upstream handoff: `../surgeist-style/src/adapters/layout.rs`

- [ ] **Step 1: Write failing layout calc store tests**

Add to `src/tests.rs`:

```rust
#[test]
fn layout_calc_store_resolves_px_and_percent_terms() {
    let mut store = LayoutCalcStore::new();
    let id = store.push(CalcExpression::sum([
        CalcTerm::px(20.0),
        CalcTerm::percent(0.10),
    ]));

    assert!(store.calc_depends_on_basis(id));
    assert_eq!(store.resolve_calc(id, Some(200.0)).value, Some(40.0));
    assert_eq!(store.resolve_calc(id, None).value, None);
}

#[test]
fn length_calc_resolves_through_resolver_hook() {
    let mut store = LayoutCalcStore::new();
    let id = store.push(CalcExpression::sum([
        CalcTerm::px(5.0),
        CalcTerm::percent(0.25),
    ]));

    assert!(Length::calc(id).depends_on_basis_with(&store));
    assert_eq!(Length::calc(id).resolve_with(Some(20.0), &store), Some(10.0));
}
```

Add to `../surgeist-style/src/adapters/layout.rs` tests:

```rust
#[test]
fn lower_calc_width_normalizes_percent_once() {
    let sheet = crate::css::parse_sheet(".panel { width: calc(20px + 10%); }").unwrap();
    let mut resolver = crate::style::Resolver::new(sheet);
    let tree = adapter_test_tree("panel");
    let resolved = resolver
        .resolve(crate::style::Context::new(&tree, tree.root()))
        .unwrap();

    let lowered = lower_with_store(&resolved).unwrap();
    let id = match lowered.node.size.width {
        layout::Dimension::Calc(id) => id,
        other => panic!("expected calc dimension, got {other:?}"),
    };

    assert_eq!(lowered.calc_store.resolve_calc(id, Some(200.0)).value, Some(40.0));
}
```

If `style/adapters/layout.rs` has no test harness tree, create a minimal local test tree that implements `crate::style::Tree` with one root node whose class list contains `panel`.

- [ ] **Step 2: Run tests to verify failure**

Run:

```sh
cargo test -p surgeist-layout tests::layout_calc_store_resolves_px_and_percent_terms tests::length_calc_resolves_through_resolver_hook
```

Expected: tests fail to compile because `LayoutCalcStore`, `CalcExpression`, layout calc variants, and `lower_with_store` do not exist.

- [ ] **Step 3: Add safe layout calc store**

In `src/value.rs`, add:

```rust
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LayoutCalcStore {
    expressions: Vec<CalcExpression>,
}

impl LayoutCalcStore {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            expressions: Vec::new(),
        }
    }

    pub fn push(&mut self, expression: CalcExpression) -> CalcId {
        let id = CalcId::new(self.expressions.len() as u32);
        self.expressions.push(expression);
        id
    }

    #[must_use]
    pub fn get(&self, id: CalcId) -> Option<&CalcExpression> {
        self.expressions.get(id.index() as usize)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.expressions.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.expressions.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CalcExpression {
    terms: Vec<CalcTerm>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CalcTerm {
    Px(Scalar),
    Percent(Scalar),
}

impl CalcExpression {
    #[must_use]
    pub fn sum(terms: impl IntoIterator<Item = CalcTerm>) -> Self {
        Self {
            terms: terms.into_iter().collect(),
        }
    }

    #[must_use]
    pub fn depends_on_basis(&self) -> bool {
        self.terms.iter().any(|term| matches!(term, CalcTerm::Percent(_)))
    }

    #[must_use]
    pub fn resolve(&self, basis: Option<Scalar>) -> CalcResolution {
        let mut total = 0.0;
        for term in &self.terms {
            match *term {
                CalcTerm::Px(value) => total += value,
                CalcTerm::Percent(value) => {
                    let Some(basis) = basis else {
                        return CalcResolution::unresolved(true);
                    };
                    total += value * basis;
                }
            }
        }
        CalcResolution::definite(total, self.depends_on_basis())
    }
}

impl CalcTerm {
    #[must_use]
    pub const fn px(value: Scalar) -> Self {
        Self::Px(value)
    }

    #[must_use]
    pub const fn percent(value: Scalar) -> Self {
        Self::Percent(value)
    }
}

impl CalcResolver for LayoutCalcStore {
    fn resolve_calc(&self, id: CalcId, basis: Option<Scalar>) -> CalcResolution {
        self.get(id)
            .map_or(CalcResolution::unresolved(false), |expression| expression.resolve(basis))
    }

    fn calc_depends_on_basis(&self, id: CalcId) -> bool {
        self.get(id).is_some_and(CalcExpression::depends_on_basis)
    }
}
```

Add `Calc(CalcId)` variants to layout `Length`, `LengthAuto`, and `Dimension`, then update methods:

```rust
Self::Calc(id) => resolver.resolve_calc(id, basis).value
```

For dependency checks, add resolver-aware methods:

```rust
pub fn depends_on_basis_with(self, resolver: &(impl CalcResolver + ?Sized)) -> bool {
    match self {
        Self::Percent(_) => true,
        Self::Calc(id) => resolver.calc_depends_on_basis(id),
        Self::Normal | Self::Px(_) => false,
    }
}
```

Keep the existing resolver-free `depends_on_basis` as the conservative form:

```rust
Self::Percent(_) | Self::Calc(_) => true
```

- [ ] **Step 4: Expose the layout-pass calc resolver through the compute trait**

In `src/traits.rs`, import calc resolver types:

```rust
use super::{CalcResolver, ComputeInput, ComputeOutput, NoCalcResolver, NodeInput, NodeOutput};
```

Then add the resolver hook to `Compute`:

```rust
pub trait Compute: Traverse {
    fn node_input(&self, node: Self::Node) -> &NodeInput;

    fn calc_resolver(&self) -> &dyn CalcResolver {
        &NoCalcResolver
    }

    fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput);
    fn compute_child(&mut self, node: Self::Node, input: ComputeInput) -> ComputeOutput;
}
```

`NodeInput` remains per-node normalized style data. Do not add `LayoutCalcStore`, `LayoutCalcResolver`, `CalcArena`, or any other layout-pass store field to `NodeInput`.

- [ ] **Step 5: Lower style calc into layout calc handles**

In `../surgeist-style/src/adapters/layout.rs`, add an output type and lowering session:

This step is a handoff preview only. The authoritative style implementation recipe is `../surgeist-style/plans/2026-06-21-surgeist-style-typed-calc-integration.md`; keep the public contract names synchronized with that plan: `LayoutLoweringOutput`, `LayoutLoweringSession`, `lower_with_store`, `CalcExpression`, `CalcTerm`, and `LayoutCalcStore::push`.

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct LayoutLoweringOutput {
    pub node: layout::NodeInput,
    pub calc_store: layout::LayoutCalcStore,
}

#[derive(Clone, Debug, Default)]
pub struct LayoutLoweringSession {
    calc_store: layout::LayoutCalcStore,
}

impl LayoutLoweringSession {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn lower_node(&mut self, resolved: &Resolved) -> Result<layout::NodeInput> {
        lower_node_with_session(resolved, self)
    }

    #[must_use]
    pub fn finish(self) -> layout::LayoutCalcStore {
        self.calc_store
    }

    fn lower_calc_length(&mut self, calc: &crate::style::CalcLength) -> layout::CalcId {
        let expression = self.lower_calc_expression(calc);
        self.calc_store.push(expression)
    }

    fn lower_calc_expression(&self, calc: &crate::style::CalcLength) -> layout::CalcExpression {
        match calc {
            crate::style::CalcLength::Px(value) => {
                layout::CalcExpression::sum([layout::CalcTerm::px(*value)])
            }
            crate::style::CalcLength::Percent(value) => {
                layout::CalcExpression::sum([layout::CalcTerm::percent(percent(*value))])
            }
            crate::style::CalcLength::Sum(terms) => {
                let mut lowered = Vec::new();
                for term in terms {
                    let sign = match term.operator {
                        crate::style::CalcOperator::Add => 1.0,
                        crate::style::CalcOperator::Sub => -1.0,
                    };
                    collect_calc_terms(&term.value, sign, &mut lowered);
                }
                layout::CalcExpression::sum(lowered)
            }
        }
    }
}

fn collect_calc_terms(
    calc: &crate::style::CalcLength,
    sign: f32,
    output: &mut Vec<layout::CalcTerm>,
) {
    match calc {
        crate::style::CalcLength::Px(value) => output.push(layout::CalcTerm::px(sign * *value)),
        crate::style::CalcLength::Percent(value) => {
            output.push(layout::CalcTerm::percent(sign * percent(*value)));
        }
        crate::style::CalcLength::Sum(terms) => {
            for term in terms {
                let term_sign = match term.operator {
                    crate::style::CalcOperator::Add => sign,
                    crate::style::CalcOperator::Sub => -sign,
                };
                collect_calc_terms(&term.value, term_sign, output);
            }
        }
    }
}
```

Keep the existing calc-free convenience API available for current callers and add the store-returning API. The calc-free `lower(resolved)` must reject calc-bearing resolved values instead of discarding a calc store; calc-bearing callers use `lower_with_store(resolved)`.

```rust
pub fn lower(resolved: &Resolved) -> Result<layout::NodeInput> {
    if resolved_uses_calc(resolved) {
        return Err(unsupported("calc values require lower_with_store"));
    }
    let mut session = LayoutLoweringSession::new();
    session.lower_node(resolved)
}

pub fn lower_with_store(resolved: &Resolved) -> Result<LayoutLoweringOutput> {
    let mut session = LayoutLoweringSession::new();
    let node = session.lower_node(resolved)?;
    Ok(LayoutLoweringOutput {
        node,
        calc_store: session.finish(),
    })
}
```

Rename the current `lower` body to `lower_node_with_session(resolved, session)` and update `lower_dimension`, `lower_length_auto`, `lower_length`, and `lower_gap_length` to accept `&mut LayoutLoweringSession` and map `style::Length::Calc(calc)` to the corresponding layout calc variant.

Ensure `percent(value)` remains the only place CSS percent `10.0` becomes layout factor `0.10`.

- [ ] **Step 6: Run tests**

Run:

```sh
cargo test -p surgeist-layout tests::layout_calc_store_resolves_px_and_percent_terms tests::length_calc_resolves_through_resolver_hook
```

Expected: tests pass.

- [ ] **Step 7: Commit**

```sh
git add -- src/value.rs src/lib.rs src/traits.rs src/tests.rs
git commit -m "layout: add calc value handles"
```

### Task 5: Make Calc Participate in Flex and Grid Basis Logic

**Files:**
- Modify: `src/value.rs`
- Modify: `src/tests.rs`
- Modify: `src/traits.rs`
- Modify: `src/flex.rs`
- Modify: `src/grid/mod.rs`
- Modify: `src/grid/tracks.rs`
- Modify: `src/grid/lanes.rs`

- [ ] **Step 1: Write failing value tests for calc percentage behavior**

Add to `src/tests.rs`:

```rust
#[test]
fn calc_percent_track_participates_in_percent_detection() {
    let mut store = LayoutCalcStore::new();
    let id = store.push(CalcExpression::sum([
        CalcTerm::px(20.0),
        CalcTerm::percent(0.10),
    ]));
    let track = TrackSizing::new(
        MinTrackSizing::Length(Length::calc(id)),
        MaxTrackSizing::Length(Length::px(80.0)),
    );

    assert!(track.depends_on_basis_with(&store));
    assert_eq!(track.percent_fraction_with(&store), 0.10);
}

#[test]
fn calc_px_only_track_does_not_request_percent_rerun() {
    let mut store = LayoutCalcStore::new();
    let id = store.push(CalcExpression::sum([CalcTerm::px(20.0), CalcTerm::px(10.0)]));
    let track = TrackSizing::new(
        MinTrackSizing::Length(Length::calc(id)),
        MaxTrackSizing::Length(Length::px(80.0)),
    );

    assert!(!track.depends_on_basis_with(&store));
    assert_eq!(track.percent_fraction_with(&store), 0.0);
}
```

- [ ] **Step 2: Run tests to verify failure**

Run:

```sh
cargo test -p surgeist-layout tests::calc_percent_track_participates_in_percent_detection tests::calc_px_only_track_does_not_request_percent_rerun
```

Expected: tests fail to compile because resolver-aware track methods do not exist.

- [ ] **Step 3: Add resolver-aware track methods**

In `src/value.rs`, add:

```rust
impl Length {
    #[must_use]
    pub fn percent_fraction_with(self, resolver: &(impl CalcResolver + ?Sized)) -> Scalar {
        match self {
            Self::Percent(value) => value,
            Self::Calc(id) => resolver
                .calc_percent_fraction(id)
                .unwrap_or_else(|| if resolver.calc_depends_on_basis(id) { 1.0 } else { 0.0 }),
            Self::Normal | Self::Px(_) => 0.0,
        }
    }
}
```

Add this method to the `CalcResolver` trait:

```rust
fn calc_percent_fraction(&self, id: CalcId) -> Option<Scalar>;
```

Implement it for `NoCalcResolver` as `None` and for `LayoutCalcStore` by summing the absolute percent terms in the expression. This is conservative for grid intrinsic distribution and keeps `calc(20px - 10%)` basis-dependent:

```rust
impl CalcExpression {
    #[must_use]
    pub fn percent_fraction(&self) -> Scalar {
        self.terms
            .iter()
            .filter_map(|term| match *term {
                CalcTerm::Percent(value) => Some(value.abs()),
                CalcTerm::Px(_) => None,
            })
            .sum()
    }
}
```

Add resolver-aware methods on `MinTrackSizing`, `MaxTrackSizing`, and `TrackSizing`:

```rust
impl MinTrackSizing {
    #[must_use]
    pub fn depends_on_basis_with(self, resolver: &(impl CalcResolver + ?Sized)) -> bool {
        match self {
            Self::Length(length) => length.depends_on_basis_with(resolver),
            Self::Auto | Self::MinContent | Self::MaxContent => false,
        }
    }

    #[must_use]
    pub fn percent_fraction_with(self, resolver: &(impl CalcResolver + ?Sized)) -> Scalar {
        match self {
            Self::Length(length) => length.percent_fraction_with(resolver),
            Self::Auto | Self::MinContent | Self::MaxContent => 0.0,
        }
    }
}

impl MaxTrackSizing {
    #[must_use]
    pub fn depends_on_basis_with(self, resolver: &(impl CalcResolver + ?Sized)) -> bool {
        match self {
            Self::Length(length) | Self::FitContent(length) => {
                length.depends_on_basis_with(resolver)
            }
            Self::Flex(_) | Self::Auto | Self::MinContent | Self::MaxContent => false,
        }
    }

    #[must_use]
    pub fn percent_fraction_with(self, resolver: &(impl CalcResolver + ?Sized)) -> Scalar {
        match self {
            Self::Length(length) | Self::FitContent(length) => {
                length.percent_fraction_with(resolver)
            }
            Self::Flex(_) | Self::Auto | Self::MinContent | Self::MaxContent => 0.0,
        }
    }
}

impl TrackSizing {
    #[must_use]
    pub fn depends_on_basis_with(self, resolver: &(impl CalcResolver + ?Sized)) -> bool {
        self.min.depends_on_basis_with(resolver) || self.max.depends_on_basis_with(resolver)
    }

    #[must_use]
    pub fn percent_fraction_with(self, resolver: &(impl CalcResolver + ?Sized)) -> Scalar {
        self.min
            .percent_fraction_with(resolver)
            .max(self.max.percent_fraction_with(resolver))
    }
}
```

For `MaxTrackSizing::FitContent(limit)`, include `limit.percent_fraction_with(resolver)`.

- [ ] **Step 4: Replace exact percent checks in flex**

In `src/flex.rs`, find direct checks:

```sh
rg -n "Dimension::Percent|Length::Percent|depends_on_basis|resolve_dimension" src/flex.rs
```

Replace rerun/intrinsic checks such as:

```rust
matches!(style.size.height, Dimension::Percent(_))
```

with resolver-aware checks:

```rust
style.size.height.depends_on_basis_with(constants.resolver)
```

Update exact flex structures and signatures in `src/flex.rs`:

```rust
struct Constants<'a> {
    resolver: &'a dyn CalcResolver,
    direction: FlexDirection,
    layout_direction: Direction,
    node_outer_size: Size<Option<Scalar>>,
    node_inner_size: Size<Option<Scalar>>,
    min_outer_size: Size<Option<Scalar>>,
    max_outer_size: Size<Option<Scalar>>,
    max_inner_size: Size<Option<Scalar>>,
    border: Edges,
    padding_border_size: Size,
    scrollbar_gutter: Point,
    content_box_inset: Edges,
    gap: Size,
    align_items: AlignItems,
    align_content: AlignContent,
    justify_content: AlignContent,
    wraps: bool,
    wrap_reverse: bool,
    available: Size<Available>,
    available_main: Available,
}

// Existing impl block; change the constructor signature to this:
fn new(style: &NodeInput, input: ComputeInput, resolver: &'a dyn CalcResolver) -> Self
```

Inside the current `Constants::new` body, store `resolver` in the returned `Constants` value and replace every `resolve_length_or_zero`, `resolve_auto_or_zero`, and `resolve_dimension` call with the resolver-aware layout value method. For example:

```rust
let padding = style
    .padding
    .zip_inline_size(input.parent, |length, basis| {
        length.resolve_with(basis, resolver).unwrap_or(0.0)
    });
let style_size = style
    .size
    .zip_map(input.parent, |dimension, basis| {
        dimension.resolve_with(basis, resolver)
    })
    .apply_aspect_ratio(style.aspect_ratio)
    .add_optional(box_sizing_adjustment);
```

Update call sites:

```rust
pub fn compute_flex<Tree>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: ComputeInput,
) -> ComputeOutput
where
    Tree: Compute,
{
    let style = tree.node_input(node).clone();
    let constants = Constants::new(&style, input, tree.calc_resolver());
    let mut items = collect_items(tree, node, &constants, input.run_mode);
    let mut lines = collect_flex_lines(&items, &constants);
    let mut layout_constants =
        resolved_layout_constants(tree, input, &style, &constants, &mut items, &lines);
    resolve_lines(tree, &mut items, &mut lines, &layout_constants);
    let cross_layout_constants = resolved_cross_layout_constants(&layout_constants, &lines);
    let layout_constants = if cross_layout_constants.node_inner_size != layout_constants.node_inner_size {
        resolve_lines(tree, &mut items, &mut lines, &cross_layout_constants);
        cross_layout_constants
    } else {
        cross_layout_constants
    };
    let absolute_content_size = if input.run_mode.is_perform_layout() {
        final_layout(tree, &mut items, &layout_constants);
        let absolute_content_size = layout_absolute_children(tree, node, &layout_constants);
        layout_hidden_children(tree, node);
        absolute_content_size
    } else {
        Size::ZERO
    };
    container_output(input, &style, &layout_constants, &items, &lines, absolute_content_size)
}

fn build_item<Tree>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    order: u32,
    style: &NodeInput,
    constants: &Constants<'_>,
    run_mode: RunMode,
) -> FlexItem<<Tree as Traverse>::Node>
where
    Tree: Compute,
```

Change local helpers in `flex.rs` to accept the resolver explicitly:

```rust
fn resolve_dimension(
    dimension: Dimension,
    basis: Option<Scalar>,
    resolver: &dyn CalcResolver,
) -> Option<Scalar> {
    dimension.resolve_with(basis, resolver)
}
```

- [ ] **Step 5: Replace exact percent checks in grid**

In `src/grid/mod.rs`, update the container constants and track input structs:

```rust
struct Constants<'a> {
    resolver: &'a dyn CalcResolver,
    node_outer_size: Size<Option<Scalar>>,
    node_inner_size: Size<Option<Scalar>>,
    node_min_size: Size<Option<Scalar>>,
    node_max_size: Size<Option<Scalar>>,
    available_inner_size: Size<Option<Scalar>>,
    content_box_inset: Edges,
    padding: Edges,
    border: Edges,
}

// Existing impl block; change the constructor signature to this:
fn new(style: &NodeInput, input: ComputeInput, resolver: &'a dyn CalcResolver) -> Self
```

Inside the current grid `Constants::new` body, store `resolver` in the returned `Constants` value and replace local helper calls with resolver-aware methods:

```rust
let padding = style
    .padding
    .zip_inline_size(input.parent, |length, basis| {
        length.resolve_with(basis, resolver).unwrap_or(0.0)
    });
let style_size = if input.sizing_mode == SizingMode::InherentSize {
    style
        .size
        .zip_map(input.parent, |dimension, basis| {
            dimension.resolve_with(basis, resolver)
        })
        .apply_aspect_ratio(style.aspect_ratio)
        .add_optional(box_sizing_adjustment)
} else {
    Size::NONE
};
```

Thread the resolver through exact grid input structs:

```rust
struct GridTrackResolutionInput<'a, Node> {
    resolver: &'a dyn CalcResolver,
    tree: &'a mut dyn Compute<Node = Node>,
    node: Node,
    style: &'a NodeInput,
    constants: &'a Constants<'a>,
    column_tracks: &'a [TrackSizing],
    row_tracks: &'a [TrackSizing],
}

struct InlineTrackInput<'a> {
    resolver: &'a dyn CalcResolver,
    tracks: &'a [TrackSizing],
    basis: Option<Scalar>,
    definite_size: Option<Scalar>,
    available_size: Option<Scalar>,
    gap: Scalar,
    alignment: AlignContent,
    intrinsic_sizes: &'a [Scalar],
}
```

Replace `track_needs_layout_width_resolution` and `track_needs_layout_height_resolution` with resolver-aware signatures:

```rust
fn track_needs_layout_width_resolution(
    track: &TrackSizing,
    resolver: &dyn CalcResolver,
) -> bool {
    track.max.depends_on_basis_with(resolver) || track.min.depends_on_basis_with(resolver)
}

fn track_needs_layout_height_resolution(
    track: &TrackSizing,
    resolver: &dyn CalcResolver,
) -> bool {
    track.max.depends_on_basis_with(resolver) || track.min.depends_on_basis_with(resolver)
}
```

In `src/grid/tracks.rs`, replace percent helpers with resolver-aware forms:

```rust
pub(super) fn track_has_percent_sizing(track: &TrackSizing, resolver: &dyn CalcResolver) -> bool {
    track.depends_on_basis_with(resolver)
}

pub(super) fn track_percent_sum(tracks: &[TrackSizing], resolver: &dyn CalcResolver) -> Scalar {
    tracks
        .iter()
        .map(|track| track.percent_fraction_with(resolver))
        .sum::<Scalar>()
}
```

Update exact tracks signatures:

```rust
pub(super) fn resolve_tracks(
    tracks: &[TrackSizing],
    basis: Option<Scalar>,
    gap: Scalar,
    alignment: AlignContent,
    intrinsic_sizes: &[Scalar],
    resolver: &dyn CalcResolver,
) -> Vec<Scalar>

pub(super) fn resolve_tracks_with_intrinsics(
    tracks: &[TrackSizing],
    basis: Option<Scalar>,
    gap: Scalar,
    alignment: AlignContent,
    min_intrinsic_sizes: &[Scalar],
    max_intrinsic_sizes: &[Scalar],
    resolver: &dyn CalcResolver,
) -> Vec<Scalar>

pub(super) fn distribute_intrinsic_span(
    sizes: &mut [Scalar],
    tracks: &[TrackSizing],
    kind: IntrinsicSpanContribution,
    percent_basis: Option<Scalar>,
    contribution: Scalar,
    resolver: &dyn CalcResolver,
)
```

In `src/grid/lanes.rs`, update lane sizing inputs that inspect tracks:

```rust
pub resolver: &'a dyn CalcResolver,
```

Pass `tree.calc_resolver()` from `compute_grid`, through `Constants::new`, into `GridTrackResolutionInput`, `InlineTrackInput`, `IntrinsicGrid`, and `LaneIntrinsicSizingInput`. Do not construct or clone a resolver inside grid modules.

- [ ] **Step 6: Add focused flex/grid calc tests**

Add a flex test in `src/flex.rs` tests or the existing flex test module:

```rust
#[test]
fn flex_percent_dependent_calc_size_requests_definite_cross_rerun() {
    let mut store = LayoutCalcStore::new();
    let height = store.push(CalcExpression::sum([
        CalcTerm::px(10.0),
        CalcTerm::percent(0.50),
    ]));
    let mut child = NodeInput::DEFAULT;
    child.size.height = Dimension::calc(height);

    assert!(child.size.height.depends_on_basis_with(&store));
}
```

Add a grid test in `src/grid/tests.rs`:

```rust
#[test]
fn grid_calc_percent_track_needs_layout_resolution() {
    let mut store = LayoutCalcStore::new();
    let id = store.push(CalcExpression::sum([
        CalcTerm::px(20.0),
        CalcTerm::percent(0.10),
    ]));
    let track = TrackSizing::new(
        MinTrackSizing::Length(Length::calc(id)),
        MaxTrackSizing::Length(Length::px(100.0)),
    );

    assert!(track.depends_on_basis_with(&store));
    assert_eq!(track.percent_fraction_with(&store), 0.10);
}
```

- [ ] **Step 7: Run focused tests**

Run:

```sh
cargo test -p surgeist-layout tests::calc_percent_track_participates_in_percent_detection tests::calc_px_only_track_does_not_request_percent_rerun
cargo test -p surgeist-layout grid::tests::grid_calc_percent_track_needs_layout_resolution
cargo test -p surgeist-layout flex::
cargo test -p surgeist-layout grid::
```

Expected: selected tests pass, and existing flex/grid tests keep passing.

- [ ] **Step 8: Commit**

```sh
git add -- src/value.rs src/tests.rs src/traits.rs src/flex.rs src/grid/mod.rs src/grid/tracks.rs src/grid/lanes.rs
git commit -m "layout: route calc through percent-dependent sizing"
```

### Task 6: Upstream Handoff - Open Property and Typed Value Descriptor Boundary

This task belongs to `surgeist-style` after the split. It is retained here only because it affects the overall typed-value roadmap and future layout adapter inputs. Do not execute or commit this task from the `surgeist-layout` repo.

**Files:**
- Upstream handoff: `../surgeist-style/src/property.rs`
- Upstream handoff: `../surgeist-style/src/value.rs`
- Upstream handoff: `../surgeist-style/src/declaration.rs`
- Upstream handoff: `../surgeist-style/src/resolver.rs`
- Upstream handoff: `../surgeist-style/src/lib.rs`

- [ ] **Step 1: Write failing property descriptor tests**

Add tests to the existing `#[cfg(test)]` module in `../surgeist-style/src/property.rs` or create one at the bottom:

```rust
#[test]
fn built_in_property_has_stable_open_property_id() {
    let id = PropertyId::from(Property::Width);
    assert_eq!(id.name(), "width");
    assert_eq!(Property::try_from(id), Ok(Property::Width));
}

#[test]
fn custom_property_descriptor_carries_metadata_without_value_enum_variant() {
    let descriptor = PropertyDescriptor::custom(
        PropertyId::custom("--studio-accent"),
        ValueKind::Color,
        Metadata::new(Value::Color(Color::BLACK)).impact(Impact::empty().paint()),
    )
    .unwrap();

    assert_eq!(descriptor.id().name(), "--studio-accent");
    assert_eq!(descriptor.kind(), ValueKind::Color);
    assert_eq!(descriptor.metadata().impact, Impact::empty().paint());
}
```

Add a value-lane test in `../surgeist-style/src/value.rs`:

```rust
#[test]
fn custom_typed_value_preserves_property_identity() {
    let property = PropertyId::custom("--studio-spacing");
    let value = TypedValue::custom(
        property.clone(),
        ValueKind::Length,
        CustomValue::Length(Length::px(8.0)),
    );

    assert_eq!(value.property_id(), &property);
    assert_eq!(value.kind(), ValueKind::Length);
}
```

Add declaration and resolver integration tests to `../surgeist-style/src/declaration.rs` and `../surgeist-style/src/resolver.rs`:

```rust
#[test]
fn declarations_fingerprint_includes_custom_property_identity_and_value() {
    let first = Declarations::new()
        .custom(
            PropertyId::custom("--studio-spacing"),
            ValueKind::Length,
            CustomValue::Length(Length::px(8.0)),
        )
        .unwrap();
    let second = Declarations::new()
        .custom(
            PropertyId::custom("--studio-spacing"),
            ValueKind::Length,
            CustomValue::Length(Length::px(12.0)),
        )
        .unwrap();
    let third = Declarations::new()
        .custom(
            PropertyId::custom("--studio-gap"),
            ValueKind::Length,
            CustomValue::Length(Length::px(8.0)),
        )
        .unwrap();

    assert_ne!(first.fingerprint(), second.fingerprint());
    assert_ne!(first.fingerprint(), third.fingerprint());
}

#[test]
fn resolver_preserves_custom_typed_declarations_in_snapshot() {
    let id = Id::new(1, 0);
    let tree = TestTree::new([id]);
    let local = Declarations::new()
        .custom(
            PropertyId::custom("--studio-spacing"),
            ValueKind::Length,
            CustomValue::Length(Length::px(8.0)),
        )
        .unwrap();
    let mut resolver = Resolver::new(Sheet::new());

    let resolved = resolver
        .resolve(Context::new(&tree, id).local(&local))
        .unwrap();

    assert_eq!(
        resolved
            .get_typed(&PropertyId::custom("--studio-spacing"))
            .unwrap()
            .custom_value(),
        Some(&CustomValue::Length(Length::px(8.0)))
    );
}
```

- [ ] **Step 2: Run tests to verify failure**

Run:

```sh
cargo test -p surgeist-style
cargo test -p surgeist-style
```

Expected: tests fail to compile because `PropertyId`, `PropertyDescriptor`, `ValueKind`, `TypedValue`, `CustomValue`, declaration custom insertion, and resolved typed lookup do not exist.

- [ ] **Step 3: Add open property IDs and descriptors**

In `../surgeist-style/src/property.rs`, add:

```rust
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PropertyId {
    BuiltIn(Property),
    Custom(String),
}

impl PropertyId {
    #[must_use]
    pub fn custom(name: impl Into<String>) -> Self {
        Self::Custom(name.into())
    }

    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::BuiltIn(property) => property.name(),
            Self::Custom(name) => name,
        }
    }
}

impl From<Property> for PropertyId {
    fn from(property: Property) -> Self {
        Self::BuiltIn(property)
    }
}

impl TryFrom<PropertyId> for Property {
    type Error = Error;

    fn try_from(value: PropertyId) -> Result<Self> {
        match value {
            PropertyId::BuiltIn(property) => Ok(property),
            PropertyId::Custom(name) => Err(Error::new(
                ErrorCode::InvalidProperty,
                format!("custom property `{name}` is not a built-in property"),
            )),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValueKind {
    Keyword,
    Display,
    Number,
    Length,
    Edges,
    Color,
    Grid,
    Text,
    Transform,
    Custom,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PropertyDescriptor {
    id: PropertyId,
    kind: ValueKind,
    metadata: Metadata,
}

impl PropertyDescriptor {
    pub fn built_in(property: Property) -> Self {
        let metadata = property.metadata();
        Self {
            id: property.into(),
            kind: metadata.default.kind(),
            metadata,
        }
    }

    pub fn custom(id: PropertyId, kind: ValueKind, metadata: Metadata) -> Result<Self> {
        match id {
            PropertyId::Custom(_) => Ok(Self { id, kind, metadata }),
            PropertyId::BuiltIn(property) => Err(Error::new(
                ErrorCode::InvalidProperty,
                format!("custom descriptor cannot use built-in property `{property:?}`"),
            )),
        }
    }

    #[must_use]
    pub fn id(&self) -> &PropertyId {
        &self.id
    }

    #[must_use]
    pub const fn kind(&self) -> ValueKind {
        self.kind
    }

    #[must_use]
    pub const fn metadata(&self) -> &Metadata {
        &self.metadata
    }
}
```

Add `Property::name()` with canonical kebab names used by CSS:

```rust
#[must_use]
pub const fn name(self) -> &'static str {
    match self {
        Self::Display => "display",
        Self::BoxSizing => "box-sizing",
        Self::Position => "position",
        Self::Inset => "inset",
        Self::Width => "width",
        Self::Height => "height",
        Self::MinWidth => "min-width",
        Self::MinHeight => "min-height",
        Self::MinSize => "min-size",
        Self::MaxWidth => "max-width",
        Self::MaxHeight => "max-height",
        Self::MaxSize => "max-size",
        Self::AspectRatio => "aspect-ratio",
        Self::Margin => "margin",
        Self::Padding => "padding",
        Self::Overflow => "overflow",
        Self::OverflowX => "overflow-x",
        Self::OverflowY => "overflow-y",
        Self::ScrollbarWidth => "scrollbar-width",
        Self::ZIndex => "z-index",
        Self::Direction => "direction",
        Self::WritingMode => "writing-mode",
        Self::TextAlign => "text-align",
        Self::Float => "float",
        Self::Clear => "clear",
        Self::FlexDirection => "flex-direction",
        Self::FlexWrap => "flex-wrap",
        Self::FlexGrow => "flex-grow",
        Self::FlexShrink => "flex-shrink",
        Self::FlexBasis => "flex-basis",
        Self::Align => "align",
        Self::AlignItems => "align-items",
        Self::AlignSelf => "align-self",
        Self::AlignContent => "align-content",
        Self::Justify => "justify",
        Self::JustifyItems => "justify-items",
        Self::JustifySelf => "justify-self",
        Self::JustifyContent => "justify-content",
        Self::Gap => "gap",
        Self::RowGap => "row-gap",
        Self::ColumnGap => "column-gap",
        Self::GridTemplateRows => "grid-template-rows",
        Self::GridTemplateColumns => "grid-template-columns",
        Self::GridTemplateAreas => "grid-template-areas",
        Self::GridTemplate => "grid-template",
        Self::GridAutoRows => "grid-auto-rows",
        Self::GridAutoColumns => "grid-auto-columns",
        Self::GridAutoFlow => "grid-auto-flow",
        Self::GridFlowTolerance => "grid-flow-tolerance",
        Self::GridRowStart => "grid-row-start",
        Self::GridRowEnd => "grid-row-end",
        Self::GridColumnStart => "grid-column-start",
        Self::GridColumnEnd => "grid-column-end",
        Self::GridRow => "grid-row",
        Self::GridColumn => "grid-column",
        Self::GridArea => "grid-area",
        Self::Grid => "grid",
        Self::Background => "background",
        Self::Foreground => "foreground",
        Self::Color => "color",
        Self::BorderColor => "border-color",
        Self::BorderWidth => "border-width",
        Self::BorderStyle => "border-style",
        Self::Radius => "radius",
        Self::Shadow => "shadow",
        Self::Opacity => "opacity",
        Self::Visibility => "visibility",
        Self::FontFamily => "font-family",
        Self::FontSize => "font-size",
        Self::FontWeight => "font-weight",
        Self::FontStyle => "font-style",
        Self::LineHeight => "line-height",
        Self::TextWrap => "text-wrap",
        Self::WhiteSpace => "white-space",
        Self::WordBreak => "word-break",
        Self::OverflowWrap => "overflow-wrap",
        Self::TextOverflow => "text-overflow",
        Self::TextDecoration => "text-decoration",
        Self::SelectionColor => "selection-color",
        Self::Cursor => "cursor",
        Self::PointerEvents => "pointer-events",
        Self::FocusOutline => "focus-outline",
        Self::SelectionPaint => "selection-paint",
        Self::Transform => "transform",
        Self::TransformOrigin => "transform-origin",
        Self::Filter => "filter",
        Self::TransitionProperty => "transition-property",
        Self::TransitionDuration => "transition-duration",
        Self::TransitionDelay => "transition-delay",
        Self::TransitionTiming => "transition-timing",
        Self::AnimationName => "animation-name",
    }
}
```

- [ ] **Step 4: Add typed custom value lane**

In `../surgeist-style/src/value.rs`, add open typed values without forcing them into the existing closed `Value` enum:

```rust
#[derive(Clone, Debug, PartialEq)]
pub enum CustomValue {
    Number(f32),
    Length(Length),
    Color(Color),
    Text(String),
    Opaque(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum TypedValue {
    BuiltIn {
        property: PropertyId,
        value: Value,
    },
    Custom {
        property: PropertyId,
        kind: ValueKind,
        value: CustomValue,
    },
}

impl TypedValue {
    #[must_use]
    pub fn built_in(property: Property, value: Value) -> Self {
        Self::BuiltIn {
            property: PropertyId::from(property),
            value,
        }
    }

    #[must_use]
    pub fn custom(property: PropertyId, kind: ValueKind, value: CustomValue) -> Self {
        Self::Custom {
            property,
            kind,
            value,
        }
    }

    #[must_use]
    pub fn property_id(&self) -> &PropertyId {
        match self {
            Self::BuiltIn { property, .. } => property,
            Self::Custom { property, .. } => property,
        }
    }

    #[must_use]
    pub fn kind(&self) -> ValueKind {
        match self {
            Self::BuiltIn { value, .. } => value.kind(),
            Self::Custom { kind, .. } => *kind,
        }
    }

    #[must_use]
    pub fn built_in_value(&self) -> Option<&Value> {
        match self {
            Self::BuiltIn { value, .. } => Some(value),
            Self::Custom { .. } => None,
        }
    }

    #[must_use]
    pub fn custom_value(&self) -> Option<&CustomValue> {
        match self {
            Self::BuiltIn { .. } => None,
            Self::Custom { value, .. } => Some(value),
        }
    }
}

impl Value {
    #[must_use]
    pub const fn kind(&self) -> ValueKind {
        match self {
            Self::Keyword(_) => ValueKind::Keyword,
            Self::Display(_) => ValueKind::Display,
            Self::Number(_) => ValueKind::Number,
            Self::Length(_) | Self::Size(_) => ValueKind::Length,
            Self::Edges(_) | Self::Corners(_) => ValueKind::Edges,
            Self::Color(_) => ValueKind::Color,
            Self::GridTrackList(_)
            | Self::GridTemplateAreas(_)
            | Self::GridTemplate(_)
            | Self::GridDefinition(_)
            | Self::GridLine(_)
            | Self::GridPlacement(_)
            | Self::GridAreaPlacement(_)
            | Self::GridAutoFlow(_)
            | Self::GridFlowTolerance(_) => ValueKind::Grid,
            Self::Text(_) => ValueKind::Text,
            Self::Transform(_) => ValueKind::Transform,
            _ => ValueKind::Custom,
        }
    }
}
```

- [ ] **Step 5: Integrate typed declarations and fingerprints**

In `../surgeist-style/src/declaration.rs`, change declarations to carry `TypedValue`:

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct Declaration {
    pub property: PropertyId,
    pub value: TypedValue,
}

impl Declaration {
    #[must_use]
    pub fn built_in(property: Property, value: Value) -> Self {
        Self {
            property: PropertyId::from(property),
            value: TypedValue::built_in(property, value),
        }
    }

    pub fn try_built_in(property: Property, value: Value) -> Result<Self> {
        property.validate_value(&value)?;
        Ok(Self::built_in(property, value))
    }

    pub fn custom(property: PropertyId, kind: ValueKind, value: CustomValue) -> Result<Self> {
        match property {
            PropertyId::Custom(_) => Ok(Self {
                property: property.clone(),
                value: TypedValue::custom(property, kind, value),
            }),
            PropertyId::BuiltIn(property) => Err(Error::new(
                ErrorCode::InvalidProperty,
                format!("custom declaration cannot use built-in property `{property:?}`"),
            )),
        }
    }
}
```

Keep existing authoring APIs intact by adapting `Declarations::insert`, `try_insert`, and `get`:

```rust
pub fn insert(&mut self, property: Property, value: Value) -> &mut Self {
    for declaration in canonical_declarations(property, value) {
        self.insert_typed(Declaration::built_in(declaration.property, declaration.value));
    }
    self
}

pub fn custom(
    mut self,
    property: PropertyId,
    kind: ValueKind,
    value: CustomValue,
) -> Result<Self> {
    self.try_insert_custom(property, kind, value)?;
    Ok(self)
}

pub fn try_insert_custom(
    &mut self,
    property: PropertyId,
    kind: ValueKind,
    value: CustomValue,
) -> Result<&mut Self> {
    self.insert_typed(Declaration::custom(property, kind, value)?);
    Ok(self)
}

fn insert_typed(&mut self, declaration: Declaration) {
    if let Some(existing) = self
        .values
        .iter_mut()
        .find(|existing| existing.property == declaration.property)
    {
        *existing = declaration;
    } else {
        self.values.push(declaration);
    }
}

#[must_use]
pub fn get(&self, property: Property) -> Option<&Value> {
    self.values
        .iter()
        .find(|declaration| declaration.property == PropertyId::from(property))
        .and_then(|declaration| declaration.value.built_in_value())
}

#[must_use]
pub fn get_typed(&self, property: &PropertyId) -> Option<&TypedValue> {
    self.values
        .iter()
        .find(|declaration| &declaration.property == property)
        .map(|declaration| &declaration.value)
}
```

Change `Declarations::fingerprint` so it hashes both built-in and custom declarations:

```rust
pub fn fingerprint(&self) -> Fingerprint {
    let mut hasher = DefaultHasher::new();
    for declaration in &self.values {
        declaration.property.hash(&mut hasher);
        hash_typed_value(&declaration.value, &mut hasher);
    }
    Fingerprint(hasher.finish())
}

fn hash_typed_value(value: &TypedValue, state: &mut DefaultHasher) {
    match value {
        TypedValue::BuiltIn { value, .. } => hash_value(value, state),
        TypedValue::Custom { kind, value, .. } => {
            kind.hash(state);
            hash_custom_value(value, state);
        }
    }
}
```

Add `Hash` to `ValueKind` derives and implement `hash_custom_value` by hashing the variant tag plus the same primitive hash helpers used by `hash_value`.

- [ ] **Step 6: Integrate resolver snapshots**

In `../surgeist-style/src/resolver.rs`, store custom typed values beside built-in resolved values:

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct Resolved {
    values: BTreeMap<Property, Value>,
    custom_values: BTreeMap<PropertyId, TypedValue>,
}

impl Resolved {
    #[must_use]
    pub fn new() -> Self {
        let mut values = BTreeMap::new();
        let mut custom_values = BTreeMap::new();
        for property in Property::ALL {
            if property.is_canonical() {
                let default = property.metadata().default;
                values.insert(*property, default.clone());
                custom_values.insert(
                    PropertyId::BuiltIn(*property),
                    TypedValue::built_in(*property, default),
                );
            }
        }
        Self {
            values,
            custom_values,
        }
    }

    #[must_use]
    pub fn get_typed(&self, property: &PropertyId) -> Option<&TypedValue> {
        match property {
            PropertyId::BuiltIn(property) => self.values.get(property).map(|_| {
                self.custom_values
                    .get(&PropertyId::BuiltIn(*property))
                    .unwrap_or_else(|| panic!("built-in typed value missing for `{property:?}`"))
            }),
            PropertyId::Custom(_) => self.custom_values.get(property),
        }
    }
}
```

When resolver application walks declarations, route by property ID:

```rust
fn apply_declaration(resolved: &mut Resolved, declaration: &Declaration) -> Result<()> {
    match (&declaration.property, &declaration.value) {
        (PropertyId::BuiltIn(property), TypedValue::BuiltIn { value, .. }) => {
            resolved.values.insert(*property, value.clone());
            resolved.custom_values.insert(
                PropertyId::BuiltIn(*property),
                declaration.value.clone(),
            );
        }
        (PropertyId::Custom(_), TypedValue::Custom { .. }) => {
            resolved
                .custom_values
                .insert(declaration.property.clone(), declaration.value.clone());
        }
        _ => {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "declaration property and typed value kind do not match",
            ));
        }
    }
    Ok(())
}
```

The resolver stores built-in typed values in `custom_values` under `PropertyId::BuiltIn(property)` and custom typed values under their custom IDs. The invariant is that every declaration can be read from the resolved style snapshot by `PropertyId`, while existing built-in APIs such as `Resolved::get(Property)` remain unchanged.

- [ ] **Step 7: Export new boundary types**

In `../surgeist-style/src/lib.rs`, update exports:

```rust
pub use property::{
    Impact, Interpolation, Metadata, Property, PropertyDescriptor, PropertyId, ValueKind,
};
pub use value::{
    AlignContent, AlignItems, BoxSizing, Clear, Color, Corners, Cursor, CustomValue, Dash,
    Direction, Display, Edges, FlexDirection, FlexWrap, Float, GridAreaPlacement, GridAutoFlow,
    GridDefinition, GridFlowTolerance, GridLine, GridPlacement, GridTemplate, GridTemplateAreaRow,
    GridTemplateAreas, GridTrackComponent, GridTrackList, Keyword, LayoutPosition, Length,
    LineStyle, MaxTrackSizing, MinTrackSizing, Overflow, OverflowAxes, PointerEvents, Shadow,
    SideSet, Size, Stroke, StrokeAlign, StyleTextAlign, SubgridLineNameComponent,
    SubgridLineNameRepeatCount, SubgridTrack, TextValue, TrackRepeat, TrackRepeatCount,
    TrackSizing, Transform, TransformOp, TypedValue, Value, Visibility, WritingMode,
};
```

- [ ] **Step 8: Run tests**

Run:

```sh
cargo test -p surgeist-style
cargo test -p surgeist-style
```

Expected: selected tests and style module tests pass.

- [ ] **Step 9: Commit**

```sh
echo "commit upstream files in the owning crate repo"
git commit -m "style: introduce open property descriptors"
```

### Task 7: Cross-Crate Handoff - Split Rich Style Display From Layout Dispatch

The style descriptor and CSS parser work belongs to `surgeist-style` and `surgeist-css`. The only crate-local layout work in this task is preserving the existing closed `src/node_input.rs::Display` dispatch contract and adding layout tests if a new adapter output requires it. Do not edit sibling crates from this project.

**Files:**
- Upstream handoff: `../surgeist-style/src/value.rs`
- Upstream handoff: `../surgeist-style/src/adapters/layout.rs`
- Modify: `src/node_input.rs`
- Modify: `src/tests.rs`
- Upstream handoff: `../surgeist-css/src/lib.rs`

- [ ] **Step 1: Write failing display descriptor tests**

Add to `../surgeist-style/src/value.rs` tests:

```rust
#[test]
fn style_display_descriptor_separates_outer_inner_and_box_generation() {
    let descriptor = DisplayDescriptor::inline_grid();

    assert_eq!(descriptor.outer(), DisplayOuter::Inline);
    assert_eq!(descriptor.inner(), DisplayInner::Grid);
    assert_eq!(descriptor.box_generation(), BoxGeneration::Normal);
    assert!(descriptor.is_inline_level());
}

#[test]
fn display_none_has_no_box_generation_without_layout_dispatch_leak() {
    let descriptor = DisplayDescriptor::none();

    assert_eq!(descriptor.box_generation(), BoxGeneration::None);
    assert_eq!(descriptor.inner(), DisplayInner::Flow);
    assert!(!descriptor.is_inline_level());
}
```

Add to `../surgeist-style/src/adapters/layout.rs` tests:

```rust
#[test]
fn lower_display_descriptor_to_closed_layout_dispatch() {
    assert_eq!(
        lower_display(DisplayDescriptor::inline_grid()).unwrap(),
        layout::Display::InlineGrid
    );
    assert_eq!(
        lower_display(DisplayDescriptor::none()).unwrap(),
        layout::Display::None
    );
}
```

- [ ] **Step 2: Run tests to verify failure**

Run:

```sh
cargo test -p surgeist-style
```

Expected: tests fail to compile because descriptor types do not exist and adapter lowering still accepts `style::Display`.

- [ ] **Step 3: Add style display descriptor types**

In `../surgeist-style/src/value.rs`, add:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DisplayOuter {
    Block,
    Inline,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DisplayInner {
    Flow,
    FlowRoot,
    Flex,
    Grid,
    GridLanes,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BoxGeneration {
    Normal,
    None,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DisplayDescriptor {
    outer: DisplayOuter,
    inner: DisplayInner,
    box_generation: BoxGeneration,
}

impl DisplayDescriptor {
    #[must_use]
    pub const fn new(outer: DisplayOuter, inner: DisplayInner) -> Self {
        Self {
            outer,
            inner,
            box_generation: BoxGeneration::Normal,
        }
    }

    #[must_use]
    pub const fn inline_grid() -> Self {
        Self::new(DisplayOuter::Inline, DisplayInner::Grid)
    }

    #[must_use]
    pub const fn none() -> Self {
        Self {
            outer: DisplayOuter::Block,
            inner: DisplayInner::Flow,
            box_generation: BoxGeneration::None,
        }
    }

    #[must_use]
    pub const fn outer(self) -> DisplayOuter {
        self.outer
    }

    #[must_use]
    pub const fn inner(self) -> DisplayInner {
        self.inner
    }

    #[must_use]
    pub const fn box_generation(self) -> BoxGeneration {
        self.box_generation
    }

    #[must_use]
    pub const fn is_inline_level(self) -> bool {
        matches!(self.outer, DisplayOuter::Inline)
            && matches!(self.box_generation, BoxGeneration::Normal)
    }
}
```

Keep existing `style::Display` temporarily as a compatibility facade:

```rust
impl From<Display> for DisplayDescriptor {
    fn from(display: Display) -> Self {
        match display {
            Display::Block => Self::new(DisplayOuter::Block, DisplayInner::Flow),
            Display::Flex => Self::new(DisplayOuter::Block, DisplayInner::Flex),
            Display::Grid => Self::new(DisplayOuter::Block, DisplayInner::Grid),
            Display::InlineBlock => Self::new(DisplayOuter::Inline, DisplayInner::FlowRoot),
            Display::InlineGrid => Self::inline_grid(),
            Display::GridLanes => Self::new(DisplayOuter::Block, DisplayInner::GridLanes),
            Display::InlineGridLanes => Self::new(DisplayOuter::Inline, DisplayInner::GridLanes),
            Display::None => Self::none(),
        }
    }
}
```

- [ ] **Step 4: Update adapter lowering**

Change `lower_display` in `../surgeist-style/src/adapters/layout.rs` to accept `DisplayDescriptor`:

```rust
fn lower_display(display: DisplayDescriptor) -> Result<layout::Display> {
    if display.box_generation() == BoxGeneration::None {
        return Ok(layout::Display::None);
    }

    match (display.outer(), display.inner()) {
        (DisplayOuter::Block, DisplayInner::Flow) => Ok(layout::Display::Block),
        (DisplayOuter::Block, DisplayInner::Flex) => Ok(layout::Display::Flex),
        (DisplayOuter::Block, DisplayInner::Grid) => Ok(layout::Display::Grid),
        (DisplayOuter::Block, DisplayInner::GridLanes) => Ok(layout::Display::GridLanes),
        (DisplayOuter::Inline, DisplayInner::FlowRoot) => Ok(layout::Display::InlineBlock),
        (DisplayOuter::Inline, DisplayInner::Grid) => Ok(layout::Display::InlineGrid),
        (DisplayOuter::Inline, DisplayInner::GridLanes) => Ok(layout::Display::InlineGridLanes),
        (_, DisplayInner::FlowRoot | DisplayInner::Flow) => Ok(layout::Display::Block),
        (_, DisplayInner::Flex) => Err(unsupported("inline flex display")),
    }
}
```

In `lower(resolved)`, use:

```rust
display: lower_display(DisplayDescriptor::from(resolved.display()))?,
```

- [ ] **Step 5: Run tests**

Run:

```sh
cargo test -p surgeist-style
cargo test -p surgeist-layout tests::inline_display_values_preserve_outer_participation_and_inner_context
```

Expected: tests pass and existing layout dispatch remains closed.

- [ ] **Step 6: Commit**

```sh
echo "commit upstream files in the owning crate repo"
git commit -m "style: split display descriptor from layout dispatch"
```

### Task 8: Upstream Handoff - Narrow Length Wrappers and Grid Name Newtypes

This task belongs to `surgeist-style` after the split. Keep this section as source material for an upstream issue or style-crate plan; do not execute it from the `surgeist-layout` project.

**Files:**
- Upstream handoff: `../surgeist-style/src/value.rs`

- [ ] **Step 1: Write failing wrapper tests**

Add to `../surgeist-style/src/value.rs` tests:

```rust
#[test]
fn style_context_lengths_reject_wrong_keywords_before_adapter_lowering() {
    assert!(DimensionLength::try_from(Length::Auto).is_ok());
    assert!(DimensionLength::try_from(Length::Normal).is_err());
    assert!(EdgeLength::try_from(Length::Auto).is_ok());
    assert!(EdgeLength::try_from(Length::MinContent).is_err());
    assert!(GapLength::try_from(Length::Normal).is_ok());
    assert!(GapLength::try_from(Length::Auto).is_err());
}

#[test]
fn grid_line_name_newtype_rejects_empty_names() {
    assert!(GridLineName::new("content-start").is_ok());
    assert!(GridLineName::new("").is_err());
    assert!(GridAreaName::new("hero").is_ok());
    assert!(GridAreaName::new(" ").is_err());
}
```

- [ ] **Step 2: Run tests to verify failure**

Run:

```sh
cargo test -p surgeist-style
```

Expected: tests fail to compile because wrapper and name types do not exist.

- [ ] **Step 3: Add narrow wrappers**

In `../surgeist-style/src/value.rs`, add:

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct DimensionLength(Length);

#[derive(Clone, Debug, PartialEq)]
pub struct EdgeLength(Length);

#[derive(Clone, Debug, PartialEq)]
pub struct GapLength(Length);

impl TryFrom<Length> for DimensionLength {
    type Error = Error;

    fn try_from(value: Length) -> Result<Self> {
        match value {
            Length::Normal => Err(Error::new(ErrorCode::InvalidValue, "dimension does not accept normal")),
            Length::Px(_) | Length::Percent(_) | Length::Calc(_) | Length::Fill | Length::Fit | Length::MinContent | Length::MaxContent | Length::Auto => Ok(Self(value)),
        }
    }
}

impl TryFrom<Length> for EdgeLength {
    type Error = Error;

    fn try_from(value: Length) -> Result<Self> {
        match value {
            Length::Px(_) | Length::Percent(_) | Length::Calc(_) | Length::Auto => Ok(Self(value)),
            Length::Normal | Length::Fill | Length::Fit | Length::MinContent | Length::MaxContent => Err(Error::new(
                ErrorCode::InvalidValue,
                "edge length accepts px, percent, calc, or auto",
            )),
        }
    }
}

impl TryFrom<Length> for GapLength {
    type Error = Error;

    fn try_from(value: Length) -> Result<Self> {
        match value {
            Length::Normal | Length::Px(_) | Length::Percent(_) | Length::Calc(_) => Ok(Self(value)),
            Length::Auto | Length::Fill | Length::Fit | Length::MinContent | Length::MaxContent => Err(Error::new(
                ErrorCode::InvalidValue,
                "gap length accepts normal, px, percent, or calc",
            )),
        }
    }
}
```

This task only introduces the wrapper types and their tests. Parser, adapter, declaration storage, and grid field migrations are intentionally outside this task because those changes touch separate behavior surfaces and should be planned after the wrappers have a stable shape.

- [ ] **Step 4: Add grid line and area name newtypes**

In `../surgeist-style/src/value.rs`, add:

```rust
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct GridLineName(String);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct GridAreaName(String);

impl GridLineName {
    pub fn new(name: impl Into<String>) -> Result<Self> {
        let name = name.into();
        validate_grid_name(&name, "grid line name")?;
        Ok(Self(name))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl GridAreaName {
    pub fn new(name: impl Into<String>) -> Result<Self> {
        let name = name.into();
        validate_grid_name(&name, "grid area name")?;
        Ok(Self(name))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn validate_grid_name(name: &str, label: &str) -> Result<()> {
    if name.trim().is_empty() {
        return Err(Error::new(
            ErrorCode::InvalidValue,
            format!("{label} must not be empty"),
        ));
    }
    Ok(())
}
```

Stage migration by adding conversions at boundaries:

```rust
impl From<GridLineName> for String {
    fn from(value: GridLineName) -> Self {
        value.0
    }
}
```

Do not convert existing `Vec<String>` grid fields in this task. A later scoped migration should update `GridTrackComponent::LineNames`, `SubgridLineNameComponent`, `GridTemplateAreaRow`, CSS parsing, and layout lowering together.

- [ ] **Step 5: Run tests**

Run:

```sh
cargo test -p surgeist-style
```

Expected: selected wrapper tests pass and existing style/CSS tests pass.

- [ ] **Step 6: Commit**

```sh
echo "commit upstream files in the owning crate repo"
git commit -m "style: add narrowed value wrappers"
```

### Task 9: Retire Existing Layout Lint Suppressions In a Separate Pass

**Files:**
- Modify only if suppressions exist: `src/grid/mod.rs`
- Modify only if suppressions exist: `src/grid/tracks.rs`
- Modify only if suppressions exist: `src/grid/child.rs`
- Modify only if suppressions exist: `src/grid/lanes.rs`
- Modify only if suppressions exist: `src/grid/subgrid.rs`
- Modify only if suppressions exist: any other `src/**/*.rs` file reported by the scan command below

- [ ] **Step 1: Scan existing layout suppressions**

Run:

```sh
grep -RInE '#\[(allow|expect)\]|clippy::' src || true
```

Expected: command reports existing suppressions or no matches. Record the exact matched files in the task notes before editing. This task may remove existing suppressions but must not add new ones.

- [ ] **Step 2: Pick one suppression group and write a focused behavior test**

If the scan reports a suppression around grid track sizing, add a test to `src/grid/tests.rs` that proves the behavior protected by the surrounding code. Example for a branch-heavy track helper:

```rust
#[test]
fn percent_tracks_keep_intrinsic_distribution_stable_after_lint_cleanup() {
    let tracks = vec![
        TrackSizing::percent(0.25),
        TrackSizing::minmax(MinTrackSizing::MIN_CONTENT, MaxTrackSizing::AUTO),
    ];
    let mut sizes = vec![0.0, 0.0];

    super::tracks::distribute_min_content_span_with_percent(
        &mut sizes,
        &tracks,
        Overflow::Visible,
        Some(200.0),
        120.0,
    );

    assert!(sizes[1] >= 70.0);
}
```

If the scan reports no suppressions, add no test and proceed to Step 5.

- [ ] **Step 3: Run the focused test before cleanup**

Run the exact test added in Step 2, for example:

```sh
cargo test -p surgeist-layout grid::tests::percent_tracks_keep_intrinsic_distribution_stable_after_lint_cleanup
```

Expected: test passes before cleanup, proving the behavior is already present.

- [ ] **Step 4: Remove one suppression group by reshaping code**

Fix the warning cause directly. Acceptable fixes include extracting a named helper, reducing argument count with a small input struct, splitting a long function into two behavior-named functions, or making a match exhaustive with clearer local names.

Do not add replacement suppression attributes, lint expectation attributes, crate-level lint allow lists, or Clippy suppression paths.

- [ ] **Step 5: Run lint and layout tests**

Run:

```sh
cargo clippy -p surgeist-layout --all-targets -- -D warnings
cargo test -p surgeist-layout --lib
```

Expected: clippy exits successfully with no warnings and layout tests pass.

- [ ] **Step 6: Commit only if a suppression group was changed**

If Step 1 found no suppressions, make no commit in this task and proceed to final verification.

If the example grid track sizing group was changed, stage only the exact edited files:

```sh
git add -- src/grid/tracks.rs src/grid/tests.rs
git commit -m "layout: retire lint suppressions"
```

If a different suppression group was changed, replace the file list with the exact files recorded in Step 1 and edited in Step 4. Do not stage `src` as a directory.

### Task 10: Final Verification

**Files:**
- Read: all files changed by previous tasks
- Do not modify: sibling crates or top-level workspace files
- Modify only if public API changed: `api/public-api.txt`

- [ ] **Step 1: Confirm scoped diff**

Run:

```sh
git status --short --branch
git diff --stat
```

Expected: only files listed in this plan are modified. Sibling crate or top-level changes are outside this crate-local plan; do not stage or commit them from this repo.

- [ ] **Step 2: Run formatting**

Run:

```sh
cargo fmt --check
```

Expected: exits successfully. If formatting fails, run `cargo fmt`, inspect the diff, then rerun `cargo fmt --check`.

- [ ] **Step 3: Refresh source-derived public API artifact if needed**

If any public item, re-export, trait method, enum variant, or public type signature changed, run:

```sh
cargo run --manifest-path api/generator/Cargo.toml
```

Expected: `api/public-api.txt` is unchanged for internal-only work or contains the expected source-derived public API delta for intentional public API changes.

- [ ] **Step 4: Run focused package tests**

Run:

```sh
cargo test -p surgeist-layout --lib
cargo test -p surgeist-layout grid::
cargo test -p surgeist-layout flex::
```

Expected: all focused tests pass.

- [ ] **Step 5: Run full package tests**

Run:

```sh
cargo test -p surgeist-layout
```

Expected: package tests pass.

- [ ] **Step 6: Run lint verification**

Run:

```sh
cargo clippy -p surgeist-layout --all-targets -- -D warnings
```

Expected: exits successfully with no warnings.

- [ ] **Step 7: Confirm no new lint suppression text in changed files**

Run:

```sh
files=$(git diff --name-only -- src)
if [ -n "$files" ]; then
  if printf '%s\n' "$files" | xargs grep -nE '#\[(allow|expect)\]|clippy::'; then
    echo "unexpected lint suppression text found in changed source files"
    exit 1
  else
    status=$?
    if [ "$status" -eq 1 ]; then
      echo "no lint suppression text found in changed source files"
    else
      exit "$status"
    fi
  fi
else
  echo "no changed source files to scan"
fi
```

Expected: no matches in newly changed source files. Existing matches in files not touched by this plan are outside this final check.

- [ ] **Step 8: Commit final verification-only adjustments if any**

Only if Steps 2 through 6 required formatting or small test-name adjustments, commit them:

```sh
git add -- src/value.rs src/lib.rs src/traits.rs src/tests.rs src/compute.rs src/block.rs src/flex.rs src/grid/mod.rs src/grid/child.rs src/grid/tracks.rs src/grid/lanes.rs src/grid/subgrid.rs api/public-api.txt
git commit -m "layout: verify typed calc upgrade"
```

Expected: no commit is needed if previous tasks already left the tree formatted, tested, and lint-clean.

## Execution Notes

- Keep each task's commit scoped to the exact files in that task. Do not stage sibling crate or top-level files from this crate repo.
- If a calc behavior requires subtraction in layout expressions, represent subtraction as a negative px or percent term inside `layout::CalcExpression`; keep authored sign structure in `style::CalcLength`.
- If trait objects are required for `CalcResolver`, make the trait object-safe by avoiding generic methods on the trait itself.
- The open property/value work is intentionally after calc because it is the broadest API boundary. Do not begin the display split before calc lowering and percent-dependent behavior are passing.
