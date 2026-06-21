# Surgeist Named Grid Syntax Engine Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement production layout-engine support for spec-complete named grid syntax in Surgeist, using the completed named-grid oracle as the behavioral reference.

**Architecture:** Preserve named grid-line facts from resolved style into layout `NodeInput`, build production line-name maps from template track lists, template areas, and inherited subgrid names, then resolve every grid item to the existing numeric placement shape before occupancy, track sizing, child layout, absolute layout, and grid-lanes logic consume placements. The production resolver should mirror the oracle rules without importing test-only oracle modules.

**Tech Stack:** Rust under `crates/surgeist/src/layout` and `crates/surgeist/src/style/adapters/layout.rs`, production grid code under `crates/surgeist/src/layout/grid`, oracle comparison tests under `crates/surgeist/tests/layout_oracle.rs`, direct layout tests under `crates/surgeist/tests/layout/grid.rs`, browser parity fixtures under `crates/surgeist/tests/layout_browser_parity`, and verification with `cargo test -p surgeist`, parity filters, and `cargo clippy -p surgeist --all-targets --all-features -- -D warnings`.

---

## Source References

- Oracle plan: `docs/superpowers/plans/2026-06-17-surgeist-named-grid-syntax-oracle-implementation.md`
- Oracle implementation: `crates/surgeist/tests/support/oracle/grid/named.rs`
- Production style named syntax:
  - `crates/surgeist/src/style/value.rs`
  - `crates/surgeist/src/style/adapters/layout.rs`
  - `crates/surgeist/tests/layout_browser_parity/support.rs`
- Production layout input:
  - `crates/surgeist/src/layout/node_input.rs`
  - `crates/surgeist/src/layout/value.rs`
- Production grid engine:
  - `crates/surgeist/src/layout/grid/mod.rs`
  - `crates/surgeist/src/layout/grid/placement.rs`
  - `crates/surgeist/src/layout/grid/tracks.rs`
  - `crates/surgeist/src/layout/grid/child.rs`
  - `crates/surgeist/src/layout/grid/subgrid.rs`
  - `crates/surgeist/src/layout/grid/lanes.rs`
- Test surfaces:
  - `crates/surgeist/tests/oracle.rs`
  - `crates/surgeist/tests/layout_oracle.rs`
  - `crates/surgeist/tests/layout/grid.rs`
  - `crates/surgeist/tests/layout_browser_parity.rs`

---

## File Map

- Modify `crates/surgeist/src/layout/node_input.rs`
  - Keep the existing numeric `GridPlacement` shape as the resolved placement consumed by current grid code.
  - Add layout-layer `RawGridLine` and `RawGridPlacement` for parser/style facts that may still contain named lines.
  - Add `raw_grid_column` and `raw_grid_row` to `NodeInput` alongside existing numeric `grid_column` and `grid_row`.

- Modify `crates/surgeist/src/style/value.rs`
  - Add `style::GridLine::BareIdent(String)` so a bare `foo` can resolve side-aware through `foo-start`/`foo-end` before falling back to `foo 1`.
  - Keep `style::GridLine::NamedLine { name, index }` for explicit `foo 1`, `foo 2`, and `2 foo` syntax.
  - Preserve subgrid line-name repeat syntax instead of flattening `repeat(auto-fill, [name])` before the used subgrid span is known.

- Modify `crates/surgeist/src/style/declaration.rs`
  - Keep `grid-row`, `grid-column`, and `grid-area` shorthand expansion spec-equivalent for bare idents and omitted sides.

- Modify `crates/surgeist/src/layout/mod.rs`
  - Re-export `RawGridLine`, `RawGridPlacement`, and existing numeric `GridPlacement`.

- Modify `crates/surgeist/src/style/adapters/layout.rs`
  - Lower `style::GridLine::NamedLine` and `style::GridLine::NamedSpan` into layout `RawGridLine` instead of returning unsupported.
  - Lower `grid-template-areas` into `NodeInput`.

- Modify `crates/surgeist/src/layout/node_input.rs` and `crates/surgeist/src/style/adapters/layout.rs`
  - Add `grid_template_areas: GridTemplateAreas` to `NodeInput`.
  - Add layout-layer `GridTemplateAreas` and `GridTemplateAreaRow` in `crates/surgeist/src/layout/value.rs`.
  - Preserve `GridTrackComponent::LineNames` into layout track components instead of dropping them.
  - Preserve line-name components inside fixed `repeat(...)` track lists.
  - Preserve subgrid line-name repeat components until production subgrid inheritance knows the used span.

- Create `crates/surgeist/src/layout/grid/named.rs`
  - Own production named line maps, fixed-repeat name expansion, area-generated line names, named placement resolution, subgrid local-name expansion, subgrid name inheritance, and conversion to `GridPlacement`.

- Modify `crates/surgeist/src/layout/grid/mod.rs`
  - Split current grid initialization into a placement-independent explicit-track phase and a placement-dependent completion phase.
  - Build container named-line contexts after explicit tracks exist but before leading implicit tracks, track requirements, visible cell count, occupancy, sizing traversal, child layout, and absolute layout consume placements.
  - Resolve child raw placements in child order before the placement-dependent track completion phase.
  - Carry inherited named-line facts through `GridParentContext`.

- Modify `crates/surgeist/src/layout/grid/placement.rs`
  - Change placement helpers to consume `GridPlacement`.
  - Keep numeric placement algorithms behavior-equivalent.

- Modify `crates/surgeist/src/layout/grid/tracks.rs`
  - Preserve explicit line names from track-list components and fixed repeats.
  - Reject or explicitly classify named line lookup through unresolved `auto-fill`/`auto-fit` until the engine has a resolved repeat count; do not guess layout-dependent repetition counts.

- Modify `crates/surgeist/src/layout/grid/subgrid.rs`
  - Merge inherited parent line names, clipped area-generated names, and local subgrid line names.
  - Carry reversed-axis mapping consistently with existing subgrid track inheritance.

- Modify `crates/surgeist/src/layout/grid/child.rs`
  - Consume resolved placements from the container placement context rather than reading raw `NodeInput.grid_column`/`grid_row` directly.

- Modify `crates/surgeist/src/layout/grid/lanes.rs`
  - Consume resolved placements for any lane path that currently reads `GridPlacement` directly.

- Modify tests:
  - `crates/surgeist/tests/style.rs`
  - `crates/surgeist/tests/layout/grid.rs`
  - `crates/surgeist/tests/layout_oracle.rs`
  - `crates/surgeist/tests/layout_browser_parity.rs`
  - `crates/surgeist/tests/layout_browser_parity/support.rs`

---

## Required Boundaries

- [ ] Do not import `crates/surgeist/tests/support/oracle/*` from production code.
- [ ] Keep the oracle as the reference model and production `named.rs` as a separate implementation.
- [ ] Preserve ordinary numeric grid behavior and current subgrid/grid-lanes behavior.
- [ ] Preserve raw named syntax through the style-to-layout adapter; do not resolve names in style code.
- [ ] Resolve names inside the grid container after explicit tracks, areas, gaps, direction, and parent subgrid context are known, but before placement-dependent track initialization.
- [ ] Keep placement side awareness: start-side bare idents prefer `<name>-start`, end-side bare idents prefer `<name>-end`.
- [ ] Preserve the distinction between bare `foo` and explicit `foo 1` from parser through engine resolution.
- [ ] Match production-relevant oracle errors: reserved `auto`/`span` custom idents, zero line, zero span, auto without cursor, line before first, area not found, empty specified template areas, non-rectangular template area, zero repeat, and multiple subgrid auto-fill repeats.
- [ ] Commit after each logical phase with a short concrete message.
- [ ] Run `git status --short --branch` before staging each commit.
- [ ] Use narrow `git add` path lists.
- [ ] Run `git diff --check` before each commit.

---

## Implementation Overview

The production style layer already has `style::GridLine::NamedLine` and `style::GridLine::NamedSpan`, and parity helper parsing can read named grid-line syntax. The current blocker is `style/adapters/layout.rs`, where named placement returns `unsupported("named grid line placement")`, and `layout::NodeInput` only stores numeric `GridPlacement`. This plan first adds raw placement fields alongside the numeric fields, then moves the grid engine to resolve raw placement into numeric placement once explicit named line maps exist.

The implementation should happen in this order:

1. Preserve raw named placement and template-area facts into layout input without breaking existing numeric `GridPlacement` call sites.
2. Preserve and test shorthand expansion for `grid-row`, `grid-column`, and `grid-area`.
3. Add production named-line maps for ordinary track lists and template areas.
4. Resolve named ordinary-grid child placement into numeric placement before the existing placement pipeline.
5. Thread resolved placement context through grid placement, sizing, child layout, and absolute layout.
6. Add subgrid line-name inheritance and clamped subgrid named placement.
7. Wire named placement through grid-lanes paths.
8. Add broad oracle comparison tests and flip the known named-grid parity fixtures from expected engine gap to expected pass.

---

## Task 1: Preserve Named Syntax In Layout Input

**Files:**
- Modify `crates/surgeist/src/layout/node_input.rs`
- Modify `crates/surgeist/src/layout/mod.rs`
- Modify `crates/surgeist/src/layout/value.rs`
- Modify `crates/surgeist/src/style/value.rs`
- Modify `crates/surgeist/src/style/declaration.rs`
- Modify `crates/surgeist/src/style/adapters/layout.rs`
- Modify `crates/surgeist/tests/style.rs`
- Modify `crates/surgeist/tests/layout_browser_parity/support.rs`

- [ ] **Step 1: Add failing adapter tests for named placements**

Add tests near the existing grid-style lowering tests:

```rust
#[test]
fn layout_adapter_preserves_named_grid_line_placement() {
    let resolved = resolved_style(
        s::Declarations::new()
            .display(s::Display::Grid)
            .grid_column(s::GridPlacement::new(
                s::GridLine::NamedLine {
                    name: "content".to_string(),
                    index: 2,
                },
                s::GridLine::NamedSpan {
                    name: "content".to_string(),
                    index: 3,
                },
            )),
    );

    let input = surgeist::style::adapters::layout::lower(&resolved).unwrap();
    assert_eq!(
        input.raw_grid_column,
        layout::RawGridPlacement::new(
            layout::RawGridLine::NamedLine {
                name: "content".to_string(),
                index: 2,
            },
            layout::RawGridLine::NamedSpan {
                name: "content".to_string(),
                index: 3,
            },
        )
    );
}
```

Expected before implementation: fails with `named grid line placement cannot be lowered to layout style yet`.

Also add a test that proves bare idents remain distinct from explicit named occurrences:

```rust
#[test]
fn layout_adapter_preserves_bare_grid_line_ident() {
    let resolved = resolved_style(
        s::Declarations::new().grid_column(s::GridPlacement::new(
            s::GridLine::BareIdent("main".to_string()),
            s::GridLine::NamedLine {
                name: "main".to_string(),
                index: 1,
            },
        )),
    );

    let input = surgeist::style::adapters::layout::lower(&resolved).unwrap();
    assert_eq!(
        input.raw_grid_column,
        layout::RawGridPlacement::new(
            layout::RawGridLine::BareIdent("main".to_string()),
            layout::RawGridLine::NamedLine {
                name: "main".to_string(),
                index: 1,
            },
        )
    );
}
```

- [ ] **Step 1a: Preserve bare ident syntax in style values and fixture parsing**

In `crates/surgeist/src/style/value.rs`, extend `GridLine`:

```rust
pub enum GridLine {
    Auto,
    Line(i16),
    Span(u16),
    BareIdent(String),
    NamedLine { name: String, index: i16 },
    NamedSpan { name: String, index: u16 },
}
```

Validation for `BareIdent` must call the same reserved-name check as `NamedLine`/`NamedSpan`.

In `crates/surgeist/tests/layout_browser_parity/support.rs`, change the single-token non-number parser branch:

```rust
[token] => match parse_style_line_index(token) {
    Ok(line) => Ok(s::GridLine::Line(line)),
    Err(_) => Ok(s::GridLine::BareIdent(parse_custom_ident(token)?.to_owned())),
},
```

Keep two-token `name index` and `index name` parsing as `NamedLine`.

Before committing Task 1, run:

```bash
rg -n "GridLine::|hash_grid_line|lower_grid_placement|parse_style_grid_line" crates/surgeist/src crates/surgeist/tests
```

Update every exhaustive `GridLine` match for `BareIdent`, especially validation, hashing in `crates/surgeist/src/style/declaration.rs`, numeric fallback lowering in `style/adapters/layout.rs`, parser helper tests, and any declaration hashing tests.

- [ ] **Step 1b: Add shorthand tests before changing lowering**

Add tests near existing style shorthand tests:

```rust
#[test]
fn grid_column_shorthand_repeats_bare_ident_on_omitted_end_side() {
    let declarations = s::Declarations::new().grid_column(s::GridPlacement::new(
        s::GridLine::BareIdent("main".to_string()),
        s::GridLine::Auto,
    ));

    assert_eq!(
        declarations.get(s::Property::GridColumn),
        Some(&s::Value::GridPlacement(s::GridPlacement::new(
            s::GridLine::BareIdent("main".to_string()),
            s::GridLine::BareIdent("main".to_string()),
        )))
    );
}

#[test]
fn grid_column_shorthand_omits_non_ident_end_to_auto() {
    let declarations = s::Declarations::new().grid_column(s::GridPlacement::new(
        s::GridLine::Line(2),
        s::GridLine::Auto,
    ));

    assert_eq!(
        declarations.get(s::Property::GridColumn),
        Some(&s::Value::GridPlacement(s::GridPlacement::new(
            s::GridLine::Line(2),
            s::GridLine::Auto,
        )))
    );
}

#[test]
fn grid_area_one_bare_ident_expands_to_all_four_sides() {
    let declarations = s::Declarations::new().grid_area(s::GridAreaPlacement::new(
        s::GridLine::BareIdent("main".to_string()),
        s::GridLine::Auto,
        s::GridLine::Auto,
        s::GridLine::Auto,
    ));

    assert_eq!(
        declarations.get(s::Property::GridRow),
        Some(&s::Value::GridPlacement(s::GridPlacement::new(
            s::GridLine::BareIdent("main".to_string()),
            s::GridLine::BareIdent("main".to_string()),
        )))
    );
    assert_eq!(
        declarations.get(s::Property::GridColumn),
        Some(&s::Value::GridPlacement(s::GridPlacement::new(
            s::GridLine::BareIdent("main".to_string()),
            s::GridLine::BareIdent("main".to_string()),
        )))
    );
}
```

If shorthand expansion is not represented by `Declarations::grid_area` today, add equivalent CSS/declaration tests at the existing shorthand parser boundary. Invalid `grid-area` arity must remain a parser/declaration error before layout lowering.

- [ ] **Step 2: Add layout raw placement fields without changing numeric placement**

In `crates/surgeist/src/layout/node_input.rs`, keep the existing numeric `GridPlacement` exactly as the current `{ start: Option<isize>, end: Option<isize>, span: Option<usize> }` type. Add this raw placement shape next to it:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RawGridLine {
    Auto,
    Line(isize),
    Span(usize),
    BareIdent(String),
    NamedLine { name: String, index: isize },
    NamedSpan { name: String, index: usize },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RawGridPlacement {
    pub start: RawGridLine,
    pub end: RawGridLine,
}
```

Add raw constructors without moving the existing numeric constructors:

```rust
impl RawGridPlacement {
    pub const AUTO: Self = Self {
        start: RawGridLine::Auto,
        end: RawGridLine::Auto,
    };

    pub const fn line(line: isize) -> Self {
        Self::new(RawGridLine::Line(line), RawGridLine::Auto)
    }

    pub const fn lines(start: isize, end: isize) -> Self {
        Self::new(RawGridLine::Line(start), RawGridLine::Line(end))
    }

    pub const fn span(span: usize) -> Self {
        Self::new(RawGridLine::Auto, RawGridLine::Span(span))
    }
}
```

Add fields to `NodeInput`:

```rust
pub raw_grid_column: RawGridPlacement,
pub raw_grid_row: RawGridPlacement,
```

Default both raw fields to `RawGridPlacement::AUTO`. Existing numeric `grid_column` and `grid_row` stay in place and continue to support the current engine until Task 3 starts resolving raw placement.

- [ ] **Step 3: Re-export new layout placement types**

In `crates/surgeist/src/layout/mod.rs`, export:

```rust
pub use node_input::{GridPlacement, RawGridLine, RawGridPlacement};
```

Remove duplicate old exports if present.

- [ ] **Step 4: Lower style named grid lines into layout raw lines**

Add side-preserving raw lowering in `crates/surgeist/src/style/adapters/layout.rs`:

```rust
fn lower_raw_grid_line(line: GridLine) -> Result<layout::RawGridLine> {
    Ok(match line {
        GridLine::Auto => layout::RawGridLine::Auto,
        GridLine::Line(line) => layout::RawGridLine::Line(isize::from(line)),
        GridLine::Span(span) => layout::RawGridLine::Span(usize::from(span)),
        GridLine::BareIdent(name) => layout::RawGridLine::BareIdent(name),
        GridLine::NamedLine { name, index } => layout::RawGridLine::NamedLine {
            name,
            index: isize::from(index),
        },
        GridLine::NamedSpan { name, index } => layout::RawGridLine::NamedSpan {
            name,
            index: usize::from(index),
        },
    })
}

fn lower_raw_grid_placement(start: GridLine, end: GridLine) -> Result<layout::RawGridPlacement> {
    Ok(layout::RawGridPlacement::new(
        lower_raw_grid_line(start)?,
        lower_raw_grid_line(end)?,
    ))
}
```

Keep the existing numeric `lower_grid_placement` for now, but make it return `GridPlacement::AUTO` when either side is named. Task 3 replaces that temporary fallback by resolving `raw_grid_column` and `raw_grid_row` inside the grid container.

- [ ] **Step 5: Add layout template-area values and lowering**

Add this layout-layer value shape to `crates/surgeist/src/layout/value.rs`:

```rust
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GridTemplateAreas {
    pub rows: Vec<GridTemplateAreaRow>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GridTemplateAreaRow {
    pub cells: Vec<Option<String>>,
}
```

Add `grid_template_areas: GridTemplateAreas` to `NodeInput`, default it to empty, and lower `style::GridTemplateAreas` by cloning row cells.

- [ ] **Step 6: Preserve explicit track line names in layout track values**

In `crates/surgeist/src/layout/value.rs`, extend track values so line names survive lowering:

```rust
pub struct TrackRepetition {
    pub repeat: TrackRepeat,
    pub components: Vec<TrackComponent>,
}

pub enum TrackComponent {
    LineNames(Vec<String>),
    Track(TrackSizing),
    Repeat(TrackRepetition),
    Subgrid(SubgridTrack),
}

pub struct SubgridTrack {
    pub name_components: Vec<SubgridLineNameComponent>,
}

pub enum SubgridLineNameComponent {
    LineNames(Vec<String>),
    Repeat {
        count: SubgridLineNameRepeatCount,
        line_name_sets: Vec<Vec<String>>,
    },
}

pub enum SubgridLineNameRepeatCount {
    Count(usize),
    AutoFill,
}
```

Update constructors so existing `TrackRepetition::count(count, tracks: Vec<TrackSizing>)` still works by wrapping every track as `TrackComponent::Track`. Add a second constructor for already-tokenized components:

```rust
impl TrackRepetition {
    pub fn count_components(count: usize, components: Vec<TrackComponent>) -> Self {
        Self {
            repeat: TrackRepeat::Count(count),
            components,
        }
    }

    pub fn auto_fill_components(components: Vec<TrackComponent>) -> Self {
        Self {
            repeat: TrackRepeat::AutoFill,
            components,
        }
    }

    pub fn auto_fit_components(components: Vec<TrackComponent>) -> Self {
        Self {
            repeat: TrackRepeat::AutoFit,
            components,
        }
    }
}
```

In `style/adapters/layout.rs`, stop dropping `GridTrackComponent::LineNames`:

```rust
GridTrackComponent::LineNames(names) => {
    lowered.push(layout::TrackComponent::LineNames(names.clone()));
}
```

In `lower_track_repeat`, lower `LineNames` and `Track` into repeat components, reject nested `Repeat`, and reject `Subgrid`.

Before committing, run:

```bash
rg -n "\\.tracks|TrackRepetition::auto_fill|TrackRepetition::auto_fit|TrackComponent::" crates/surgeist/src crates/surgeist/tests
```

Update every `.tracks` call site and every exhaustive `TrackComponent` match so line-name components are ignored for sizing and preserved for named maps.

- [ ] **Step 7: Add sizing extraction helpers for line-name-aware track components**

Because `TrackComponent` and `TrackRepetition` now preserve `LineNames`, update production sizing helpers so sizing code still sees only tracks:

```rust
fn track_sizing_components(components: &[TrackComponent]) -> Vec<TrackSizing>
```

Use this extraction in:
- `expand_track_components`
- fixed and auto-repeat expansion/counting helpers
- `reserved_track_space`
- `tracks_need_available_basis`
- `subgrid_components`
- any helper in `crates/surgeist/src/layout/grid/tracks.rs` that currently assumes `TrackRepetition` stores only `Vec<TrackSizing>`

Rules:
- `LineNames` contributes no sizing track.
- `Track` contributes one sizing track.
- fixed `Repeat` contributes the sizing tracks from its nested components.
- `Subgrid` remains a subgrid template marker and must not become an `auto` track.

- [ ] **Step 8: Preserve subgrid line-name repeat syntax**

In `crates/surgeist/src/style/value.rs`, change `SubgridTrack` from an already-expanded `line_names: Vec<Vec<String>>` shape to a component list that can represent fixed and auto-fill repeats. Mirror the layout shape added in Step 6:

```rust
pub struct SubgridTrack {
    pub name_components: Vec<SubgridLineNameComponent>,
}
```

Update parser/lowering helpers so:
- `subgrid [a] [b]` lowers to two `LineNames` components.
- `subgrid repeat(2, [a] [b])` lowers to a `Repeat { Count(2), ... }` component.
- `subgrid repeat(auto-fill, [a]) [end]` lowers to an `AutoFill` component followed by a fixed trailing component.
- `repeat(0, [a])` is rejected.
- more than one `repeat(auto-fill, ...)` is rejected by production subgrid expansion.

- [ ] **Step 9: Keep direct numeric call sites compiling**

Most tests and production helpers should continue using numeric `GridPlacement::line`, `GridPlacement::lines`, and `GridPlacement::span`. New adapter tests should assert `raw_grid_column`/`raw_grid_row`. Do not migrate production grid call sites in this task; Task 3 introduces the real raw-to-numeric resolution boundary.

- [ ] **Step 10: Run focused checks**

```bash
cargo test -p surgeist --test style named_grid
cargo test -p surgeist --test layout_browser_parity parse_grid_line
cargo test -p surgeist --test layout -- grid
cargo fmt --check
```

- [ ] **Step 11: Commit**

```bash
git status --short --branch
git diff --check
git add crates/surgeist/src/layout/node_input.rs crates/surgeist/src/layout/mod.rs crates/surgeist/src/layout/value.rs crates/surgeist/src/style/value.rs crates/surgeist/src/style/declaration.rs crates/surgeist/src/style/adapters/layout.rs crates/surgeist/tests/style.rs crates/surgeist/tests/layout_browser_parity/support.rs
git commit -m "Preserve named grid syntax in layout input"
```

---

## Task 2: Add Production Named Line Maps

**Files:**
- Create `crates/surgeist/src/layout/grid/named.rs`
- Modify `crates/surgeist/src/layout/grid/mod.rs`
- Modify `crates/surgeist/src/layout/grid/tracks.rs`
- Modify `crates/surgeist/src/layout/grid/tests.rs`

- [ ] **Step 1: Add failing private unit tests for explicit line names and fixed repeats**

Add tests in `crates/surgeist/src/layout/grid/tests.rs`, where private `grid::named` helpers are visible through `super::*`:

```rust
#[test]
fn production_named_lines_preserve_explicit_names_and_fixed_repeats() {
    let lines = named::named_lines_from_track_components(
        GridAxisKind::Column,
        vec![
            TrackComponent::line_names(["a"]),
            TrackComponent::px(20.0),
            TrackComponent::line_names(["b", "a"]),
            TrackComponent::repeat_count(
                2,
                vec![
                    TrackComponent::line_names(["c"]),
                    TrackComponent::px(10.0),
                    TrackComponent::line_names(["d"]),
                ],
            ),
        ],
    )
    .unwrap();

    assert_eq!(lines.named_occurrences("a"), vec![1, 2]);
    assert_eq!(lines.named_occurrences("c"), vec![2, 3]);
    assert_eq!(lines.named_occurrences("d"), vec![3, 4]);
}
```

If `TrackComponent::line_names` and `repeat_count` helpers do not exist in layout, add private `#[cfg(test)]` helpers in `crates/surgeist/src/layout/grid/tests.rs` instead of broadening the public API.

- [ ] **Step 2: Create production named-line types**

In `crates/surgeist/src/layout/grid/named.rs`:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct NamedGridLines {
    pub(super) axis: GridAxisKind,
    pub(super) explicit_track_count: usize,
    pub(super) line_names: Vec<Vec<LineNameEntry>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct LineNameEntry {
    pub(super) name: String,
    pub(super) origin: LineNameOrigin,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum LineNameOrigin {
    Explicit,
    AreaGenerated,
    LocalSubgrid,
}
```

- [ ] **Step 3: Build line maps from track components**

Implement:

```rust
pub(super) fn named_lines_from_track_components(
    axis: GridAxisKind,
    components: &[TrackComponent],
    explicit_track_count: usize,
) -> NamedGridLines
```

Rules:
- `TrackComponent::Track(_)` advances one line.
- `TrackComponent::Repeat(TrackRepetition { repeat: TrackRepeat::Count(count), components })` expands line names and tracks in source order.
- `TrackComponent::Repeat(AutoFill | AutoFit, _)` returns `NamedGridError::UnresolvedAutoRepeatNames` when a named placement depends on names inside that unresolved repeat. Numeric placement may continue using the existing resolved sizing path.
- `TrackComponent::Subgrid(_)` returns an empty local map here; subgrid inheritance is Task 5.
- Existing `LineNames` components live in style track lists, not current layout track lists. If Task 1 did not preserve line names in layout `TrackComponent`, add `TrackComponent::LineNames(Vec<String>)` now and keep track sizing expansion ignoring it.

- [ ] **Step 4: Add generated names from template areas**

Implement:

```rust
pub(super) fn add_area_generated_lines(
    axis: GridAxisKind,
    base: NamedGridLines,
    areas: &GridTemplateAreas,
) -> Result<NamedGridLines, NamedGridError>
```

Rules:
- A default `GridTemplateAreas { rows: Vec::new() }` in `NodeInput` means "unspecified" and adds no names.
- A production facts constructor for specified template areas must reject an empty row matrix as `NamedGridError::EmptyTemplateAreas`.
- Area rows must be nonempty and have equal width; invalid data should return `NamedGridError::TemplateAreaRowLengthMismatch`.
- Named area rectangles must be rectangular.
- Null cells are ignored, including parser inputs made of one or more `.` characters before they become `None`.
- Add `<area>-start` and `<area>-end` to both row and column maps.
- Area-generated maps expand explicit track count to at least the area matrix size for that axis.
- Retain `row_count`, `column_count`, `area_rectangles`, and `area_order` in a production `GridAreaNameFacts` value so subgrid clipping can recompute area-generated names.

- [ ] **Step 5: Wire module**

In `crates/surgeist/src/layout/grid/mod.rs`:

```rust
mod named;
use named::{GridNamedContext, NamedGridError, build_grid_named_context};
```

Do not change placement behavior yet.

- [ ] **Step 6: Run checks**

```bash
cargo test -p surgeist --test layout -- named_lines
cargo test -p surgeist --test layout -- grid
cargo fmt --check
```

- [ ] **Step 7: Commit**

```bash
git status --short --branch
git diff --check
git add crates/surgeist/src/layout/grid/named.rs crates/surgeist/src/layout/grid/mod.rs crates/surgeist/src/layout/grid/tracks.rs crates/surgeist/src/layout/grid/tests.rs
git commit -m "Build production named grid line maps"
```

---

## Task 3: Resolve Named Ordinary-Grid Placement Before Layout

**Files:**
- Modify `crates/surgeist/src/layout/grid/named.rs`
- Modify `crates/surgeist/src/layout/grid/mod.rs`
- Modify `crates/surgeist/src/layout/grid/placement.rs`
- Modify `crates/surgeist/tests/layout/grid.rs`
- Modify `crates/surgeist/tests/layout_oracle.rs`

- [ ] **Step 1: Add failing layout tests for named ordinary-grid placement**

Add direct production layout tests:

```rust
#[test]
fn named_grid_column_places_item_between_repeated_named_lines() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .style(1, NodeInput {
            display: Display::Grid,
            size: Size::new(Dimension::px(120.0), Dimension::px(20.0)),
            grid_template_columns: vec![
                TrackComponent::line_names(["a"]),
                TrackComponent::px(40.0),
                TrackComponent::line_names(["a"]),
                TrackComponent::px(40.0),
                TrackComponent::line_names(["a"]),
                TrackComponent::px(40.0),
            ],
            grid_template_rows: vec![TrackComponent::px(20.0)],
            ..NodeInput::DEFAULT
        })
        .style(2, NodeInput {
            raw_grid_column: RawGridPlacement::new(
                RawGridLine::NamedLine {
                    name: "a".to_string(),
                    index: 2,
                },
                RawGridLine::NamedSpan {
                    name: "a".to_string(),
                    index: 1,
                },
            ),
            ..NodeInput::DEFAULT
        });

    compute_oracle_grid(&mut tree);
    let child = tree.final_layout(2).expect("child should be laid out");

    assert_eq!(child.location.x, 40.0);
    assert_eq!(child.size.width, 40.0);
}
```

This test belongs in `crates/surgeist/tests/layout/grid.rs`, which already defines `OracleTree`, `compute_oracle_grid`, and final-layout helpers.

Every new test added by this plan must include the filter token used by that task's command in its function name:
- Task 3 named placement tests include `named_grid`.
- Task 4 absolute tests include `absolute_grid`.
- Task 4 auto-placement tests include `auto_place`.
- Task 5 subgrid tests include `subgrid_line_names` or `subgrid_named`.
- Task 6 lane tests include `grid_lanes` or `lanes`.

Also add `layout_oracle` comparisons for:
- bare area name resolving through generated `foo-start`/`foo-end`
- bare `foo` distinct from explicit `foo 1`
- negative named occurrence
- negative missing named occurrence resolving before line 1 when the fallback requires it
- missing occurrence extending into implicit grid
- backward named span fallback extending before line 1 before validation or subgrid clamping
- span/name search not counting the opposite edge
- lone named span with auto on the opposite side defaulting to one-track auto placement
- conflict resolution for start-after-end and equal named lines

- [ ] **Step 2: Implement line and span resolution**

In `named.rs`, mirror oracle behavior with production names:

```rust
pub(super) fn resolve_grid_placement(
    lines: &NamedGridLines,
    placement: &RawGridPlacement,
    auto_cursor_line: Option<isize>,
) -> Result<GridPlacement, NamedGridError>
```

Required helpers:
- `resolve_numeric_line(lines, raw_line)`
- `resolve_named_line(lines, name, occurrence)`
- `resolve_bare_ident(lines, name, side)`
- `resolve_span_from_start(lines, start_line, line)`
- `resolve_span_from_end(lines, end_line, line)`

Use these side rules:
- start-side bare `foo` tries `foo-start`, then `foo 1`.
- end-side bare `foo` tries `foo-end`, then `foo 1`.
- `span foo` means `span 1 foo`.
- named span searches skip the opposite edge line.
- `span foo / auto` and `auto / span foo` default to one-track auto placement instead of performing named lookup.
- both spans drops the end span.
- start after end swaps.
- equal lines drops the end and defaults to span 1.
- negative numeric lines resolve from the explicit grid end before placement validation.

- [ ] **Step 2a: Define the production error boundary**

`compute_grid` returns `ComputeOutput`, not `Result`. Production named-placement errors must therefore be handled at the grid boundary:

```rust
fn resolve_grid_placement_or_auto(
    lines: &NamedGridLines,
    placement: &RawGridPlacement,
    auto_cursor_line: Option<isize>,
) -> GridPlacement {
    match resolve_grid_placement(lines, placement, auto_cursor_line) {
        Ok(placement) => placement,
        Err(error) => {
            debug_assert!(false, "invalid named grid placement: {error:?}");
            GridPlacement::AUTO
        }
    }
}
```

Rules:
- CSS/style parser validation must reject authored invalid values before layout lowering whenever possible.
- Direct layout API misuse must not panic in release layout; it falls back to `GridPlacement::AUTO` for the affected axis.
- Tests in `crates/surgeist/src/layout/grid/tests.rs` should assert direct invalid raw placement fallback for reserved names, zero lines, and area-not-found.
- Internal errors caused by implementation bugs should still be visible in debug builds through `debug_assert!`.

- [ ] **Step 3: Split grid initialization into explicit and placement-dependent phases**

The current `initialize_grid_tracks` reads child placement while it computes visible cell count, leading implicit tracks, and track requirements. That is too early for named placement because named resolution needs explicit line maps first.

Replace the single initialization path with:

```rust
struct ExplicitGridTrackInitialization<Node> {
    children: Vec<Node>,
    column_template: GridAxisTemplate,
    row_template: GridAxisTemplate,
    explicit_columns: usize,
    explicit_rows: usize,
    gap: Size,
    named_context: GridNamedContext,
    subgrid_report: GridSubgridReport<Node>,
}

struct CompletedGridTrackInitialization<Node> {
    column_tracks: Vec<TrackSizing>,
    row_tracks: Vec<TrackSizing>,
    context: GridContainerContext,
    placements: GridPlacementContext<Node>,
    subgrid_report: GridSubgridReport<Node>,
}
```

Add helpers:

```rust
fn initialize_explicit_grid_tracks<Tree>(
    tree: &Tree,
    node: <Tree as Traverse>::Node,
    style: &NodeInput,
    constants: &Constants,
    parent_context: &GridParentContext,
) -> ExplicitGridTrackInitialization<<Tree as Traverse>::Node>
where
    Tree: Compute

fn complete_grid_tracks_after_placement<Node>(
    explicit: ExplicitGridTrackInitialization<Node>,
    placements: GridPlacementContext<Node>,
    constants: &Constants,
) -> CompletedGridTrackInitialization<Node>
```

`initialize_explicit_grid_tracks` may inspect children for visibility and subgrid eligibility, but must not read `grid_column`, `grid_row`, `raw_grid_column`, or `raw_grid_row` to size the explicit grid. `complete_grid_tracks_after_placement` owns visible cell count, leading implicit tracks, placement track requirements, and auto-track extension.

- [ ] **Step 4: Add resolved child-order placement context**

Add a production context:

```rust
struct ResolvedGridItemPlacement {
    column: GridPlacement,
    row: GridPlacement,
}

struct GridPlacementContext<Node> {
    children: Vec<Node>,
    items: Vec<ResolvedGridItemPlacement>,
}
```

The `children` and `items` vectors must have identical order and length. Do not add `Node: Eq` requirements. Pass child-order slices through downstream helpers and zip with existing `tree.children(node)` snapshots.

Build the context once:

```rust
fn resolve_grid_child_placements<Node>(
    children: Vec<Node>,
    raw_placements: impl IntoIterator<Item = (RawGridPlacement, RawGridPlacement)>,
    named_context: &GridNamedContext,
) -> Result<GridPlacementContext<Node>, NamedGridError>
```

- [ ] **Step 5: Route track requirement and leading implicit tracks through resolved placement**

Change signatures:

```rust
fn grid_track_requirement_from_placements(
    placements: &[ResolvedGridItemPlacement],
) -> Size<usize>

fn leading_implicit_tracks_from_placements(
    placements: &[ResolvedGridItemPlacement],
    axis: GridAxisKind,
    explicit_count: usize,
) -> usize
```

Keep old functions only as test wrappers if needed.

- [ ] **Step 6: Route definite and auto placement through resolved placement**

Change placement helpers in `placement.rs` to accept numeric `GridPlacement` values that came from `GridPlacementContext`:

```rust
pub(super) fn placement_track_requirement(placement: GridPlacement) -> usize
pub(super) fn fully_definite_area(
    column: GridPlacement,
    row: GridPlacement,
    columns: &[Scalar],
    rows: &[Scalar],
    gap: Size,
    lines: GridLines,
) -> Option<GridArea>
```

Use raw `RawGridPlacement` only at the named-resolution boundary.

- [ ] **Step 7: Thread placement context through sizing call sites**

Add `placements: &'a GridPlacementContext<Node>` or an `&'a [ResolvedGridItemPlacement]` slice to every production grid sizing path that currently re-reads child placement:
- intrinsic grid construction in `crates/surgeist/src/layout/grid/tracks.rs`
- constrained track sizing in `tracks.rs`
- cyclic percent sizing in `crates/surgeist/src/layout/grid/mod.rs`
- subgrid intrinsic traversal in `crates/surgeist/src/layout/grid/subgrid.rs`

These paths must consume child-order placement slices instead of reading `tree.node_input(child).grid_column` or `grid_row`.

- [ ] **Step 8: Run checks**

```bash
cargo test -p surgeist --test layout -- named_grid
cargo test -p surgeist --test layout_oracle named_grid
cargo test -p surgeist --test layout -- grid
cargo fmt --check
```

- [ ] **Step 9: Commit**

```bash
git status --short --branch
git diff --check
git add crates/surgeist/src/layout/grid/named.rs crates/surgeist/src/layout/grid/mod.rs crates/surgeist/src/layout/grid/placement.rs crates/surgeist/src/layout/grid/tracks.rs crates/surgeist/src/layout/grid/subgrid.rs crates/surgeist/tests/layout/grid.rs crates/surgeist/tests/layout_oracle.rs
git commit -m "Resolve named grid placement in engine"
```

---

## Task 4: Thread Resolved Placements Through Child And Absolute Layout

**Files:**
- Modify `crates/surgeist/src/layout/grid/mod.rs`
- Modify `crates/surgeist/src/layout/grid/child.rs`
- Modify `crates/surgeist/src/layout/grid/placement.rs`
- Modify `crates/surgeist/src/layout/grid/lanes.rs`
- Modify `crates/surgeist/tests/layout/grid.rs`

- [ ] **Step 1: Add failing tests for named absolute placement and auto placement collision**

Add tests:
- absolute item with `grid-column: a / c`
- in-flow named item occupying a cell before an auto-placed sibling
- named implicit fallback increasing explicit requirement before auto placement

Assert final child rectangles, not only successful computation.

- [ ] **Step 2: Pass child-order placement slices, not node lookups**

Do not add `Node: Eq` or lookup-by-node APIs. Extend `GridChildLayoutInput` and absolute layout inputs with child-order placement data:

```rust
struct GridChildLayoutInput<'a, Node> {
    children: &'a [Node],
    placements: &'a [ResolvedGridItemPlacement],
}
```

Every helper that iterates children must zip the same child snapshot with the same placement slice. Add debug assertions that lengths match at every public helper boundary.

- [ ] **Step 3: Update child layout input**

Extend `GridChildLayoutInput`:

```rust
children: &'a [Node],
placements: &'a [ResolvedGridItemPlacement],
```

Replace direct reads of:

```rust
style.grid_column
style.grid_row
```

inside grid child layout with resolved placements from the context.

- [ ] **Step 4: Update absolute layout**

Change `absolute_grid_area` and every caller to pass resolved numeric placements. Named absolute placement should resolve against the same container line context as in-flow children.

- [ ] **Step 5: Update grid-lanes compile boundary**

Any lane function that currently reads `NodeInput.grid_column`, `NodeInput.grid_row`, `raw_grid_column`, or `raw_grid_row` should instead receive the child-order numeric `GridPlacement` slice from `GridPlacementContext`. If a lane function still needs raw placement for a later task, name the field `raw_placement` explicitly and keep it out of numeric helpers.

- [ ] **Step 6: Run checks**

```bash
cargo test -p surgeist --test layout -- absolute_grid
cargo test -p surgeist --test layout -- auto_place
cargo test -p surgeist --test layout -- grid
cargo test -p surgeist --test layout_oracle
cargo fmt --check
```

- [ ] **Step 7: Commit**

```bash
git status --short --branch
git diff --check
git add crates/surgeist/src/layout/grid/mod.rs crates/surgeist/src/layout/grid/child.rs crates/surgeist/src/layout/grid/placement.rs crates/surgeist/src/layout/grid/lanes.rs crates/surgeist/tests/layout/grid.rs
git commit -m "Thread resolved grid placements through layout"
```

---

## Task 5: Implement Subgrid Named Line Inheritance

**Files:**
- Modify `crates/surgeist/src/layout/grid/named.rs`
- Modify `crates/surgeist/src/layout/grid/subgrid.rs`
- Modify `crates/surgeist/src/layout/grid/mod.rs`
- Modify `crates/surgeist/tests/layout/grid.rs`
- Modify `crates/surgeist/tests/layout_oracle.rs`
- Modify `crates/surgeist/tests/layout_browser_parity.rs`

- [ ] **Step 1: Add failing tests for inherited subgrid names**

Add oracle comparison and production layout tests for:
- inherited explicit parent names over a subgrid span
- local subgrid names merged at the corresponding local line
- reversed subgrid axis preserving name sets while reversing local line order
- parent area-generated names clipped to the subgrid span
- named placement beyond subgrid span clamped in the subgridded axis

Use existing parity fixtures as smoke targets:

```bash
SURGEIST_PARITY_FILTER=subgrid_line_names_004_b_to_b_minus_1 cargo test -p surgeist --test layout_browser_parity runs_all_checked_in_browser_parity_xml -- --ignored
SURGEIST_PARITY_FILTER=subgrid_line_names_repeat_outer_span_a_to_a_8 cargo test -p surgeist --test layout_browser_parity runs_all_checked_in_browser_parity_xml -- --ignored
```

Expected before implementation: classification remains `engine gap: named grid placement lowering`.

- [ ] **Step 2: Carry named-line facts in parent context**

Extend `InheritedGridAxis`:

```rust
named_lines: NamedGridLines,
area_facts: Option<GridAreaNameFacts>,
```

Extend `GridParentContext` construction in child layout and intrinsic subgrid traversal so nested subgrids receive both used tracks and named-line maps.

Also extend `GridContainerContext` with the container's named facts:

```rust
named_columns: NamedGridLines,
named_rows: NamedGridLines,
area_facts: Option<GridAreaNameFacts>,
```

When subgrid intrinsic traversal fabricates a `GridParentContext`, it must derive clipped `NamedGridLines` from the resolved subgrid area and parent `GridAreaNameFacts`; it must not use empty placeholder name maps. Add tests for a nested subgrid whose intrinsic contribution depends on a parent area-generated name.

- [ ] **Step 3: Expand local subgrid line names**

In `named.rs`, implement:

```rust
pub(super) fn expand_subgrid_local_line_names(
    axis: GridAxisKind,
    used_track_count: usize,
    components: &[SubgridLineNameComponent],
) -> Result<Vec<Vec<LineNameEntry>>, NamedGridError>
```

Rules:
- `LineNames(names)` consumes one line-name slot.
- `Repeat { Count(n), line_name_sets }` appends the sets `n` times and rejects `n == 0`.
- `Repeat { AutoFill, line_name_sets }` fills remaining slots while reserving trailing fixed slots.
- Empty `line_name_sets` in auto-fill contributes no names and must terminate without an infinite loop.
- More than one auto-fill repeat returns `NamedGridError::MultipleAutoFillRepeats`.
- The final result has exactly `used_track_count + 1` line-name slots.

- [ ] **Step 4: Merge inherited, area-generated, and local subgrid names**

Implement:

```rust
pub(super) fn inherit_subgrid_named_lines(
    parent: &NamedGridLines,
    parent_start: usize,
    parent_end: usize,
    reversed: bool,
    local_line_names: &[Vec<LineNameEntry>],
    parent_area_facts: Option<&GridAreaNameFacts>,
) -> Result<NamedGridLines, NamedGridError>
```

Rules:
- Parent explicit names are copied over the parent span.
- Parent area-generated names are recomputed from rectangles clipped to the subgrid span.
- Local names are appended after inherited/area-generated names.
- Reversed axes map local line `1` to parent `parent_end`, local line `N+1` to parent `parent_start`.

- [ ] **Step 5: Resolve subgrid child named placements with clamping**

Use:

```rust
pub(super) fn resolve_subgrid_placement(
    lines: &NamedGridLines,
    placement: &RawGridPlacement,
    auto_cursor_line: Option<isize>,
) -> Result<GridPlacement, NamedGridError>
```

This should perform ordinary named resolution first, then clamp final start/end to `1..=explicit_track_count + 1` for subgridded axes.

If clamping collapses both lines to the same edge, expand to the nearest edge track:
- collapsed at or before line 1 becomes `1 / 2`
- collapsed at or after the last line becomes `explicit_track_count / explicit_track_count + 1`

Add a production/oracle comparison for the oracle case where `Number(2) / span 3` in a one-track subgrid resolves to `1 / 2`.

- [ ] **Step 6: Run checks**

```bash
cargo test -p surgeist --test layout -- subgrid_line_names
cargo test -p surgeist --test layout_oracle subgrid_named
SURGEIST_PARITY_FILTER=subgrid_line_names_004_b_to_b_minus_1 cargo test -p surgeist --test layout_browser_parity runs_all_checked_in_browser_parity_xml -- --ignored
SURGEIST_PARITY_FILTER=subgrid_line_names_repeat_outer_span_a_to_a_8 cargo test -p surgeist --test layout_browser_parity runs_all_checked_in_browser_parity_xml -- --ignored
cargo fmt --check
```

- [ ] **Step 7: Commit**

```bash
git status --short --branch
git diff --check
git add crates/surgeist/src/layout/grid/named.rs crates/surgeist/src/layout/grid/subgrid.rs crates/surgeist/src/layout/grid/mod.rs crates/surgeist/tests/layout/grid.rs crates/surgeist/tests/layout_oracle.rs crates/surgeist/tests/layout_browser_parity.rs
git commit -m "Inherit named grid lines through subgrid"
```

---

## Task 6: Wire Named Placement Through Grid-Lanes

**Files:**
- Modify `crates/surgeist/src/layout/grid/lanes.rs`
- Modify `crates/surgeist/src/layout/grid/mod.rs`
- Modify `crates/surgeist/tests/layout_oracle.rs`
- Modify `crates/surgeist/tests/layout/grid.rs`

- [ ] **Step 1: Add failing named grid-lanes tests**

Add tests for:
- lane item start/end by named ordinary grid line
- lane item spanning a named implicit fallback line
- lane placement inside a subgridded lane axis using inherited names

Each test should compare production output to the oracle where possible.

- [ ] **Step 2: Pass resolved placements into lane placement input**

Change lane placement input fields that currently read raw or node-input placement:

```rust
pub(super) struct GridLanesResolvedItem<Node> {
    pub(super) node: Node,
    pub(super) column: GridPlacement,
    pub(super) row: GridPlacement,
}
```

Use this in lane placement and sizing functions that need item spans.

- [ ] **Step 3: Pass resolved placements into lane intrinsic sizing**

Update lane intrinsic sizing inputs before final lane layout:

```rust
pub(super) struct LaneIntrinsicTrackSizeInput<'a, Node> {
    pub(super) children: &'a [Node],
    pub(super) placements: &'a [ResolvedGridItemPlacement],
}
```

Thread the same child-order placement slice into:
- `lane_intrinsic_track_sizes`
- every caller in `crates/surgeist/src/layout/grid/mod.rs`
- `resolve_grid_lanes_placement_with_resolved_tracks`
- any helper that currently reads `tree.node_input(child).grid_column` or `grid_row` inside `crates/surgeist/src/layout/grid/lanes.rs`

- [ ] **Step 4: Keep lane algorithm numeric after resolution**

Do not duplicate named-line lookup inside `lanes.rs`. All named syntax should enter lanes as resolved numeric placement from `named.rs`.

- [ ] **Step 5: Run checks**

```bash
cargo test -p surgeist --test layout_oracle lanes
cargo test -p surgeist --test layout -- grid_lanes
cargo test -p surgeist --test layout -- named_grid
cargo fmt --check
```

- [ ] **Step 6: Commit**

```bash
git status --short --branch
git diff --check
git add crates/surgeist/src/layout/grid/lanes.rs crates/surgeist/src/layout/grid/mod.rs crates/surgeist/tests/layout_oracle.rs crates/surgeist/tests/layout/grid.rs
git commit -m "Resolve named placement for grid lanes"
```

---

## Task 7: Browser Parity And Gap Reclassification

**Files:**
- Modify `crates/surgeist/tests/layout_browser_parity.rs`
- Modify `crates/surgeist/tests/layout_browser_parity/support.rs`
- Modify checked-in fixtures under `crates/surgeist/tests/layout_browser_parity/html/subgrid`
- Modify corpus manifest/reporting inputs when a browser fail-list fixture needs
  auditable expected-failure handling; treat generated XML as output only

- [ ] **Step 1: Run known named-grid parity filters**

```bash
SURGEIST_PARITY_FILTER=subgrid_line_names_004_b_to_b_minus_1 cargo test -p surgeist --test layout_browser_parity runs_all_checked_in_browser_parity_xml -- --ignored
SURGEIST_PARITY_FILTER=subgrid_line_names_repeat_outer_span_a_to_a_8 cargo test -p surgeist --test layout_browser_parity runs_all_checked_in_browser_parity_xml -- --ignored
```

Expected after Tasks 1-6: both pass or fail on a non-named-grid issue.

- [ ] **Step 2: Remove stale engine-gap classification**

Remove or narrow the `engine gap: named grid placement lowering` classification once the named placement fixtures pass. If a fixture still fails due to a distinct feature, classify it by that feature with the exact reason.

- [ ] **Step 3: Add or refresh named-grid parity fixtures**

This instruction is superseded by the parity corpus consolidation plan. Raw
multi-assertion WPT HTML is now a first-class corpus input under
`crates/surgeist/tests/layout_browser_parity/wpt`, with manifest-driven
assertion fan-out through `surgeist-layout-generate`. Local constrained HTML may
remain as reduced Surgeist fixtures, but new or refreshed WPT coverage should
enter through the importer/manifest/generator path rather than manual
single-assertion conversion. Add or import coverage for:
- ordinary repeated line names
- `grid-template-areas` generated names
- negative named occurrence
- subgrid inherited explicit names
- subgrid repeated local names
- grid-lanes named placement

- [ ] **Step 4: Run parity parser tests and filtered fixtures**

```bash
cargo test -p surgeist --test layout_browser_parity parse_grid_line
cargo test -p surgeist --test layout_browser_parity -- --ignored
```

If the full ignored parity suite is too broad for the current machine, run every named-grid filter added in this task and document skipped unrelated parity domains in the commit message body.

- [ ] **Step 5: Commit**

```bash
git status --short --branch
git diff --check
git add crates/surgeist/tests/layout_browser_parity.rs crates/surgeist/tests/layout_browser_parity/support.rs crates/surgeist/tests/layout_browser_parity/html/subgrid crates/surgeist/tests/layout_browser_parity/wpt crates/surgeist/tests/layout_browser_parity/xml crates/surgeist/tests/layout_browser_parity/wpt/manifests
git commit -m "Cover named grid engine parity"
```

---

## Task 8: Full Verification And Cleanup

**Files:**
- Modify any files touched by cleanup from earlier tasks.

- [ ] **Step 1: Search for stale unsupported named-grid lowering**

```bash
rg -n "named grid line placement|named grid placement lowering|NamedLine|NamedSpan|GridLine::Named" crates/surgeist/src crates/surgeist/tests docs/superpowers
```

Expected:
- no production adapter rejection for named placement
- no stale parity classification claiming named-grid placement lowering is unsupported
- remaining docs mention historical gap only in completed plan/spec context

- [ ] **Step 2: Run focused suites**

```bash
cargo test -p surgeist --test oracle
cargo test -p surgeist --test layout_oracle
cargo test -p surgeist --test layout -- grid
cargo test -p surgeist --test layout_browser_parity parse_grid_line
```

- [ ] **Step 3: Run whole crate checks**

```bash
cargo fmt -p surgeist --check
cargo test -p surgeist
cargo clippy -p surgeist --all-targets --all-features -- -D warnings
```

- [ ] **Step 4: Commit cleanup if needed**

```bash
git status --short --branch
git diff --check
git diff --name-only
git diff --name-only | xargs git add
git commit -m "Clean up named grid engine implementation"
```

Only run the `xargs git add` command when `git diff --name-only` prints cleanup files. Skip the cleanup commit if Step 1-3 produce no file changes.

---

## Review Requirements

This plan is complete only after at least three clean-context review cycles:

1. Review against the completed named-grid oracle for behavioral coverage.
2. Review against the production grid/subgrid/grid-lanes engine structure for integration risks.
3. Review against implementation practicality, test coverage, and commit/verification sequencing.

Each cycle must record:
- reviewer focus
- concrete recommendations
- changes made to this plan
- remaining accepted risks

---

## Review Log

### Review Cycle 1

Status: complete.

Reviewer focus: behavioral coverage against the completed named-grid oracle.

Recommendations implemented:
- Added `BareIdent` preservation in style and layout so bare `foo` remains distinct from explicit `foo 1`.
- Added shorthand expansion requirements for `grid-column`, `grid-row`, and `grid-area`.
- Added lone named-span normalization rules for `span foo / auto` and `auto / span foo`.
- Added subgrid local name-repeat preservation and expansion rules, including fixed repeat, auto-fill, zero repeat, and multiple auto-fill errors.
- Added collapsed subgrid clamp expansion to the nearest edge track.
- Added retained template-area facts for area order, rectangle data, row count, and column count.
- Added stricter template-area validation and dot-run null cell handling.
- Added required error coverage and negative-side implicit fallback coverage.

Remaining accepted risks:
- The exact CSS parser entry points for `grid-area` shorthand may differ from the declaration helper names shown in this plan; the implementation must place the tests at the actual parser/declaration boundary if the helper API differs.

### Review Cycle 2

Status: complete.

Reviewer focus: integration feasibility against the production grid, subgrid, and grid-lanes engine structure.

Recommendations implemented:
- Reworked Task 1 to add `RawGridLine`/`RawGridPlacement` alongside existing numeric `GridPlacement` instead of changing `GridPlacement` in a way that would break the checkpoint.
- Added a two-phase grid initialization plan: explicit-track/named-context construction before placement-dependent track completion.
- Replaced node lookup with child-order placement slices to avoid adding generic `Node: Eq` requirements.
- Added placement-context threading through intrinsic sizing, constrained sizing, cyclic percent sizing, child layout, absolute layout, and subgrid traversal.
- Added grid-lanes intrinsic sizing call sites, not only final lane placement.
- Added `NamedGridLines` and `GridAreaNameFacts` to parent/container context for real layout and intrinsic subgrid traversal.
- Added a sizing extraction layer for line-name-aware `TrackComponent`/`TrackRepetition` values.

Remaining accepted risks:
- The exact split of `initialize_grid_tracks` may expose additional helper boundaries in `tracks.rs`; the plan now names the required phases and call-site categories rather than prescribing every internal helper name.

### Review Cycle 3

Status: complete.

Reviewer focus: implementation practicality, executable snippets, test coverage, and commit/checkpoint quality.

Recommendations implemented:
- Added a concrete production error boundary: invalid direct raw layout input falls back to auto placement in release and trips a debug assertion in debug builds.
- Moved named-map tests to private `crates/surgeist/src/layout/grid/tests.rs` unit tests instead of referencing a nonexistent public `test_support` module.
- Replaced imaginary layout fixture names with the existing `OracleTree`/`compute_oracle_grid` harness in `crates/surgeist/tests/layout/grid.rs`.
- Added exhaustive-match searches for new `GridLine::BareIdent` handling, including hashing, validation, lowering, and parser tests.
- Added `auto_fill_components` and `auto_fit_components` constructors plus `.tracks`/`TrackComponent` migration searches for track repeat changes.
- Added test naming rules so focused verification filters cannot silently skip new tests.
- Replaced the misleading cleanup staging command with `git diff --name-only | xargs git add`, guarded by the preceding file list.

Remaining accepted risks:
- Some code snippets use planned helper constructors such as `TrackComponent::line_names`; when those helpers are private test conveniences, the worker must add them locally in the test module rather than exporting them broadly.
