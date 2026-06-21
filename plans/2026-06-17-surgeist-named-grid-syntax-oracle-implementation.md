# Surgeist Named Grid Syntax Oracle Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement a spec-complete named grid syntax oracle for Surgeist tests, covering named lines, repeated names, positive and negative named occurrences, named spans, implicit fallback lines, `grid-template-areas` generated names, fixed-repeat expansion, subgrid inherited/merged names, and browser parity fixture lowering.

**Architecture:** Add a test-only named-grid oracle module that resolves typed line-name facts into existing oracle placement facts. The module must remain an oracle, not a second production engine: it consumes explicit track/name/area/subgrid facts, emits transparent reports, and does not traverse production trees, resolve production styles, measure children, or call `compute_grid`.

**Tech Stack:** Rust test support under `crates/surgeist/tests/support/oracle/grid`, pure oracle tests in `crates/surgeist/tests/oracle.rs`, fixture-lowering parser tests in `crates/surgeist/tests/layout_browser_parity/support.rs`, current style grid syntax in `crates/surgeist/src/style/value.rs`, and final verification with `cargo test -p surgeist`.

**Primary References:**
- CSS Grid Layout Module Level 2, especially named grid lines, grid placement grammar, implicit fallback lines for missing named occurrences, `grid-template-areas` generated names, and subgrid line names: `https://www.w3.org/TR/css-grid-2/`
- Local prior oracle plan: `docs/superpowers/plans/2026-06-16-surgeist-subgrid-grid-lanes-oracle-implementation.md`
- Local current boundary: `crates/surgeist/tests/support/oracle/grid/placement.rs`
- Local style vocabulary: `crates/surgeist/src/style/value.rs`

---

## File Map

- Create `crates/surgeist/tests/support/oracle/grid/named.rs`
  - Owns line-name maps, fixed-repeat expansion, area-generated line names, named line lookup, named span lookup, named placement resolution, and subgrid name inheritance/merging reports.

- Modify `crates/surgeist/tests/support/oracle/grid/mod.rs`
  - Exposes the named-grid oracle front-door types and functions.

- Modify `crates/surgeist/tests/support/oracle/grid/placement.rs`
  - Keep the existing positive numeric placement solver intact.
  - Add only a minimal public constructor such as `AxisPlacement::try_new(start, end) -> Result<AxisPlacement, PlacementError>`.
  - Keep named-resolution errors owned by `NamedGridError` in `named.rs` and map `PlacementError` at the named-oracle boundary.

- Modify `crates/surgeist/tests/oracle.rs`
  - Add pure oracle tests for every named syntax phase and a small number of composed named-placement scenarios.

- Modify `crates/surgeist/tests/layout_browser_parity/support.rs`
  - Extend fixture lowering to parse named grid line syntax into existing style values.
  - Add tests proving checked-in single-assertion fixture HTML/XML can carry named lines and named spans without being rejected by parser helpers.

- Optional modify `crates/surgeist/tests/layout_browser_parity.rs`
  - Add or refine classification in `classified_error_kind` only if named parser support exposes production-engine failures that were previously hidden by parse rejection.

---

## Non-Goals

- Do not implement production named grid layout in this plan.
- Do not implement named lowering in `crates/surgeist/src/style/adapters/layout.rs`.
- Do not change `crates/surgeist/src/layout/node_input.rs` or `crates/surgeist/src/layout/grid/*`.
- Do not add a CSS parser to the oracle module.
- Do not expand `auto-fill` or `auto-fit` by guessing layout-dependent repetition counts inside the oracle.
- Do not collapse the existing numeric `LinePlacement` oracle into a broader production-shaped resolver.

---

## Spec Rules The Oracle Must Model

- Positive numeric line `N` resolves to line `N`; negative numeric line `-N` resolves from the explicit grid end line before implicit placement fallback is considered.
- A named line lookup with positive occurrence `name N` counts matching explicit line names from the start side.
- A named line lookup with negative occurrence `name -N` counts matching explicit line names from the end side.
- If a named lookup requests more occurrences than the explicit grid contains, implicit grid lines on the relevant search side are treated as having that name for lookup.
- `span name` means `span 1 name`.
- `span N name` counts the `N`th matching named line away from the opposite edge in the span search direction.
- Named spans can extend into implicit grid space if explicit named lines are insufficient.
- In normal grids, named spans can extend into implicit grid space if explicit named lines are insufficient.
- In subgridded axes, named placement may resolve against hypothetical implicit lines for rule evaluation, but the final item area clamps to the subgrid's explicit span because subgrids do not create implicit tracks in subgridded dimensions.
- Named span searches do not count the opposite edge line. `grid-column: 1 / span 2 a` searches after line `1`; if named `a` occurs at lines `1`, `3`, and `5`, the resolved end is line `5`.
- Placement conflict handling follows CSS Grid: if both sides are spans, the end-side span is dropped; if the start line is after the end line, the two lines are swapped; if both sides resolve to the same line, the end side is dropped and defaults to `span 1`.
- Bare `<custom-ident>` placement is side-aware: start-side bare `foo` first resolves `foo-start`, end-side bare `foo` first resolves `foo-end`, and each falls back to `foo 1` only when the preferred generated-name form has no matching line.
- Repeated line names are distinct occurrences and must preserve source order.
- Fixed `repeat(N, ...)` expands line-name lists and track counts before lookup.
- `grid-template-areas` generates `<area>-start` and `<area>-end` line names on both axes at the area boundary lines, in addition to explicit names already present on those lines.
- A placement by bare area name must resolve through generated `foo-start` and `foo-end` names on both axes.
- Template areas require a non-empty rectangular matrix: every row has the same nonzero cell count and every named area occupies a rectangle.
- Template area null cells are tokens made entirely of one or more `.` characters, not only the single token `"."`.
- Template areas contribute to the explicit grid size; area-generated line maps expand the base line map to `max(base.explicit_track_count, template_axis_track_count)`.
- The oracle uses deterministic line-name ordering for assertions: explicit names first, then generated area names in row-major first-discovery order, then local subgrid names.
- Line names carry origin: explicit, area-generated, or local subgrid. The origin is observable in reports even when `line_names(line)` returns strings for readable assertions.
- Subgrid line names include inherited explicit parent names over the parent span plus locally supplied subgrid line names, merged at corresponding subgrid lines.
- Parent area-generated names are not blindly copied into subgrids; they are recomputed from parent area rectangles clipped to the subgrid span.
- Reversed subgrid axes map local line order to parent line order in reverse while preserving the set of names exposed at each local line.
- Subgrid local `<line-name-list>` supports name-repeat syntax, including `repeat(N, [name])` and `repeat(auto-fill, [name])` expansion against the used subgrid span.
- Reserved idents `auto` and `span` are invalid line names and invalid grid-line custom idents in both oracle constructors and parity parser helpers.
- Unsupported input must return an explicit oracle error with enough context for assertions.

---

## Public Oracle Shape

Implement this shape in `crates/surgeist/tests/support/oracle/grid/named.rs`. Field names may be refined during implementation, but the public capability and report granularity must remain.

```rust
use super::placement::{AxisPlacement, GridAxis};
use super::subgrid::TrackSpan;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamedGridLines {
    pub axis: GridAxis,
    pub explicit_track_count: usize,
    pub line_names: Vec<Vec<LineNameEntry>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LineNameEntry {
    pub name: String,
    pub origin: LineNameOrigin,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LineNameOrigin {
    Explicit,
    AreaGenerated,
    LocalSubgrid,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AreaGeneratedFacts {
    pub areas: TemplateAreas,
    pub columns: NamedGridLines,
    pub rows: NamedGridLines,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamedLineOccurrence {
    pub line: isize,
    pub names: Vec<String>,
    pub explicit: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NamedGridLine {
    Auto,
    Number(isize),
    BareIdent(String),
    Named { name: String, occurrence: isize },
    Span { name: Option<String>, count: usize },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlacementSide {
    Start,
    End,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamedAxisPlacement {
    pub start: NamedGridLine,
    pub end: NamedGridLine,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamedLookupReport {
    pub axis: GridAxis,
    pub name: String,
    pub requested_occurrence: isize,
    pub resolved_line: isize,
    pub explicit_matches: Vec<isize>,
    pub implicit_lines_assumed_named: Vec<isize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamedPlacementReport {
    pub axis: GridAxis,
    pub original_start: NamedGridLine,
    pub original_end: NamedGridLine,
    pub normalized_start: NamedGridLine,
    pub normalized_end: NamedGridLine,
    pub conflict_resolution: Option<NamedPlacementConflictResolution>,
    pub start_lookup: Option<NamedLookupReport>,
    pub end_lookup: Option<NamedLookupReport>,
    pub resolved: AxisPlacement,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NamedPlacementConflictResolution {
    DroppedEndSpan,
    SwappedResolvedLines,
    DroppedEqualEndLine,
    DefaultedLoneNamedSpanToOne,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamedGridAreaPlacement {
    pub row: NamedAxisPlacement,
    pub column: NamedAxisPlacement,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NamedTrackComponent {
    LineNames(Vec<String>),
    Track,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SubgridNameComponent {
    LineNames(Vec<String>),
    Repeat {
        count: SubgridNameRepeatCount,
        line_name_sets: Vec<Vec<String>>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubgridNameRepeatCount {
    Number(usize),
    AutoFill,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubgridNameExpansionReport {
    pub axis: GridAxis,
    pub used_track_count: usize,
    pub local_line_names: Vec<Vec<String>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubgridAxisPlacementReport {
    pub unclamped_start_line: isize,
    pub unclamped_end_line: isize,
    pub clamped: NamedPlacementReport,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NamedGridError {
    EmptyLineNames { axis: GridAxis },
    LineNameCountMismatch { axis: GridAxis, explicit_track_count: usize, line_count: usize },
    ReservedLineName { name: String },
    ZeroLine,
    ZeroSpan,
    AutoWithoutCursor,
    LineBeforeFirst { axis: GridAxis, start_line: isize, end_line: isize },
    EmptyTemplateAreas,
    TemplateAreaRowLengthMismatch { expected: usize, actual: usize, row: usize },
    AreaNotRectangular { area: String },
    AreaNotFound { area: String },
    ZeroRepeat,
    SubgridSpanOutOfRange { axis: GridAxis },
}
```

Use the existing `subgrid::TrackSpan`, which is already re-exported by `grid/mod.rs`, for subgrid line-name inheritance inputs.

---

## Task 1: Add Named Line Map Vocabulary

**Files:**
- Create `crates/surgeist/tests/support/oracle/grid/named.rs`
- Modify `crates/surgeist/tests/support/oracle/grid/mod.rs`
- Modify `crates/surgeist/tests/oracle.rs`

- [ ] Add failing tests:

```rust
#[test]
fn oracle_named_grid_lines_preserve_repeated_names_in_source_order() {
    let lines = support::oracle::grid::NamedGridLines::new(
        support::oracle::grid::GridAxis::Column,
        3,
        vec![vec!["a"], vec!["b", "a"], vec!["a"], vec!["b"]],
    )
    .unwrap();

    assert_eq!(lines.named_occurrences("a"), vec![1, 2, 3]);
    assert_eq!(lines.named_occurrences("b"), vec![2, 4]);
}

#[test]
fn oracle_named_grid_lines_reject_mismatched_line_count() {
    let err = support::oracle::grid::NamedGridLines::new(
        support::oracle::grid::GridAxis::Row,
        2,
        vec![vec!["a"], vec!["b"]],
    )
    .unwrap_err();

    assert_eq!(
        err,
        support::oracle::grid::NamedGridError::LineNameCountMismatch {
            axis: support::oracle::grid::GridAxis::Row,
            explicit_track_count: 2,
            line_count: 2,
        }
    );
}
```

- [ ] Implement `NamedGridLines::new`, `NamedGridLines::empty(axis, explicit_track_count)`, `named_occurrences`, and `line_names(line)`.
- [ ] Make `NamedGridLines::new` accept nested string-like values, such as `Vec<Vec<&str>>`, and normalize to owned `String`.
- [ ] Make `line_names(line)` return `Vec<&str>` so tests can assert readable string slices while the oracle stores owned names.
- [ ] Validate that `line_names.len() == explicit_track_count + 1`.
- [ ] Reject reserved line names `auto` and `span` as `NamedGridError::ReservedLineName`.
- [ ] Add `pub mod named;` to `grid/mod.rs`.
- [ ] Export every front-door named-grid symbol used by tests from `grid/mod.rs`: `AreaGeneratedFacts`, `LineNameEntry`, `LineNameOrigin`, `NamedAxisPlacement`, `NamedGridAreaPlacement`, `NamedGridLine`, `NamedGridLines`, `NamedGridError`, `NamedLookupReport`, `NamedPlacementConflictResolution`, `NamedPlacementReport`, `NamedTrackComponent`, `PlacementSide`, `SubgridAxisPlacementReport`, `SubgridLineNameInheritanceReport`, `SubgridNameComponent`, `SubgridNameExpansionReport`, `SubgridNameRepeatCount`, `TemplateAreas`, and all `resolve_*`, `expand_*`, `area_*`, and `inherit_*` helpers.
- [ ] Add this helper near the named-grid oracle tests and use it consistently:

```rust
fn named_columns(
    explicit_track_count: usize,
    line_names: Vec<Vec<&str>>,
) -> support::oracle::grid::NamedGridLines {
    support::oracle::grid::NamedGridLines::new(
        support::oracle::grid::GridAxis::Column,
        explicit_track_count,
        line_names,
    )
    .unwrap()
}
```

- [ ] Run:

```bash
cargo test -p surgeist --test oracle oracle_named_grid_lines
```

- [ ] Commit:

```bash
git add crates/surgeist/tests/support/oracle/grid/named.rs crates/surgeist/tests/support/oracle/grid/mod.rs crates/surgeist/tests/oracle.rs
git commit -m "Add named grid oracle line maps"
```

---

## Task 2: Resolve Positive And Negative Named Lines

**Files:**
- Modify `crates/surgeist/tests/support/oracle/grid/named.rs`
- Modify `crates/surgeist/tests/oracle.rs`

- [ ] Add failing tests:

```rust
#[test]
fn oracle_named_line_lookup_counts_positive_occurrences_from_start() {
    let lines = named_columns(3, vec![vec!["a"], vec!["b", "a"], vec!["a"], vec!["b"]]);
    let report = support::oracle::grid::resolve_named_line(&lines, "a", 2).unwrap();

    assert_eq!(report.resolved_line, 2);
    assert_eq!(report.explicit_matches, vec![1, 2, 3]);
    assert!(report.implicit_lines_assumed_named.is_empty());
}

#[test]
fn oracle_named_line_lookup_counts_negative_occurrences_from_end() {
    let lines = named_columns(3, vec![vec!["a"], vec!["b", "a"], vec!["a"], vec!["b"]]);
    let report = support::oracle::grid::resolve_named_line(&lines, "a", -1).unwrap();

    assert_eq!(report.resolved_line, 3);
    assert_eq!(report.explicit_matches, vec![1, 2, 3]);
}

#[test]
fn oracle_named_line_lookup_extends_after_for_missing_positive_occurrence() {
    let lines = named_columns(2, vec![vec!["a"], vec![], vec!["a"]]);
    let report = support::oracle::grid::resolve_named_line(&lines, "a", 4).unwrap();

    assert_eq!(report.resolved_line, 5);
    assert_eq!(report.explicit_matches, vec![1, 3]);
    assert_eq!(report.implicit_lines_assumed_named, vec![4, 5]);
}

#[test]
fn oracle_named_line_lookup_extends_before_for_missing_negative_occurrence() {
    let lines = named_columns(2, vec![vec!["a"], vec![], vec!["a"]]);
    let report = support::oracle::grid::resolve_named_line(&lines, "a", -3).unwrap();

    assert_eq!(report.resolved_line, 0);
    assert_eq!(report.explicit_matches, vec![1, 3]);
    assert_eq!(report.implicit_lines_assumed_named, vec![0]);
}
```

- [ ] Implement `resolve_named_line(lines, name, occurrence)`.
- [ ] Reject `occurrence == 0` as `NamedGridError::ZeroLine`.
- [ ] Positive overflow uses implicit lines after the explicit end line.
- [ ] Positive overflow must count each implicit line after the explicit end as a distinct named occurrence after all explicit matches have been counted.
- [ ] Negative overflow uses implicit lines before explicit line `1`, so line `0`, `-1`, and lower may be valid intermediate named-resolution results.
- [ ] Negative overflow must count each implicit line before explicit line `1` as a distinct named occurrence after all explicit reverse matches have been counted.
- [ ] Do not pass line `0` or negative lines into existing `AxisPlacement::new` until paired placement resolution has checked whether the final area is valid for the scenario being asserted.
- [ ] Run:

```bash
cargo test -p surgeist --test oracle oracle_named_line_lookup
```

- [ ] Commit:

```bash
git add crates/surgeist/tests/support/oracle/grid/named.rs crates/surgeist/tests/oracle.rs
git commit -m "Resolve named grid line occurrences"
```

---

## Task 3: Resolve Numeric Negative Lines Against Explicit Grid

**Files:**
- Modify `crates/surgeist/tests/support/oracle/grid/named.rs`
- Modify `crates/surgeist/tests/oracle.rs`

- [ ] Add failing tests:

```rust
#[test]
fn oracle_named_numeric_negative_line_counts_from_explicit_end() {
    let lines = support::oracle::grid::NamedGridLines::empty(
        support::oracle::grid::GridAxis::Column,
        4,
    );

    assert_eq!(support::oracle::grid::resolve_numeric_line(&lines, -1).unwrap(), 5);
    assert_eq!(support::oracle::grid::resolve_numeric_line(&lines, -2).unwrap(), 4);
}

#[test]
fn oracle_named_numeric_zero_line_is_invalid() {
    let lines = support::oracle::grid::NamedGridLines::empty(
        support::oracle::grid::GridAxis::Column,
        4,
    );

    assert_eq!(
        support::oracle::grid::resolve_numeric_line(&lines, 0).unwrap_err(),
        support::oracle::grid::NamedGridError::ZeroLine,
    );
}
```

- [ ] Implement `resolve_numeric_line(lines, raw_line)`.
- [ ] Positive numbers pass through.
- [ ] Negative numbers resolve as `explicit_track_count + 2 + raw_line`, so `-1` maps to the explicit end line.
- [ ] Keep underflow visible as an integer result; placement validation decides whether a final area before line 1 is acceptable for that test.
- [ ] Run:

```bash
cargo test -p surgeist --test oracle oracle_named_numeric
```

- [ ] Commit:

```bash
git add crates/surgeist/tests/support/oracle/grid/named.rs crates/surgeist/tests/oracle.rs
git commit -m "Resolve negative grid lines in oracle"
```

---

## Task 4: Resolve Named Spans

**Files:**
- Modify `crates/surgeist/tests/support/oracle/grid/named.rs`
- Modify `crates/surgeist/tests/oracle.rs`

- [ ] Add failing tests:

```rust
#[test]
fn oracle_named_span_from_start_finds_nth_named_line_forward() {
    let lines = named_columns(5, vec![vec!["a"], vec![], vec!["a"], vec![], vec!["a"], vec![]]);

    let report = support::oracle::grid::resolve_named_span_from_start(&lines, 1, "a", 2).unwrap();

    assert_eq!(report.resolved_line, 5);
}

#[test]
fn oracle_named_span_from_end_finds_nth_named_line_backward() {
    let lines = named_columns(5, vec![vec!["a"], vec![], vec!["a"], vec![], vec!["a"], vec![]]);

    let report = support::oracle::grid::resolve_named_span_from_end(&lines, 5, "a", 2).unwrap();

    assert_eq!(report.resolved_line, 1);
}

#[test]
fn oracle_named_span_extends_implicitly_when_name_is_missing() {
    let lines = named_columns(2, vec![vec!["a"], vec![], vec!["a"]]);

    let report =
        support::oracle::grid::resolve_named_span_from_start(&lines, 3, "a", 2).unwrap();

    assert_eq!(report.resolved_line, 5);
    assert_eq!(report.implicit_lines_assumed_named, vec![4, 5]);
}
```

- [ ] Implement `resolve_named_span_from_start(lines, start_line, name, count)`.
- [ ] Implement `resolve_named_span_from_end(lines, end_line, name, count)`.
- [ ] Treat `span name` as `count == 1`.
- [ ] Do not count the opposite edge line itself when searching for named span endpoints.
- [ ] Reject `count == 0` as `NamedGridError::ZeroSpan`.
- [ ] Add tests for anonymous spans (`span 3`) to prove they keep using direct numeric offsets.
- [ ] Run:

```bash
cargo test -p surgeist --test oracle oracle_named_span
```

- [ ] Commit:

```bash
git add crates/surgeist/tests/support/oracle/grid/named.rs crates/surgeist/tests/oracle.rs
git commit -m "Resolve named grid spans in oracle"
```

---

## Task 5: Compose Named Axis Placement

**Files:**
- Modify `crates/surgeist/tests/support/oracle/grid/named.rs`
- Modify `crates/surgeist/tests/support/oracle/grid/placement.rs`
- Modify `crates/surgeist/tests/oracle.rs`

- [ ] Add failing tests:

```rust
#[test]
fn oracle_named_axis_resolves_named_start_and_named_end() {
    let lines = named_columns(4, vec![vec!["a"], vec![], vec!["b"], vec![], vec!["b"]]);
    let report = support::oracle::grid::resolve_named_axis_placement(
        &lines,
        support::oracle::grid::NamedAxisPlacement {
            start: support::oracle::grid::NamedGridLine::Named {
                name: "a".to_owned(),
                occurrence: 1,
            },
            end: support::oracle::grid::NamedGridLine::Named {
                name: "b".to_owned(),
                occurrence: 2,
            },
        },
        None,
    )
    .unwrap();

    assert_eq!(report.resolved.start_line, 1);
    assert_eq!(report.resolved.end_line, 5);
    assert_eq!(report.resolved.span, 4);
}

#[test]
fn oracle_named_axis_resolves_line_to_named_span() {
    let lines = named_columns(4, vec![vec!["a"], vec![], vec!["a"], vec![], vec![]]);
    let report = support::oracle::grid::resolve_named_axis_placement(
        &lines,
        support::oracle::grid::NamedAxisPlacement {
            start: support::oracle::grid::NamedGridLine::Number(1),
            end: support::oracle::grid::NamedGridLine::Span {
                name: Some("a".to_owned()),
                count: 2,
            },
        },
        None,
    )
    .unwrap();

    assert_eq!(report.resolved.start_line, 1);
    assert_eq!(report.resolved.end_line, 5);
}

#[test]
fn oracle_named_axis_drops_end_span_when_both_sides_are_spans() {
    let lines = support::oracle::grid::NamedGridLines::empty(
        support::oracle::grid::GridAxis::Column,
        3,
    );
    let report = support::oracle::grid::resolve_named_axis_placement(
        &lines,
        support::oracle::grid::NamedAxisPlacement {
            start: support::oracle::grid::NamedGridLine::Span {
                name: None,
                count: 1,
            },
            end: support::oracle::grid::NamedGridLine::Span {
                name: None,
                count: 1,
            },
        },
        Some(2),
    )
    .unwrap();

    assert_eq!(
        report.conflict_resolution,
        Some(support::oracle::grid::NamedPlacementConflictResolution::DroppedEndSpan)
    );
    assert_eq!(report.resolved.start_line, 2);
    assert_eq!(report.resolved.end_line, 3);
}

#[test]
fn oracle_named_axis_swaps_reversed_resolved_lines() {
    let lines = support::oracle::grid::NamedGridLines::empty(
        support::oracle::grid::GridAxis::Column,
        4,
    );
    let report = support::oracle::grid::resolve_named_axis_placement(
        &lines,
        support::oracle::grid::NamedAxisPlacement {
            start: support::oracle::grid::NamedGridLine::Number(4),
            end: support::oracle::grid::NamedGridLine::Number(2),
        },
        None,
    )
    .unwrap();

    assert_eq!(
        report.conflict_resolution,
        Some(support::oracle::grid::NamedPlacementConflictResolution::SwappedResolvedLines)
    );
    assert_eq!(report.resolved.start_line, 2);
    assert_eq!(report.resolved.end_line, 4);
}

#[test]
fn oracle_named_axis_drops_equal_end_line_to_span_one() {
    let lines = support::oracle::grid::NamedGridLines::empty(
        support::oracle::grid::GridAxis::Column,
        4,
    );
    let report = support::oracle::grid::resolve_named_axis_placement(
        &lines,
        support::oracle::grid::NamedAxisPlacement {
            start: support::oracle::grid::NamedGridLine::Number(3),
            end: support::oracle::grid::NamedGridLine::Number(3),
        },
        None,
    )
    .unwrap();

    assert_eq!(
        report.conflict_resolution,
        Some(support::oracle::grid::NamedPlacementConflictResolution::DroppedEqualEndLine)
    );
    assert_eq!(report.resolved.start_line, 3);
    assert_eq!(report.resolved.end_line, 4);
}

#[test]
fn oracle_named_axis_defaults_lone_start_named_span_to_one() {
    let lines = named_columns(3, vec![vec!["a"], vec![], vec!["a"], vec![]]);
    let report = support::oracle::grid::resolve_named_axis_placement(
        &lines,
        support::oracle::grid::NamedAxisPlacement {
            start: support::oracle::grid::NamedGridLine::Span {
                name: Some("a".to_owned()),
                count: 4,
            },
            end: support::oracle::grid::NamedGridLine::Auto,
        },
        Some(2),
    )
    .unwrap();

    assert_eq!(
        report.conflict_resolution,
        Some(support::oracle::grid::NamedPlacementConflictResolution::DefaultedLoneNamedSpanToOne)
    );
    assert_eq!(report.resolved.start_line, 2);
    assert_eq!(report.resolved.end_line, 3);
}

#[test]
fn oracle_named_axis_defaults_lone_end_named_span_to_one() {
    let lines = named_columns(3, vec![vec!["a"], vec![], vec!["a"], vec![]]);
    let report = support::oracle::grid::resolve_named_axis_placement(
        &lines,
        support::oracle::grid::NamedAxisPlacement {
            start: support::oracle::grid::NamedGridLine::Auto,
            end: support::oracle::grid::NamedGridLine::Span {
                name: Some("a".to_owned()),
                count: 4,
            },
        },
        Some(2),
    )
    .unwrap();

    assert_eq!(
        report.conflict_resolution,
        Some(support::oracle::grid::NamedPlacementConflictResolution::DefaultedLoneNamedSpanToOne)
    );
    assert_eq!(report.resolved.start_line, 2);
    assert_eq!(report.resolved.end_line, 3);
}

#[test]
fn oracle_named_axis_bare_ident_prefers_side_generated_line_name() {
    let lines = named_columns(
        3,
        vec![
            vec!["main-start"],
            vec![],
            vec![],
            vec!["main-end"],
        ],
    );
    let report = support::oracle::grid::resolve_named_axis_placement(
        &lines,
        support::oracle::grid::NamedAxisPlacement {
            start: support::oracle::grid::NamedGridLine::BareIdent("main".to_owned()),
            end: support::oracle::grid::NamedGridLine::BareIdent("main".to_owned()),
        },
        None,
    )
    .unwrap();

    assert_eq!(report.resolved.start_line, 1);
    assert_eq!(report.resolved.end_line, 4);
}
```

- [ ] Implement `resolve_named_axis_placement(lines, placement, auto_cursor_line)`.
- [ ] Support these combinations:
  - `Auto / Auto` with cursor.
  - `Number / Number`.
  - `Named / Named`.
  - `Number / Span`.
  - `Named / Span`.
  - `Span / Number`.
  - `Span / Named`.
  - `BareIdent / BareIdent` with side-aware `-start` and `-end` preference.
  - One side `Auto` plus a definite opposite side.
- [ ] Normalize `Span / Span` by dropping the end-side span and record `DroppedEndSpan`.
- [ ] Normalize a lone named span paired with `auto` by replacing the named span count with `span 1` and record `DefaultedLoneNamedSpanToOne`.
- [ ] Normalize resolved start-after-end by swapping resolved lines and record `SwappedResolvedLines`.
- [ ] Normalize equal resolved start/end by dropping the end side to default `span 1` and record `DroppedEqualEndLine`.
- [ ] Reject unresolved `Auto` without `auto_cursor_line`.
- [ ] Add `AxisPlacement::try_new(start, end)` in `placement.rs` as a public wrapper around the existing validation, without changing numeric placement behavior.
- [ ] Compose through `AxisPlacement::try_new` after both line integers are known.
- [ ] Map `PlacementError::LineBeforeFirst` to `NamedGridError::LineBeforeFirst { axis, start_line, end_line }`.
- [ ] Add conversion helpers from named reports to `LinePlacement` only where the existing positive numeric oracle needs to be reused.
- [ ] Run:

```bash
cargo test -p surgeist --test oracle oracle_named_axis
```

- [ ] Commit:

```bash
git add crates/surgeist/tests/support/oracle/grid/named.rs crates/surgeist/tests/support/oracle/grid/placement.rs crates/surgeist/tests/oracle.rs
git commit -m "Compose named grid axis placement"
```

---

## Task 6: Expand Fixed Repeats With Line Names

**Files:**
- Modify `crates/surgeist/tests/support/oracle/grid/named.rs`
- Modify `crates/surgeist/tests/oracle.rs`

- [ ] Add failing tests:

```rust
#[test]
fn oracle_named_fixed_repeat_expands_line_names_between_tracks() {
    let expanded = support::oracle::grid::expand_named_fixed_repeat(
        support::oracle::grid::GridAxis::Column,
        2,
        [
            support::oracle::grid::NamedTrackComponent::LineNames(vec!["a".to_owned()]),
            support::oracle::grid::NamedTrackComponent::Track,
            support::oracle::grid::NamedTrackComponent::LineNames(vec!["b".to_owned()]),
            support::oracle::grid::NamedTrackComponent::Track,
            support::oracle::grid::NamedTrackComponent::LineNames(vec!["c".to_owned()]),
        ],
    )
    .unwrap();

    assert_eq!(expanded.explicit_track_count, 4);
    assert_eq!(expanded.named_occurrences("a"), vec![1, 3]);
    assert_eq!(expanded.named_occurrences("b"), vec![2, 4]);
    assert_eq!(expanded.named_occurrences("c"), vec![3, 5]);
}
```

- [ ] Add `NamedTrackComponent::{LineNames(Vec<String>), Track}`.
- [ ] Implement fixed repeat expansion for oracle inputs.
- [ ] Preserve adjacent line-name lists by merging names onto the same expanded line in source order.
- [ ] Reject zero repeat count with `NamedGridError::ZeroRepeat`.
- [ ] Explicitly document that auto-repeat expansion must be supplied as already-expanded facts by tests because its count depends on available space and track sizing.
- [ ] Run:

```bash
cargo test -p surgeist --test oracle oracle_named_fixed_repeat
```

- [ ] Commit:

```bash
git add crates/surgeist/tests/support/oracle/grid/named.rs crates/surgeist/tests/oracle.rs
git commit -m "Expand named fixed repeats in oracle"
```

---

## Task 7: Generate Template-Area Line Names

**Files:**
- Modify `crates/surgeist/tests/support/oracle/grid/named.rs`
- Modify `crates/surgeist/tests/oracle.rs`

- [ ] Add failing tests:

```rust
#[test]
fn oracle_template_areas_generate_row_and_column_line_names() {
    let areas = support::oracle::grid::TemplateAreas::new([
        vec!["head", "head"],
        vec!["nav", "main"],
        vec!["nav", "main"],
    ])
    .unwrap();

    let columns = support::oracle::grid::area_generated_lines(
        support::oracle::grid::GridAxis::Column,
        &areas,
        support::oracle::grid::NamedGridLines::empty(
            support::oracle::grid::GridAxis::Column,
            2,
        ),
    )
    .unwrap();
    let rows = support::oracle::grid::area_generated_lines(
        support::oracle::grid::GridAxis::Row,
        &areas,
        support::oracle::grid::NamedGridLines::empty(support::oracle::grid::GridAxis::Row, 3),
    )
    .unwrap();

    assert_eq!(columns.line_names(1), vec!["head-start", "nav-start"]);
    assert_eq!(columns.line_names(2), vec!["nav-end", "main-start"]);
    assert_eq!(columns.line_names(3), vec!["head-end", "main-end"]);
    assert_eq!(rows.line_names(1), vec!["head-start"]);
    assert_eq!(rows.line_names(2), vec!["head-end", "nav-start", "main-start"]);
    assert_eq!(rows.line_names(4), vec!["nav-end", "main-end"]);
}

#[test]
fn oracle_template_areas_reject_non_rectangular_area() {
    let err = support::oracle::grid::TemplateAreas::new([
        vec!["a", "a"],
        vec!["a", "b"],
    ])
    .unwrap_err();

    assert_eq!(
        err,
        support::oracle::grid::NamedGridError::AreaNotRectangular {
            area: "a".to_owned(),
        }
    );
}

#[test]
fn oracle_template_areas_reject_empty_matrix() {
    assert_eq!(
        support::oracle::grid::TemplateAreas::new(Vec::<Vec<&str>>::new()).unwrap_err(),
        support::oracle::grid::NamedGridError::EmptyTemplateAreas,
    );
}

#[test]
fn oracle_template_areas_reject_mismatched_row_lengths() {
    let err = support::oracle::grid::TemplateAreas::new([
        vec!["a", "a"],
        vec!["a"],
    ])
    .unwrap_err();

    assert_eq!(
        err,
        support::oracle::grid::NamedGridError::TemplateAreaRowLengthMismatch {
            expected: 2,
            actual: 1,
            row: 2,
        }
    );
}

#[test]
fn oracle_template_areas_treat_dot_runs_as_null_cells() {
    let areas = support::oracle::grid::TemplateAreas::new([
        vec!["....", "main"],
    ])
    .unwrap();

    assert!(!areas.contains_area("...."));
    assert!(areas.contains_area("main"));
}

#[test]
fn oracle_template_areas_expand_base_line_map_to_template_size() {
    let areas = support::oracle::grid::TemplateAreas::new([
        vec!["a", "a", "a"],
    ])
    .unwrap();
    let columns = support::oracle::grid::area_generated_lines(
        support::oracle::grid::GridAxis::Column,
        &areas,
        support::oracle::grid::NamedGridLines::empty(
            support::oracle::grid::GridAxis::Column,
            1,
        ),
    )
    .unwrap();

    assert_eq!(columns.explicit_track_count, 3);
    assert_eq!(columns.line_names(1), vec!["a-start"]);
    assert_eq!(columns.line_names(4), vec!["a-end"]);
}

#[test]
fn oracle_template_areas_preserve_larger_base_line_map() {
    let areas = support::oracle::grid::TemplateAreas::new([
        vec!["a"],
    ])
    .unwrap();
    let columns = support::oracle::grid::area_generated_lines(
        support::oracle::grid::GridAxis::Column,
        &areas,
        support::oracle::grid::NamedGridLines::empty(
            support::oracle::grid::GridAxis::Column,
            3,
        ),
    )
    .unwrap();

    assert_eq!(columns.explicit_track_count, 3);
    assert_eq!(columns.line_names(1), vec!["a-start"]);
    assert_eq!(columns.line_names(2), vec!["a-end"]);
}
```

- [ ] Add `TemplateAreas` with explicit row/column count and area rectangles.
- [ ] Treat any token made entirely of one or more `.` characters as empty.
- [ ] Reject an empty template matrix as `NamedGridError::EmptyTemplateAreas`.
- [ ] Reject zero-width rows and mismatched row widths as `NamedGridError::TemplateAreaRowLengthMismatch`.
- [ ] Validate rectangular areas.
- [ ] Generate `area-start` and `area-end` names on both axes at rectangle boundaries.
- [ ] Merge generated names with existing `NamedGridLines` without dropping explicit names.
- [ ] Expand the base line map to `max(base.explicit_track_count, areas.axis_track_count)` before inserting generated names.
- [ ] Add `area_generated_facts(areas, base_columns, base_rows) -> AreaGeneratedFacts` so callers that need subgrid clipping can keep the source rectangles with generated line maps.
- [ ] Store generated names with `LineNameOrigin::AreaGenerated`.
- [ ] Preserve deterministic assertion order: existing explicit names first, generated names next in row-major first-discovery order, and local subgrid names last when subgrid merging later composes these facts.
- [ ] Add `resolve_named_area(areas, area_name)` that returns a row and column `NamedAxisPlacement` using generated `-start` and `-end` names.
- [ ] Run:

```bash
cargo test -p surgeist --test oracle oracle_template_areas
```

- [ ] Commit:

```bash
git add crates/surgeist/tests/support/oracle/grid/named.rs crates/surgeist/tests/oracle.rs
git commit -m "Generate named grid area lines in oracle"
```

---

## Task 8: Model Subgrid Line Name Repeat, Inheritance, And Merging

**Files:**
- Modify `crates/surgeist/tests/support/oracle/grid/named.rs`
- Modify `crates/surgeist/tests/oracle.rs`

- [ ] Add failing tests:

```rust
#[test]
fn oracle_subgrid_name_repeat_expands_to_used_span() {
    let expanded = support::oracle::grid::expand_subgrid_name_list(
        support::oracle::grid::GridAxis::Column,
        4,
        vec![
            support::oracle::grid::SubgridNameComponent::LineNames(vec!["a".to_owned()]),
            support::oracle::grid::SubgridNameComponent::Repeat {
                count: support::oracle::grid::SubgridNameRepeatCount::Number(2),
                line_name_sets: vec![vec!["b".to_owned()]],
            },
            support::oracle::grid::SubgridNameComponent::LineNames(vec!["c".to_owned()]),
        ],
    )
    .unwrap();

    assert_eq!(
        expanded.local_line_names,
        vec![
            vec!["a"],
            vec!["b"],
            vec!["b"],
            vec!["c"],
            vec![],
        ]
    );
}

#[test]
fn oracle_subgrid_auto_fill_name_repeat_pads_to_used_span() {
    let expanded = support::oracle::grid::expand_subgrid_name_list(
        support::oracle::grid::GridAxis::Column,
        3,
        vec![support::oracle::grid::SubgridNameComponent::Repeat {
            count: support::oracle::grid::SubgridNameRepeatCount::AutoFill,
            line_name_sets: vec![vec!["b".to_owned()]],
        }],
    )
    .unwrap();

    assert_eq!(
        expanded.local_line_names,
        vec![vec!["b"], vec!["b"], vec!["b"], vec!["b"]]
    );
}

#[test]
fn oracle_subgrid_auto_fill_name_repeat_reserves_trailing_fixed_names() {
    let expanded = support::oracle::grid::expand_subgrid_name_list(
        support::oracle::grid::GridAxis::Column,
        4,
        vec![
            support::oracle::grid::SubgridNameComponent::LineNames(vec!["a".to_owned()]),
            support::oracle::grid::SubgridNameComponent::LineNames(vec!["a".to_owned()]),
            support::oracle::grid::SubgridNameComponent::LineNames(vec!["a".to_owned()]),
            support::oracle::grid::SubgridNameComponent::LineNames(vec!["a".to_owned()]),
            support::oracle::grid::SubgridNameComponent::Repeat {
                count: support::oracle::grid::SubgridNameRepeatCount::AutoFill,
                line_name_sets: vec![vec!["b".to_owned()]],
            },
            support::oracle::grid::SubgridNameComponent::LineNames(vec!["c".to_owned()]),
        ],
    )
    .unwrap();

    assert_eq!(
        expanded.local_line_names,
        vec![vec!["a"], vec!["a"], vec!["a"], vec!["a"], vec!["c"]]
    );
}

#[test]
fn oracle_subgrid_line_names_merge_parent_and_local_names() {
    let parent = named_columns(4, vec![vec!["a"], vec!["b"], vec![], vec!["c"], vec!["d"]]);
    let report = support::oracle::grid::inherit_named_subgrid_lines(
        &parent,
        support::oracle::grid::TrackSpan::new(2, 5),
        false,
        vec![
            vec!["local-start".to_owned()],
            vec![],
            vec!["middle".to_owned()],
            vec!["local-end".to_owned()],
        ],
        None,
    )
    .unwrap();

    assert_eq!(report.lines.line_names(1), vec!["b", "local-start"]);
    assert_eq!(report.lines.line_names(3), vec!["c", "middle"]);
    assert_eq!(report.lines.line_names(4), vec!["d", "local-end"]);
}

#[test]
fn oracle_subgrid_line_names_reverse_parent_line_order_when_axis_is_reversed() {
    let parent = named_columns(4, vec![vec!["a"], vec!["b"], vec![], vec!["c"], vec!["d"]]);
    let report = support::oracle::grid::inherit_named_subgrid_lines(
        &parent,
        support::oracle::grid::TrackSpan::new(2, 5),
        true,
        vec![vec![], vec![], vec![], vec![]],
        None,
    )
    .unwrap();

    assert_eq!(report.lines.line_names(1), vec!["d"]);
    assert_eq!(report.lines.line_names(2), vec!["c"]);
    assert_eq!(report.lines.line_names(4), vec!["b"]);
}

#[test]
fn oracle_subgrid_recomputes_area_generated_names_from_clipped_parent_areas() {
    let parent_areas = support::oracle::grid::TemplateAreas::new([
        vec!["a", "a", "a", "a"],
    ])
    .unwrap();
    let parent_facts = support::oracle::grid::area_generated_facts(
        &parent_areas,
        support::oracle::grid::NamedGridLines::empty(
            support::oracle::grid::GridAxis::Column,
            4,
        ),
        support::oracle::grid::NamedGridLines::empty(
            support::oracle::grid::GridAxis::Row,
            1,
        ),
    )
    .unwrap();
    let parent = parent_facts.columns.clone();

    let report = support::oracle::grid::inherit_named_subgrid_lines(
        &parent,
        support::oracle::grid::TrackSpan::new(2, 4),
        false,
        vec![vec![], vec![], vec![]],
        Some(&parent_facts),
    )
    .unwrap();

    assert_eq!(
        report.clipped_area_sources["a"].parent_span,
        support::oracle::grid::TrackSpan::new(2, 4)
    );
    assert_eq!(report.lines.line_names(1), vec!["a-start"]);
    assert_eq!(report.lines.line_names(3), vec!["a-end"]);
}

#[test]
fn oracle_subgrid_named_placement_clamps_to_subgrid_explicit_lines() {
    let subgrid = named_columns(2, vec![vec!["a"], vec![], vec!["a"]]);
    let report = support::oracle::grid::resolve_named_subgrid_axis_placement(
        &subgrid,
        support::oracle::grid::NamedAxisPlacement {
            start: support::oracle::grid::NamedGridLine::Named {
                name: "a".to_owned(),
                occurrence: -3,
            },
            end: support::oracle::grid::NamedGridLine::Named {
                name: "a".to_owned(),
                occurrence: 4,
            },
        },
        None,
    )
    .unwrap();

    assert_eq!(report.unclamped_start_line, 0);
    assert_eq!(report.unclamped_end_line, 5);
    assert_eq!(report.clamped.resolved.start_line, 1);
    assert_eq!(report.clamped.resolved.end_line, 3);
}
```

- [ ] Implement `SubgridNameComponent::{LineNames(Vec<String>), Repeat { count, line_name_sets }}`.
- [ ] Implement `SubgridNameRepeatCount::{Number(usize), AutoFill}`.
- [ ] Implement `expand_subgrid_name_list(axis, used_track_count, components)`.
- [ ] Fixed subgrid name repeats expand each `line_name_sets` entry in source order and truncate/pad to exactly `used_track_count + 1` line-name slots.
- [ ] `AutoFill` name repeats fill only the remaining line-name slots after reserving all non-auto components before and after the auto-repeat.
- [ ] Store expanded local subgrid names with `LineNameOrigin::LocalSubgrid` when merged.
- [ ] Implement `SubgridLineNameInheritanceReport`.
- [ ] Validate that local subgrid line names count equals inherited span length plus one.
- [ ] Copy only parent names with `LineNameOrigin::Explicit` over the parent span.
- [ ] Change `inherit_named_subgrid_lines` to accept optional `AreaGeneratedFacts` so area-generated names can be recomputed from parent area rectangles clipped to the subgrid span, then mapped into local subgrid line coordinates.
- [ ] Include clipped area-source rectangles in `SubgridLineNameInheritanceReport`.
- [ ] Reverse copied parent line order when `reversed` is true.
- [ ] Merge local line names after inherited names on each local line.
- [ ] Add `resolve_named_subgrid_axis_placement(lines, placement, auto_cursor_line)` that first resolves with the normal named placement rules, then clamps final start/end to `1..=explicit_track_count + 1` and reports the unclamped lines.
- [ ] Leave `OracleGridError::NamedLineInheritanceUnsupported` untouched unless the implementation proves it is unused after the named oracle lands.
- [ ] Run:

```bash
cargo test -p surgeist --test oracle oracle_subgrid_line_names
```

- [ ] Commit:

```bash
git add crates/surgeist/tests/support/oracle/grid/named.rs crates/surgeist/tests/oracle.rs
git commit -m "Model subgrid named line inheritance"
```

---

## Task 9: Parse Named Grid Positions In Parity Fixtures

**Files:**
- Modify `crates/surgeist/tests/layout_browser_parity/support.rs`

- [ ] Do not implement named production lowering in `crates/surgeist/src/style/adapters/layout.rs`, `crates/surgeist/src/layout/node_input.rs`, or `crates/surgeist/src/layout/grid/*`.
- [ ] Keep parser tests scoped to `parse_style_grid_line` and `to_declarations`; do not require `to_node_input` or `assert_surgeist_matches` to accept named placement yet.
- [ ] Add parser tests near existing parity parser tests:

```rust
#[test]
fn parse_grid_line_accepts_named_line() {
    assert_eq!(
        parse_style_grid_line("a").unwrap(),
        s::GridLine::NamedLine {
            name: "a".to_owned(),
            index: 1,
        }
    );
}

#[test]
fn parse_grid_line_accepts_named_line_with_occurrence() {
    assert_eq!(
        parse_style_grid_line("a 8").unwrap(),
        s::GridLine::NamedLine {
            name: "a".to_owned(),
            index: 8,
        }
    );
}

#[test]
fn parse_grid_line_accepts_integer_before_named_line() {
    assert_eq!(
        parse_style_grid_line("2 a").unwrap(),
        s::GridLine::NamedLine {
            name: "a".to_owned(),
            index: 2,
        }
    );
}

#[test]
fn parse_grid_line_accepts_negative_named_line_occurrence() {
    assert_eq!(
        parse_style_grid_line("b -1").unwrap(),
        s::GridLine::NamedLine {
            name: "b".to_owned(),
            index: -1,
        }
    );
}

#[test]
fn parse_grid_line_accepts_named_span() {
    assert_eq!(
        parse_style_grid_line("span a").unwrap(),
        s::GridLine::NamedSpan {
            name: "a".to_owned(),
            index: 1,
        }
    );
}

#[test]
fn parse_grid_line_accepts_named_span_with_count() {
    assert_eq!(
        parse_style_grid_line("span 2 a").unwrap(),
        s::GridLine::NamedSpan {
            name: "a".to_owned(),
            index: 2,
        }
    );
}

#[test]
fn parse_grid_line_accepts_named_span_with_reversed_count_order() {
    assert_eq!(
        parse_style_grid_line("span a 2").unwrap(),
        s::GridLine::NamedSpan {
            name: "a".to_owned(),
            index: 2,
        }
    );
}

#[test]
fn parse_grid_line_rejects_zero_named_line_occurrence() {
    assert!(parse_style_grid_line("a 0").is_err());
}

#[test]
fn parse_track_component_list_accepts_explicit_line_names() {
    let parsed = parse_style_track_component_list("[a] 10px [b c] 20px [d]").unwrap();

    assert_eq!(
        parsed,
        vec![
            s::GridTrackComponent::LineNames(vec!["a".to_owned()]),
            s::GridTrackComponent::Track(s::TrackSizing::px(10.0)),
            s::GridTrackComponent::LineNames(vec!["b".to_owned(), "c".to_owned()]),
            s::GridTrackComponent::Track(s::TrackSizing::px(20.0)),
            s::GridTrackComponent::LineNames(vec!["d".to_owned()]),
        ]
    );
}
```

- [ ] Implement token-based parsing in `parse_style_grid_line`.
- [ ] Accepted forms:
  - `auto`
  - `<integer>`
  - `<custom-ident>`
  - `<custom-ident> <integer>`
  - `<integer> <custom-ident>`
  - `span <integer>`
  - `span <custom-ident>`
  - `span <integer> <custom-ident>`
  - `span <custom-ident> <integer>`
- [ ] Reject `span 0`, named occurrence `0`, unknown extra tokens, and reserved idents `auto` and `span`.
- [ ] Do not rely on `s::GridLine::validate()` for zero named occurrences, because it currently rejects numeric line `0` and invalid names but not `NamedLine { index: 0 }`.
- [ ] Keep numeric-only parsing behavior unchanged for current fixtures.
- [ ] Add a required scoped parser subtask for explicit track line names in `grid-template-columns` and `grid-template-rows`, such as `[a] 10px [b]`.
- [ ] Add `parse_style_track_component_list` or equivalent test-only helper that parses track lists into style-level `s::GridTrackComponent` values.
- [ ] The track-line-name parser must produce `s::GridTrackComponent::LineNames` and style-level track components.
- [ ] Do not route explicit track line names through `layout::TrackComponent`, which cannot represent line names.
- [ ] Run:

```bash
cargo test -p surgeist --test layout_browser_parity parse_grid_line_accepts_named
cargo test -p surgeist --test layout_browser_parity parse_track_component_list_accepts_explicit_line_names
```

- [ ] Commit:

```bash
git add crates/surgeist/tests/layout_browser_parity/support.rs
git commit -m "Parse named grid lines in parity fixtures"
```

---

## Task 10: Add Composed Oracle Coverage

**Files:**
- Modify `crates/surgeist/tests/oracle.rs`
- Optional modify `crates/surgeist/tests/layout_oracle.rs` only if a composed oracle comparison can stay production-independent.

- [ ] Add pure composed tests:

```rust
#[test]
fn oracle_named_grid_resolves_area_generated_names_to_grid_area() {
    let areas = support::oracle::grid::TemplateAreas::new([
        vec!["head", "head"],
        vec!["nav", "main"],
    ])
    .unwrap();
    let columns = support::oracle::grid::area_generated_lines(
        support::oracle::grid::GridAxis::Column,
        &areas,
        support::oracle::grid::NamedGridLines::empty(
            support::oracle::grid::GridAxis::Column,
            2,
        ),
    )
    .unwrap();
    let rows = support::oracle::grid::area_generated_lines(
        support::oracle::grid::GridAxis::Row,
        &areas,
        support::oracle::grid::NamedGridLines::empty(support::oracle::grid::GridAxis::Row, 2),
    )
    .unwrap();

    let area = support::oracle::grid::resolve_named_grid_area(&columns, &rows, "main").unwrap();

    assert_eq!(
        area,
        support::oracle::grid::GridArea::new(2, 2, 1, 1)
    );
}

#[test]
fn oracle_axis_shorthand_repeats_omitted_custom_ident() {
    let expanded = support::oracle::grid::expand_axis_shorthand(
        support::oracle::grid::NamedGridLine::BareIdent("main".to_owned()),
        None,
    );

    assert_eq!(
        expanded,
        support::oracle::grid::NamedAxisPlacement {
            start: support::oracle::grid::NamedGridLine::BareIdent("main".to_owned()),
            end: support::oracle::grid::NamedGridLine::BareIdent("main".to_owned()),
        }
    );
}

#[test]
fn oracle_axis_shorthand_defaults_omitted_non_ident_to_auto() {
    let expanded = support::oracle::grid::expand_axis_shorthand(
        support::oracle::grid::NamedGridLine::Number(2),
        None,
    );

    assert_eq!(
        expanded,
        support::oracle::grid::NamedAxisPlacement {
            start: support::oracle::grid::NamedGridLine::Number(2),
            end: support::oracle::grid::NamedGridLine::Auto,
        }
    );
}

#[test]
fn oracle_grid_area_shorthand_repeats_single_custom_ident_to_all_sides() {
    let expanded = support::oracle::grid::expand_grid_area_shorthand(vec![
        support::oracle::grid::NamedGridLine::BareIdent("main".to_owned()),
    ])
    .unwrap();

    assert_eq!(expanded.row.start, support::oracle::grid::NamedGridLine::BareIdent("main".to_owned()));
    assert_eq!(expanded.row.end, support::oracle::grid::NamedGridLine::BareIdent("main".to_owned()));
    assert_eq!(expanded.column.start, support::oracle::grid::NamedGridLine::BareIdent("main".to_owned()));
    assert_eq!(expanded.column.end, support::oracle::grid::NamedGridLine::BareIdent("main".to_owned()));
}

#[test]
fn oracle_named_grid_resolves_subgrid_named_span_into_parent_space() {
    let parent = named_columns(4, vec![vec!["a"], vec!["b"], vec![], vec!["b"], vec!["c"]]);
    let subgrid = support::oracle::grid::inherit_named_subgrid_lines(
        &parent,
        support::oracle::grid::TrackSpan::new(2, 5),
        false,
        vec![vec![], vec![], vec![], vec![]],
        None,
    )
    .unwrap();

    let report = support::oracle::grid::resolve_named_axis_placement(
        &subgrid.lines,
        support::oracle::grid::NamedAxisPlacement {
            start: support::oracle::grid::NamedGridLine::Named {
                name: "b".to_owned(),
                occurrence: 1,
            },
            end: support::oracle::grid::NamedGridLine::Span {
                name: Some("c".to_owned()),
                count: 1,
            },
        },
        None,
    )
    .unwrap();

    assert_eq!(report.resolved.start_line, 1);
    assert_eq!(report.resolved.end_line, 4);
}
```

- [ ] Add helper `resolve_named_grid_area(columns, rows, area_name)`.
- [ ] Add `expand_axis_shorthand(first, second)` for `grid-column` and `grid-row` shorthand expansion.
- [ ] Add `expand_grid_area_shorthand(values)` for one-, two-, three-, and four-value `grid-area` shorthand expansion.
- [ ] Omitted custom-ident values repeat to the opposite side; omitted non-ident values default to `auto`.
- [ ] Add helper test constructors inside `oracle.rs`; do not introduce parser dependencies.
- [ ] Assert intermediate reports, not only final `GridArea`.
- [ ] Run:

```bash
cargo test -p surgeist --test oracle oracle_named_grid
```

- [ ] Commit:

```bash
git add crates/surgeist/tests/oracle.rs crates/surgeist/tests/support/oracle/grid/named.rs
git commit -m "Cover composed named grid oracle scenarios"
```

---

## Task 11: Classify Current Fixture Impact

**Files:**
- Modify `crates/surgeist/tests/layout_browser_parity.rs` only if the named parser exposes production-engine failures that were previously hidden by parse rejection.
- Do not modify generated fixture contents unless a checked-in fixture is malformed relative to browser output.

- [ ] Run the currently known named-grid fixture checks:

```bash
SURGEIST_PARITY_FILTER=subgrid_line_names_004_b_to_b_minus_1 cargo test -p surgeist --test layout_browser_parity runs_all_checked_in_browser_parity_xml -- --ignored
SURGEIST_PARITY_FILTER=subgrid_line_names_repeat_outer_span_a_to_a_8 cargo test -p surgeist --test layout_browser_parity runs_all_checked_in_browser_parity_xml -- --ignored
```

- [ ] Expected after parser work:
  - The tests should no longer fail with `invalid grid line 'b'` or `invalid grid span 'span a'`.
  - They may still fail because production layout lowering rejects `NamedLine` or `NamedSpan`; classify those failures as engine implementation gaps.

- [ ] If classification updates are necessary, edit only `crates/surgeist/tests/layout_browser_parity.rs`, likely `classified_error_kind`, to bucket `unsupported named grid line placement`.
- [ ] Add a short reason that distinguishes parser support from production layout support.
- [ ] Run:

```bash
cargo test -p surgeist --test layout_browser_parity parse_grid_line
```

- [ ] Commit any classification-only change:

```bash
git add crates/surgeist/tests/layout_browser_parity.rs
git commit -m "Classify named grid parity gaps"
```

---

## Task 12: Final Verification

- [ ] Run formatting and focused tests:

```bash
cargo fmt -p surgeist --check
cargo test -p surgeist --test oracle
cargo test -p surgeist --test layout_browser_parity parse_grid_line
git diff --check
```

- [ ] Run the full Surgeist test suite:

```bash
cargo test -p surgeist
```

- [ ] Inspect final status:

```bash
git status --short --branch
```

- [ ] Commit remaining cohesive changes:

```bash
git add crates/surgeist/tests/support/oracle/grid crates/surgeist/tests/oracle.rs crates/surgeist/tests/layout_browser_parity/support.rs crates/surgeist/tests/layout_browser_parity.rs
git commit -m "Complete named grid syntax oracle"
```

---

## Review Cycle Requirements

This plan is not complete until clean-context review has happened and accepted recommendations have been implemented.

- [ ] **Review Cycle 1: Spec correctness**
  - Ask a clean-context reviewer to compare this plan against CSS Grid Layout Level 2 named line, placement, `grid-template-areas`, and subgrid name rules.
  - Required reviewer prompt:

```text
Review docs/superpowers/plans/2026-06-17-surgeist-named-grid-syntax-oracle-implementation.md for CSS Grid named syntax correctness. Focus on named line occurrence indexing, negative occurrences, implicit named line fallback, named spans, generated area line names, fixed repeat line-name expansion, and subgrid line name inheritance/merging. Report missing spec rules, incorrect algorithms, and ambiguous test expectations. Do not edit files.
```

  - Implement accepted recommendations in the plan.
  - Commit with:

```bash
git add -f docs/superpowers/plans/2026-06-17-surgeist-named-grid-syntax-oracle-implementation.md
git commit -m "Tighten named grid oracle plan"
```

- [ ] **Review Cycle 2: Surgeist integration completeness**
  - Ask a second clean-context reviewer to compare this plan against the current Surgeist codebase and prior oracle plans.
  - Required reviewer prompt:

```text
Review docs/superpowers/plans/2026-06-17-surgeist-named-grid-syntax-oracle-implementation.md for implementation completeness in the current Surgeist repo. Focus on exact file paths, likely compile errors in proposed test snippets, module exports, error type ownership, fixture parser scope, and whether tasks accidentally implement production engine behavior. Report concrete plan edits only. Do not edit files.
```

  - Implement accepted recommendations in the plan.
  - Commit with:

```bash
git add -f docs/superpowers/plans/2026-06-17-surgeist-named-grid-syntax-oracle-implementation.md
git commit -m "Review named grid oracle plan"
```

- [ ] **Review Cycle 3: Final adversarial pass**
  - Ask a clean-context reviewer to find remaining gaps after the first two review cycles.
  - Required reviewer prompt:

```text
Do an adversarial final review of docs/superpowers/plans/2026-06-17-surgeist-named-grid-syntax-oracle-implementation.md. Assume a worker will follow it literally. Identify any remaining ambiguity, missing verification, non-compiling snippets, or spec cases that would keep the oracle from being spec-complete for named grid syntax. Do not edit files.
```

  - Implement accepted recommendations.
  - Run:

```bash
rg -n "[T]ODO|[T]BD|[e]ventually|[a]ppropriate|[s]imilar" docs/superpowers/plans/2026-06-17-surgeist-named-grid-syntax-oracle-implementation.md
git diff --check
```

  - Commit with:

```bash
git add -f docs/superpowers/plans/2026-06-17-surgeist-named-grid-syntax-oracle-implementation.md
git commit -m "Finalize named grid oracle plan"
```

---

## Completion Criteria

- The plan file exists at `docs/superpowers/plans/2026-06-17-surgeist-named-grid-syntax-oracle-implementation.md`.
- The plan covers all named grid syntax required by CSS Grid Level 2 for oracle purposes:
  - named lines,
  - repeated names,
  - numeric negative lines,
  - positive and negative named occurrence indexes,
  - named spans,
  - implicit fallback lines,
  - area-generated names,
  - fixed repeat expansion,
  - subgrid inherited and merged names,
  - parity fixture lowering.
- The plan separates oracle implementation from production engine implementation.
- At least three clean-context review cycles have run.
- Accepted review recommendations have been implemented.
- The final plan passes placeholder and whitespace checks.
- The plan has been committed on `main` with a short concrete commit message.
