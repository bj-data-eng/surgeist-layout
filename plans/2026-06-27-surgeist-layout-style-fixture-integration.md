# Surgeist Layout Style Fixture Integration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Update layout browser parity support to compile against the current intentional `surgeist-style` typed public API.

**Architecture:** Keep the existing single lowering path for browser parity fixtures: XML fixture attributes become `surgeist-style` declarations, style resolves them, and `surgeist-style::adapters::layout::LayoutLoweringSession` lowers resolved style into `surgeist-layout::NodeInput`. Do not add an extra conversion layer, do not bypass style validation, and do not weaken layout's semantic types.

**Tech Stack:** Rust 2024, `surgeist-layout`, dev-dependency `surgeist-style`, browser parity support in `tests/layout/browser_parity/support.rs`.

---

## Source Ledger

This plan resolves the open entries in:

- `/Users/codex/Development/surgeist-layout/plans/2026-06-27-surgeist-layout-generic-scalar-cross-crate-ledger.md`

Covered blockers:

- `surgeist-style` declaration insertion visibility blocks layout integration test compile.
- `surgeist-style` grid and calc value API changes block browser parity support compile.

## Non-Goals

- Do not edit `surgeist-style` from this repo.
- Do not make a layout-local style model.
- Do not duplicate `surgeist-style::adapters::layout` or add a second style-to-layout lowering path.
- Do not relax style validation to make fixtures pass.
- Do not update root `surgeist` submodule pointers from this repo.

## Task 1: Convert Declaration Construction To The Public Fallible API

**Files:**

- Modify: `tests/layout/browser_parity/support.rs`

- [x] **Step 1: Add a local insertion helper**

Add this helper near `to_declarations`:

```rust
fn insert_style_declaration(
    declarations: &mut s::Declarations,
    property: s::Property,
    value: s::Value,
) -> Result<(), Error> {
    declarations
        .try_insert(property, value)
        .map(|_| ())
        .map_err(|error| Error::new(error.to_string()))
}
```

This helper is not a new lowering layer. It is only the error-mapping boundary from style's public validation API into the browser parity test error type.

- [x] **Step 2: Replace direct declaration insertion**

In `to_declarations`, `insert_edges`, and `insert_edges_auto`, replace every call shaped like:

```rust
declarations.insert(
    s::Property::Display,
    s::Value::Display(to_style_display(display)),
);
```

with:

```rust
insert_style_declaration(
    &mut declarations,
    s::Property::Display,
    s::Value::Display(to_style_display(display)),
)?;
```

For helper functions that already receive `&mut s::Declarations`, use:

```rust
insert_style_declaration(declarations, property, s::Value::Edges(edges))?;
```

Run this check until it prints no matches:

```sh
rg -n "declarations\\.insert\\(" tests/layout/browser_parity/support.rs
```

Expected: no direct `Declarations::insert` calls remain in browser parity support.

- [x] **Step 3: Verify the focused failure moves forward**

Run:

```sh
cargo test -p surgeist-layout --test layout layout::contract::layout_scalar_supports_f32_and_f64 -- --nocapture
```

Expected: this no longer reports `method insert is private`. It may still fail on stale style value constructors until later tasks are complete.

## Task 2: Convert Track And Subgrid Name Construction To Style Constructors

**Files:**

- Modify: `tests/layout/browser_parity/support.rs`

- [x] **Step 1: Update layout track component to style component conversion**

Replace `to_style_track_component` with:

```rust
fn to_style_track_component(
    component: layout::TrackComponent,
) -> Result<s::GridTrackComponent, Error> {
    match component {
        layout::TrackComponent::Track(track) => {
            Ok(s::GridTrackComponent::Track(to_style_track_sizing(track)?))
        }
        layout::TrackComponent::Repeat(repeat) => {
            Ok(s::GridTrackComponent::Repeat(to_style_track_repeat(repeat)?))
        }
        layout::TrackComponent::LineNames(names) => s::GridTrackComponent::line_names(names)
            .map_err(|error| Error::new(error.to_string())),
        layout::TrackComponent::Subgrid(subgrid) => {
            let components = subgrid
                .name_components
                .into_iter()
                .map(to_style_subgrid_line_name_component)
                .collect::<Result<Vec<_>, _>>()?;
            let subgrid = s::SubgridTrack::from_components(components)
                .map_err(|error| Error::new(error.to_string()))?;
            Ok(s::GridTrackComponent::Subgrid(subgrid))
        }
    }
}
```

- [x] **Step 2: Update layout track repeat to style repeat conversion**

Replace `to_style_track_repeat` with:

```rust
fn to_style_track_repeat(repeat: layout::TrackRepetition) -> Result<s::TrackRepeat, Error> {
    let repeat_kind = repeat.repeat();
    let components = repeat
        .into_components()
        .into_iter()
        .map(to_style_track_component)
        .collect::<Result<Vec<_>, _>>()?;
    match repeat_kind {
        layout::TrackRepeat::Count(count) => s::TrackRepeat::count(
            count
                .get()
                .try_into()
                .map_err(|_| Error::new("repeat count does not fit style track repeat"))?,
            components,
        ),
        layout::TrackRepeat::AutoFill => s::TrackRepeat::auto_fill(components),
        layout::TrackRepeat::AutoFit => s::TrackRepeat::auto_fit(components),
    }
    .map_err(|error| Error::new(error.to_string()))
}
```

- [x] **Step 3: Update subgrid line-name component conversion**

Change `to_style_subgrid_line_name_component` to return `Result<s::SubgridLineNameComponent, Error>` and use style constructors:

```rust
fn to_style_subgrid_line_name_component(
    component: layout::SubgridLineNameComponent,
) -> Result<s::SubgridLineNameComponent, Error> {
    match component {
        layout::SubgridLineNameComponent::LineNames(names) => {
            s::SubgridLineNameComponent::line_names(names)
                .map_err(|error| Error::new(error.to_string()))
        }
        layout::SubgridLineNameComponent::Repeat {
            count,
            line_name_sets,
        } => {
            let count = match count {
                layout::SubgridLineNameRepeatCount::Count(count) => {
                    s::SubgridLineNameRepeatCount::count(count)
                        .map_err(|error| Error::new(error.to_string()))?
                }
                layout::SubgridLineNameRepeatCount::AutoFill => {
                    s::SubgridLineNameRepeatCount::AutoFill
                }
            };
            s::SubgridLineNameComponent::repeat(count, line_name_sets)
                .map_err(|error| Error::new(error.to_string()))
        }
    }
}
```

- [x] **Step 4: Update subgrid line-name parsing**

Change `parse_subgrid_line_names` to return `Result<s::GridTrackComponent, Error>` or add a helper:

```rust
fn parse_style_line_names_component(raw: &str) -> Result<s::GridTrackComponent, Error> {
    s::GridTrackComponent::line_names(parse_subgrid_line_names(raw)?)
        .map_err(|error| Error::new(error.to_string()))
}
```

Then replace `s::GridTrackComponent::LineNames(parse_subgrid_line_names(raw)?)` with:

```rust
parse_style_line_names_component(raw)?
```

- [x] **Step 5: Update parser-local repeat construction**

In `parse_style_track_component`, replace the repeat body:

```rust
let repeat = match count.trim() {
    "auto-fill" => s::TrackRepeat::auto_fill(components),
    "auto-fit" => s::TrackRepeat::auto_fit(components),
    raw => s::TrackRepeat::count(
        raw.parse()
            .map_err(|_| Error::new(format!("invalid repeat count `{raw}`")))?,
        components,
    ),
};
return Ok(s::GridTrackComponent::Repeat(repeat));
```

with:

```rust
let repeat = match count.trim() {
    "auto-fill" => s::TrackRepeat::auto_fill(components),
    "auto-fit" => s::TrackRepeat::auto_fit(components),
    raw => s::TrackRepeat::count(
        raw.parse()
            .map_err(|_| Error::new(format!("invalid repeat count `{raw}`")))?,
        components,
    ),
}
.map_err(|error| Error::new(error.to_string()))?;
return Ok(s::GridTrackComponent::Repeat(repeat));
```

- [x] **Step 6: Update line-name component test expectations**

Replace test expectations shaped like:

```rust
s::GridTrackComponent::LineNames(vec!["a".to_owned()])
```

with:

```rust
s::GridTrackComponent::line_names(["a"]).unwrap()
```

Replace multi-name expectations shaped like:

```rust
s::GridTrackComponent::LineNames(vec!["b".to_owned(), "c".to_owned()])
```

with:

```rust
s::GridTrackComponent::line_names(["b", "c"]).unwrap()
```

Run:

```sh
rg -n "GridTrackComponent::LineNames\\(vec!" tests/layout/browser_parity/support.rs
```

Expected: no matches.

- [x] **Step 7: Verify stale line-name and repeat payload errors are gone**

Run:

```sh
cargo test -p surgeist-layout --test layout layout::contract::layout_scalar_supports_f32_and_f64 -- --nocapture
```

Expected: no errors mention `expected GridLineNameSet, found Vec<String>`, `expected SubgridLineNameSets`, `expected SubgridLineNameRepeatCountValue`, or `expected TrackRepeat, found Result<TrackRepeat, Error>`.

## Task 3: Convert Grid Line Parsing And Tests To Style Constructors

**Files:**

- Modify: `tests/layout/browser_parity/support.rs`

- [x] **Step 1: Update `parse_style_grid_line`**

Replace direct enum construction in `parse_style_grid_line` with style constructors:

```rust
fn parse_style_grid_line(raw: &str) -> Result<s::GridLine, Error> {
    let tokens = split_top_level_whitespace(raw);
    match tokens.as_slice() {
        [token] if token == "auto" => Ok(s::GridLine::Auto),
        [token] if token == "span" => Err(Error::new("invalid grid span `span`")),
        [token] => match parse_style_line_index(token) {
            Ok(line) => s::GridLine::line(line).map_err(|error| Error::new(error.to_string())),
            Err(_) => s::GridLine::bare_ident(parse_custom_ident(token)?.to_owned())
                .map_err(|error| Error::new(error.to_string())),
        },
        [span, token] if span == "span" => {
            if let Ok(index) = parse_style_span_index(token) {
                return s::GridLine::span(index).map_err(|error| Error::new(error.to_string()));
            }
            s::GridLine::named_span(parse_custom_ident(token)?.to_owned(), 1)
                .map_err(|error| Error::new(error.to_string()))
        }
        [span, first, second] if span == "span" => {
            if let Ok(index) = parse_style_span_index(first) {
                return s::GridLine::named_span(parse_custom_ident(second)?.to_owned(), index)
                    .map_err(|error| Error::new(error.to_string()));
            }
            s::GridLine::named_span(
                parse_custom_ident(first)?.to_owned(),
                parse_style_span_index(second)?,
            )
            .map_err(|error| Error::new(error.to_string()))
        }
        [first, second] => {
            if let Ok(index) = parse_style_line_index(first) {
                return s::GridLine::named_line(parse_custom_ident(second)?.to_owned(), index)
                    .map_err(|error| Error::new(error.to_string()));
            }
            let index = parse_style_line_index(second)?;
            s::GridLine::named_line(parse_custom_ident(first)?.to_owned(), index)
                .map_err(|error| Error::new(error.to_string()))
        }
        _ => Err(Error::new(format!("invalid grid line `{raw}`"))),
    }
}
```

If the existing function contains additional validated cases, preserve the same accepted/rejected syntax while changing only construction.

- [x] **Step 2: Update tests to compare through constructors**

Replace expected values like:

```rust
s::GridLine::BareIdent("a".to_owned())
```

with:

```rust
s::GridLine::bare_ident("a").unwrap()
```

Replace expected struct literals like:

```rust
s::GridLine::NamedLine {
    name: "a".to_owned(),
    index: 8,
}
```

with:

```rust
s::GridLine::named_line("a", 8).unwrap()
```

Replace expected named span literals with:

```rust
s::GridLine::named_span("a", 2).unwrap()
```

- [x] **Step 3: Verify stale grid-line payload errors are gone**

Run:

```sh
cargo test -p surgeist-layout --test layout layout::contract::layout_scalar_supports_f32_and_f64 -- --nocapture
```

Expected: no errors mention `expected GridLineName`, `expected GridLineIndex`, or `expected GridSpanCount`.

## Task 4: Update Calc Sum Construction

**Files:**

- Modify: `tests/layout/browser_parity/support.rs`

- [x] **Step 1: Replace single-iterator calc sum call**

Replace:

```rust
Ok(s::CalcLength::sum([s::CalcLengthTerm::add(left), right]))
```

with:

```rust
Ok(s::CalcLength::sum(s::CalcLengthTerm::add(left), [right]))
```

- [x] **Step 2: Verify no stale calc calls remain**

Run:

```sh
rg -n "CalcLength::sum\\(\\[" tests/layout/browser_parity/support.rs
```

Expected: no matches.

## Task 5: Run Layout Verification

**Files:**

- Verify: `tests/layout/browser_parity/support.rs`
- Verify: layout source touched by the generic scalar branch

- [x] **Step 1: Run focused integration blocker check**

Run:

```sh
cargo test -p surgeist-layout --test layout layout::contract::layout_scalar_supports_f32_and_f64 -- --nocapture
```

Expected: PASS.

- [x] **Step 2: Run all layout contract tests**

Run:

```sh
cargo test -p surgeist-layout --test layout layout::contract -- --nocapture
```

Expected: PASS.

- [x] **Step 3: Run layout crate tests**

Run:

```sh
cargo test -p surgeist-layout
```

Expected: PASS.

- [x] **Step 4: Run clippy**

Run:

```sh
cargo clippy -p surgeist-layout --all-targets -- -D warnings
```

Expected: PASS.

- [x] **Step 5: Run formatting check**

Run:

```sh
cargo fmt --check
```

Expected: PASS.

## Task 6: Close The Cross-Crate Ledger Entries

**Files:**

- Modify: `plans/2026-06-27-surgeist-layout-generic-scalar-cross-crate-ledger.md`

- [x] **Step 1: Update the declaration insertion entry**

After Task 5 passes, change:

```markdown
- Status: `open`
```

under `surgeist-style declaration insertion visibility blocks layout integration test compile` to:

```markdown
- Status: `resolved`
```

Append this note to that entry:

```markdown
- Resolution: `surgeist-layout` browser parity support now constructs
  declarations through the public fallible `surgeist-style`
  `Declarations::try_insert` API, and the layout integration verification in
  `plans/2026-06-27-surgeist-layout-style-fixture-integration.md` passed.
```

- [x] **Step 2: Update the grid and calc API entry**

After Task 5 passes, change:

```markdown
- Status: `open`
```

under `surgeist-style grid and calc value API changes block browser parity support compile` to:

```markdown
- Status: `resolved`
```

Append this note to that entry:

```markdown
- Resolution: `surgeist-layout` browser parity support now uses the intentional
  public `surgeist-style` constructors for grid line names, subgrid line-name
  repeats, grid lines, track repeats, and non-empty calc sums, and the layout
  integration verification in
  `plans/2026-06-27-surgeist-layout-style-fixture-integration.md` passed.
```

- [x] **Step 3: Review the ledger diff**

Run:

```sh
git diff -- plans/2026-06-27-surgeist-layout-generic-scalar-cross-crate-ledger.md
```

Expected: only the two open entries covered by this plan move to `resolved`, and each resolution note names the verification plan that passed.

## Review Gate

Ask a clean reviewer to check:

- Browser parity support uses `surgeist-style` public constructors and `Declarations::try_insert`.
- No unchecked style constructors, compatibility aliases, or duplicate lowering modules were added.
- The only style-to-layout lowering remains `surgeist-style::adapters::layout`.
- All fixture parsing still preserves the previous accepted/rejected syntax.
- Focused layout integration and layout crate verification pass.
- The cross-crate blocker ledger entries covered by this plan are updated to `resolved` only after verification passes.

Completion for this plan is reviewer-clean layout integration plus passing layout verification.
