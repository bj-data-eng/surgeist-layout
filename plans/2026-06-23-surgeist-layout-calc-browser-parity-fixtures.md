# Surgeist Layout Calc Browser Parity Fixtures Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add crate-local browser parity coverage for layout calc values by preserving authored `calc(...)` expressions in generated XML, parsing them into style calc values for XML fixtures, lowering them into layout calc handles, and validating the resulting XML against browser-derived expectations.

**Architecture:** Keep production authored CSS parsing in `surgeist-css` and authored style value modeling in `surgeist-style`; this crate only extends the browser parity fixture format enough to test `surgeist-layout` calc behavior. The generator preserves browser-selected calc length values as typed XML strings, and the XML runner builds a per-fixture `LayoutCalcStore` so layout algorithms exercise the same calc resolver path used by crate-local unit tests.

**Tech Stack:** Rust in `tests/bin/surgeist-layout-generate/generator.rs` and `tests/layout/browser_parity/support.rs`, JavaScript helper code in `tests/layout/browser_parity/scripts/gentest/test_helper.js`, constrained HTML fixtures under `tests/layout/browser_parity/html`, generated XML under `tests/layout/browser_parity/xml`, focused verification with `cargo test -p surgeist-layout`, `cargo clippy -p surgeist-layout --all-targets -- -D warnings`, `cargo fmt --check`, `cargo run -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate`, `cargo run -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- check-corpus`, and `cargo test -p surgeist-layout --test layout layout::browser_parity::runs_all_checked_in_browser_parity_xml -- --ignored --nocapture`.

---

## Non-Negotiable Constraints

- This plan is crate-local to `/Users/codex/Development/surgeist-layout`.
- Do not edit `surgeist-css`, `surgeist-style`, the root `surgeist` repo, or sibling crate checkouts from this project.
- Do not implement general CSS parsing in this crate. The browser parity XML parser may parse a deliberately tiny fixture syntax because XML is a typed test format, not app-facing CSS.
- Do not hand-edit browser geometry in XML. Update HTML fixtures, generator/helper code, parser support, then regenerate XML.
- Do not add dependencies unless a worker first proves the standard library and existing test stack are insufficient.
- Do not add unsafe code.
- Code changes must go through worker and reviewer cycles per `AGENTS.md`.
- Commit at logical checkpoints only after review comes back clean.
- Do not commit intentionally red compile-failing code without explicit coordinator waiver.

## Current Baseline

- The regenerated generation report currently has `356` unsupported XML variants:
  - `240` unsupported `<br>` line-break semantics.
  - `100` unsupported mixed text/element content.
  - `16` unsupported missing `#test-root` fixture root.
- No currently skipped fixture is calc-shaped. There are no authored `calc(...)` HTML fixtures under `tests/layout/browser_parity/html` or the pinned Taffy `test_fixtures` cache.
- The XML parser currently rejects CSS-looking calc strings:

```rust
fn parse_length(raw: &str) -> Result<layout::Length, Error> {
    if let Some(px) = raw.strip_suffix("px") {
        return Ok(layout::Length::px(parse_number(px)?));
    }
    if let Some(percent) = raw.strip_suffix('%') {
        return Ok(layout::Length::percent(parse_number(percent)? / 100.0));
    }
    if let Ok(value) = parse_number(raw) {
        return Ok(layout::Length::px(value));
    }
    Err(Error::new(format!("unsupported length `{raw}`")))
}
```

- `support.rs` currently parses XML style attributes through `surgeist-style` declarations, resolves them, and calls `s::adapters::layout::lower(&resolved)`, which rejects calc-bearing style values because calc requires an accompanying `LayoutCalcStore`.
- `surgeist-style` already exposes `CalcLength`, `CalcLengthTerm`, and `Length::Calc`, plus `s::adapters::layout::lower_with_store(&resolved)`. The implementation must parse fixture calc strings into style calc ASTs, lower with a store, and merge that store into `TestTree` so the layout compute path can resolve the generated `CalcId` handles.
- Style calc support was reviewed and is real enough for this layout plan to rely on, though direct style coverage is currently narrow. Do not create a `surgeist-style` implementation plan unless layout implementation uncovers a concrete API gap.
- `test_helper.js` recently learned to preserve browser-selected percent margins via CSS Typed OM. Calc fixture preservation should use the same principle: trust the browser-selected value, not a hand-rolled cascade approximation.

## Boundary Notes For Style And CSS

This plan does not replace the synchronized root sequence for `surgeist-style` and `surgeist-css`.

- `surgeist-css` still owns parsing app-facing CSS `calc(...)` syntax.
- `surgeist-style` still owns authored style calc ASTs, property metadata, validation, and lowering from style values into layout `CalcId` handles.
- `surgeist-layout` owns only:
  - typed layout calc handles and resolver behavior,
  - browser parity fixture tooling,
  - fixture-local calc syntax for XML tests,
  - layout algorithm verification against generated browser expectations.

If a worker discovers a required style/css API gap while executing this plan, stop and write a precise upstream issue draft instead of editing sibling crates.

## Target XML Fixture Syntax

Use CSS-like strings in XML attributes because they are readable and already match authored fixture HTML:

```xml
<div width="calc(50% + 20px)" height="10px"/>
<div margin-left="calc(10% - 4px)" padding-right="calc(5% + 3px)"/>
<div grid-template-columns="calc(25% + 10px) 40px"/>
```

Initial supported fixture grammar:

```text
calc(<term> <op> <term>)
term := <number>px | <number>%
op := + | -
```

Examples accepted:

- `calc(50% + 20px)`
- `calc(20px + 50%)`
- `calc(50% - 8px)`
- `calc(-10% + 3px)`
- `calc(12px - 5%)`

Examples intentionally rejected in the first implementation:

- nested calc: `calc(100% - calc(2px + 1%))`
- multiplication/division: `calc(100% / 2)`
- non-length units: `calc(1em + 2px)`
- mixed keywords: `calc(max-content - 1px)`
- CSS custom properties: `calc(var(--x) + 1px)`

Rejected syntax must fail with a clear fixture parse error naming the unsupported calc expression.

## File Map

- Modify: `tests/layout/browser_parity/scripts/gentest/test_helper.js`
  - Preserve browser-selected calc values for dimensions, lengths, margins, padding, borders, gaps, insets, and grid track lengths where Chrome Typed OM exposes px/percent sums.
- Modify: `tests/bin/surgeist-layout-generate/generator.rs`
  - Serialize calc values into XML attributes without resolving them to px.
  - Add generator unit tests that cover calc XML serialization and helper smoke tests.
- Modify: `tests/layout/browser_parity/support.rs`
  - Parse fixture-local calc strings into `surgeist-style::CalcLength`.
  - Lower calc-bearing resolved style values with `s::adapters::layout::lower_with_store`.
  - Thread the returned `LayoutCalcStore` through `TestTree` as the fixture calc resolver.
  - Remove or bypass layout-handle-to-style conversion paths that would panic on `layout::Length::Calc`, `layout::LengthAuto::Calc`, `layout::Dimension::Calc`, or calc track sizing.
  - Add parser and parity tests for calc fixture values.
- Modify: `tests/layout/browser_parity/html/block/...`
  - Add constrained calc HTML fixtures for block width, margins, padding, and intrinsic width behavior.
- Modify: `tests/layout/browser_parity/html/flex/...`
  - Add constrained calc HTML fixtures for flex basis, width, margins, and gap.
- Modify: `tests/layout/browser_parity/html/grid/...`
  - Add constrained calc HTML fixtures for grid item size/margins and track sizing.
- Modify: `tests/layout/browser_parity/corpus.toml`
  - Register new Surgeist-authored calc fixtures as active cases.
- Generated: `tests/layout/browser_parity/xml/...`
  - Regenerate XML after source/helper/parser changes. Do not hand-edit generated geometry.
- Generated: `tests/layout/browser_parity/xml/generation-reports/all.json` and related report files
  - Regenerate reports with the updated helper hash and generated case inventory.
- Optional docs modify: `tests/layout/browser_parity/README.md`
  - Document that the XML fixture format supports a limited `calc(px +/- percent)` syntax for layout parity tests.

## Task 1: Characterize Calc Fixture Syntax Rejection

**Files:**
- Modify: `tests/layout/browser_parity/support.rs`

- [ ] **Step 1: Add failing parser tests for calc strings**

Add these tests near the existing parser tests in `tests/layout/browser_parity/support.rs`. The tests name the intended style-facing parser helpers before they exist:

```rust
#[test]
fn parse_style_length_accepts_fixture_calc_px_plus_percent() {
    let length = parse_style_length("calc(12px + 25%)").expect("fixture calc should parse");
    assert!(matches!(length, s::Length::Calc(_)));
}

#[test]
fn parse_style_dimension_accepts_fixture_calc_percent_minus_px() {
    let dimension =
        parse_style_dimension("calc(50% - 8px)").expect("fixture calc dimension should parse");
    assert!(matches!(dimension, s::Length::Calc(_)));
}

#[test]
fn parse_style_length_rejects_unsupported_calc_fixture_syntax() {
    let error = parse_style_length("calc(100% / 2)").expect_err("division is not supported yet");
    assert!(
        error.to_string().contains("unsupported calc expression"),
        "unexpected error: {error}"
    );
}
```

- [ ] **Step 2: Run tests to verify failure**

Run each focused test individually:

```sh
cargo test -p surgeist-layout --test layout layout::browser_parity::support::tests::parse_style_length_accepts_fixture_calc_px_plus_percent -- --nocapture
cargo test -p surgeist-layout --test layout layout::browser_parity::support::tests::parse_style_dimension_accepts_fixture_calc_percent_minus_px -- --nocapture
cargo test -p surgeist-layout --test layout layout::browser_parity::support::tests::parse_style_length_rejects_unsupported_calc_fixture_syntax -- --nocapture
```

Expected: the tests fail to compile because the style-facing helper functions do not exist yet.

- [ ] **Step 3: Do not commit red tests**

Continue without committing until Task 2 makes the tests pass. Intentionally red compile-failing TDD commits are forbidden unless the coordinator explicitly waives this rule.

## Task 2: Add Fixture-Local Calc Parser And Store Plumbing

**Files:**
- Modify: `tests/layout/browser_parity/support.rs`

- [ ] **Step 1: Add a fixture calc store to `TestTree`**

Change `TestTree` to own a layout calc store:

```rust
#[derive(Clone, Debug, Default)]
struct TestTree {
    nodes: Vec<TestNode>,
    calc_store: layout::LayoutCalcStore,
}
```

Add the resolver implementation expected by the layout compute path:

```rust
impl layout::CalcResolver for TestTree {
    fn resolve_calc(&self, id: layout::CalcId, basis: Option<Scalar>) -> layout::CalcResolution {
        self.calc_store.resolve_calc(id, basis)
    }

    fn calc_depends_on_basis(&self, id: layout::CalcId) -> bool {
        self.calc_store.calc_depends_on_basis(id)
    }

    fn calc_percent_fraction(&self, id: layout::CalcId) -> Option<Scalar> {
        self.calc_store.calc_percent_fraction(id)
    }
}
```

Update the `layout::Compute` implementation for `TestTree` so `calc_resolver()` returns `self`:

```rust
fn calc_resolver(&self) -> &dyn layout::CalcResolver {
    self
}
```

- [ ] **Step 2: Add parser helpers**

Add these helper signatures near the existing length parsers. These helpers produce style calc values because XML style attributes already enter the parity runner through `surgeist-style` declarations:

```rust
fn parse_style_calc_length(raw: &str) -> Result<s::CalcLength, Error> {
    let body = raw
        .strip_prefix("calc(")
        .and_then(|value| value.strip_suffix(')'))
        .ok_or_else(|| Error::new(format!("unsupported calc expression `{raw}`")))?;
    parse_style_calc_sum(body.trim(), raw)
}

fn parse_style_calc_sum(body: &str, raw: &str) -> Result<s::CalcLength, Error> {
    let parts = body.split_whitespace().collect::<Vec<_>>();
    let [first, operator, second] = parts.as_slice() else {
        return Err(Error::new(format!("unsupported calc expression `{raw}`")));
    };

    let left = parse_style_calc_term(first)?;
    let right = parse_style_calc_term(second)?;
    let right = match *operator {
        "+" => s::CalcLengthTerm::add(right),
        "-" => s::CalcLengthTerm::sub(right),
        _ => return Err(Error::new(format!("unsupported calc expression `{raw}`"))),
    };

    Ok(s::CalcLength::sum([s::CalcLengthTerm::add(left), right]))
}

fn parse_style_calc_term(raw: &str) -> Result<s::CalcLength, Error> {
    if let Some(px) = raw.strip_suffix("px") {
        return Ok(s::CalcLength::px(parse_number(px)?));
    }
    if let Some(percent) = raw.strip_suffix('%') {
        return Ok(s::CalcLength::percent(parse_number(percent)?));
    }
    Err(Error::new(format!("unsupported calc expression term `{raw}`")))
}
```

The implementation must avoid accepting arbitrary CSS. This exact helper intentionally requires spaces around the operator, matching the fixtures in this plan. It does not parse nested parentheses or compact syntax like `calc(50%+20px)`.

- [ ] **Step 3: Add calc-aware value parsing**

Because parsing XML style attributes now needs style calc values, replace direct `parse_length(raw)` and `to_style_length(parse_length(raw))` calls in `to_declarations` with style-aware equivalents. Use signatures like:

```rust
fn parse_style_length(raw: &str) -> Result<s::Length, Error> {
    if raw.trim_start().starts_with("calc(") {
        return Ok(s::Length::Calc(parse_style_calc_length(raw)?));
    }
    Ok(to_style_length(parse_length(raw)?))
}

fn parse_style_length_auto(raw: &str) -> Result<s::Length, Error> {
    if raw == "auto" {
        return Ok(s::Length::Auto);
    }
    parse_style_length(raw)
}

fn parse_style_dimension(raw: &str) -> Result<s::Length, Error> {
    match raw {
        "auto" => Ok(s::Length::Auto),
        "min-content" => Ok(s::Length::MinContent),
        "max-content" => Ok(s::Length::MaxContent),
        _ => {
            if raw.trim_start().starts_with("calc(") {
                return Ok(s::Length::Calc(parse_style_calc_length(raw)?));
            }
            to_style_dimension(parse_dimension(raw)?)
        }
    }
}
```

Update `to_declarations`, `insert_edges`, `insert_edges_auto`, gap parsing, flex-basis parsing, size/min/max parsing, and style track-list parsing so calc-capable properties use the style-aware helpers.

For grid track lists, do not delegate calc-bearing track sizing through calc-blind layout parsing. Add or use style-native parsing for track sizing components that may contain calc, then lower through `LayoutLoweringSession`. Cover direct track lengths such as `calc(25% + 20px)`, and generated forms that may appear inside `fit-content(calc(...))`, `minmax(calc(...), ...)`, and `repeat(...)` contents.

- [ ] **Step 4: Lower calc values with one layout calc store**

Change `to_node_input` so callers pass one shared `s::adapters::layout::LayoutLoweringSession` while the whole XML tree is built:

```rust
fn to_node_input(
    attrs: &StyleAttrs,
    lowering: &mut s::adapters::layout::LayoutLoweringSession,
) -> Result<layout::NodeInput, Error> {
    let declarations = to_declarations(attrs)?;
    let tree = StyleFixtureTree::default();
    let mut resolver = s::Resolver::new(s::Sheet::new());
    let resolved = resolver
        .resolve(s::Context::new(&tree, 0).local(&declarations))
        .map_err(|error| Error::new(error.to_string()))?;
    let mut input = lowering
        .lower_node(&resolved)
        .map_err(|error| Error::new(error.to_string()))?;
    if let Some(value) = attrs.get("vertical-align") {
        input.vertical_align = parse_vertical_align(value)?;
    }
    Ok(input)
}
```

In `TestTree::from_golden`, create one `LayoutLoweringSession`, pass it through every recursive `push_node` call, then assign `tree.calc_store = lowering.finish()` after all nodes have been lowered. This avoids per-node `CalcId` remapping.

- [ ] **Step 5: Run parser tests**

Run:

```sh
cargo test -p surgeist-layout --test layout layout::browser_parity::support::tests::parse_style_length_accepts_fixture_calc_px_plus_percent -- --nocapture
cargo test -p surgeist-layout --test layout layout::browser_parity::support::tests::parse_style_dimension_accepts_fixture_calc_percent_minus_px -- --nocapture
cargo test -p surgeist-layout --test layout layout::browser_parity::support::tests::parse_style_length_rejects_unsupported_calc_fixture_syntax -- --nocapture
```

Expected: all pass.

- [ ] **Step 6: Commit**

```sh
git add tests/layout/browser_parity/support.rs
git commit -m "layout: parse calc values in parity XML"
```

## Task 3: Preserve Calc Values During Browser Generation

**Files:**
- Modify: `tests/layout/browser_parity/scripts/gentest/test_helper.js`
- Modify: `tests/bin/surgeist-layout-generate/generator.rs`

- [ ] **Step 1: Add failing generator/helper tests**

Add a generator unit test that exercises JSON-to-XML serialization for calc values:

```rust
#[test]
fn xml_generation_preserves_calc_lengths() {
    let node = json!({
        "useRounding": true,
        "viewport": {"width": {"unit": "px", "value": 200}, "height": {"unit": "max-content"}},
        "style": {
            "display": "block",
            "size": {"width": {"unit": "calc", "value": "calc(50% + 20px)"}},
            "margin": {"left": {"unit": "calc", "value": "calc(10% - 4px)"}}
        },
        "smartRoundedLayout": {"x": 0, "y": 0, "width": 120, "height": 10, "scrollWidth": 120, "scrollHeight": 10},
        "unroundedLayout": {"x": 0, "y": 0, "width": 120, "height": 10, "scrollWidth": 120, "scrollHeight": 10},
        "naivelyRoundedLayout": {"clientWidth": 120, "clientHeight": 10},
        "children": []
    });

    let xml = generate_xml("calc_lengths__border_box_ltr", &node);

    assert!(xml.contains(r#"width="calc(50% + 20px)""#));
    assert!(xml.contains(r#"margin-left="calc(10% - 4px)""#));
}
```

Add a generator unit test that proves calc values survive inside grid track lists:

```rust
#[test]
fn xml_generation_preserves_calc_grid_tracks() {
    let node = json!({
        "useRounding": true,
        "viewport": {"width": {"unit": "px", "value": 240}, "height": {"unit": "max-content"}},
        "style": {
            "display": "grid",
            "gridTemplateColumns": [
                {"kind": "scalar", "unit": "calc", "value": "calc(25% + 20px)"},
                {"kind": "scalar", "unit": "px", "value": 80}
            ]
        },
        "smartRoundedLayout": {"x": 0, "y": 0, "width": 240, "height": 10, "scrollWidth": 240, "scrollHeight": 10},
        "unroundedLayout": {"x": 0, "y": 0, "width": 240, "height": 10, "scrollWidth": 240, "scrollHeight": 10},
        "naivelyRoundedLayout": {"clientWidth": 240, "clientHeight": 10},
        "children": []
    });

    let xml = generate_xml("calc_grid_tracks__border_box_ltr", &node);

    assert!(xml.contains(r#"grid-template-columns="calc(25% + 20px) 80px""#));
}
```

Add a helper smoke test for Typed OM calc values. Use a fake Typed OM object that mimics Chrome's sum-like representation only after checking the real returned shape in a browser probe. The worker must document the observed shape in a short test comment.

- [ ] **Step 2: Run tests to verify failure**

Run:

```sh
cargo test -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate xml_generation_preserves_calc_lengths -- --nocapture
```

Expected: fails because the XML serializer does not know `unit: "calc"` and the grid track parser/helper does not preserve calc track components yet.

- [ ] **Step 3: Implement XML serialization for calc JSON values**

Find the generator function that serializes dimensions/lengths. Extend it so:

```rust
match unit {
    "calc" => escape_attr(value["value"].as_str().ok_or_else(...)?),
    "px" => format!("{}px", ...),
    "percent" => format!("{}%", ...),
    ...
}
```

The worker must keep generated XML values escaped through the existing attribute escaping path.

- [ ] **Step 4: Implement JS helper preservation**

Update `parseDimension` or add a parallel helper so the generator can emit:

```js
{ unit: "calc", value: "calc(50% + 20px)" }
```

Use browser-selected Typed OM values where possible. If Chrome Typed OM cannot expose a reconstructable calc expression, fall back only to inline authored calc declarations and document this limitation in the helper test. Do not scan all matching stylesheets manually; that repeats the cascade bug fixed for percent margins.

- [ ] **Step 5: Support calc inside generated grid track values**

Update `tests/layout/browser_parity/scripts/gentest/test_helper.js` so `TrackSizingParser` recognizes `calc(...)` track lengths instead of treating `calc` as a generic comma-argument function. The implementation must preserve a track component shape like `{ kind: "scalar", unit: "calc", value: "calc(25% + 20px)" }` that the Rust serializer can write back as `calc(25% + 20px)`.

Add or update Rust-side serializer support so any track scalar with `{ kind: "scalar", unit: "calc", value: "calc(...)" }` uses the same escaped calc string path as ordinary dimensions and edges.

- [ ] **Step 6: Run generator tests**

Run:

```sh
cargo test -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate calc -- --nocapture
cargo test -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate bundled_helper_ -- --nocapture
```

Expected: calc serialization tests and helper tests pass.

- [ ] **Step 7: Commit**

```sh
git add tests/layout/browser_parity/scripts/gentest/test_helper.js tests/bin/surgeist-layout-generate/generator.rs
git commit -m "layout: preserve calc values in parity generation"
```

## Task 4: Add Minimal Calc HTML Fixtures

**Files:**
- Create: `tests/layout/browser_parity/html/block/block_calc_width_margin.html`
- Create: `tests/layout/browser_parity/html/flex/flex_calc_basis_margin_gap.html`
- Create: `tests/layout/browser_parity/html/grid/grid_calc_track_and_item_margin.html`
- Modify: `tests/layout/browser_parity/corpus.toml`

- [ ] **Step 1: Add block fixture**

Create `tests/layout/browser_parity/html/block/block_calc_width_margin.html`:

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <script src="../../scripts/gentest/test_helper.js"></script>
  <link rel="stylesheet" type="text/css" href="../../scripts/gentest/test_base_style.css">
  <title>Block calc width and margin</title>
</head>
<body>
<div id="test-root" style="display: block; width: 200px;">
  <div style="height: 10px; width: calc(50% + 20px); margin-left: calc(10% - 4px); margin-right: calc(5% + 2px);"></div>
</div>
</body>
</html>
```

- [ ] **Step 2: Add flex fixture**

Create `tests/layout/browser_parity/html/flex/flex_calc_basis_margin_gap.html`:

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <script src="../../scripts/gentest/test_helper.js"></script>
  <link rel="stylesheet" type="text/css" href="../../scripts/gentest/test_base_style.css">
  <title>Flex calc basis margin gap</title>
</head>
<body>
<div id="test-root" style="display: flex; width: 240px; gap: calc(5% + 2px);">
  <div style="height: 10px; flex: 0 0 calc(25% + 12px); margin-left: calc(10% - 6px);"></div>
  <div style="height: 10px; width: calc(30% + 4px);"></div>
</div>
</body>
</html>
```

- [ ] **Step 3: Add grid fixture**

Create `tests/layout/browser_parity/html/grid/grid_calc_track_and_item_margin.html`:

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <script src="../../scripts/gentest/test_helper.js"></script>
  <link rel="stylesheet" type="text/css" href="../../scripts/gentest/test_base_style.css">
  <title>Grid calc track and item margin</title>
</head>
<body>
<div id="test-root" style="display: grid; width: 240px; grid-template-columns: calc(25% + 20px) 80px; grid-template-rows: 40px;">
  <div style="height: 10px; margin-left: calc(10% - 4px); margin-right: calc(5% + 2px);"></div>
  <div style="height: 10px; width: calc(50% + 10px);"></div>
</div>
</body>
</html>
```

- [ ] **Step 4: Register fixtures in corpus manifest**

Append these entries to `tests/layout/browser_parity/corpus.toml` near other Surgeist-authored cases:

```toml
[[cases]]
id = "block/block_calc_width_margin"
source_root = "surgeist"
source = "block/block_calc_width_margin.html"
generator = "constrained-html"
status = "active"

[[cases]]
id = "flex/flex_calc_basis_margin_gap"
source_root = "surgeist"
source = "flex/flex_calc_basis_margin_gap.html"
generator = "constrained-html"
status = "active"

[[cases]]
id = "grid/grid_calc_track_and_item_margin"
source_root = "surgeist"
source = "grid/grid_calc_track_and_item_margin.html"
generator = "constrained-html"
status = "active"
```

- [ ] **Step 5: Generate only the new fixtures**

Run:

```sh
SURGEIST_LAYOUT_GENERATE_FILTER=block/block_calc_width_margin cargo run -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate
SURGEIST_LAYOUT_GENERATE_FILTER=flex/flex_calc_basis_margin_gap cargo run -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate
SURGEIST_LAYOUT_GENERATE_FILTER=grid/grid_calc_track_and_item_margin cargo run -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate
```

Expected: XML files are generated for all three fixtures and their four variants.

- [ ] **Step 6: Verify generated XML preserves calc strings**

Run:

```sh
rg -n 'calc\\(' tests/layout/browser_parity/xml/block/block_calc_width_margin__*.xml tests/layout/browser_parity/xml/flex/flex_calc_basis_margin_gap__*.xml tests/layout/browser_parity/xml/grid/grid_calc_track_and_item_margin__*.xml
```

Expected: output includes calc strings in input attributes, not only resolved px geometry.

- [ ] **Step 7: Do not commit yet**

Do not commit the new active fixtures immediately after generation. Continue to Task 5 first so the active corpus is only committed after focused calc parse and parity checks pass.

## Task 5: Run Calc Fixtures Through XML Parity

**Files:**
- Modify: `tests/layout/browser_parity/support.rs`
- Generated: `tests/layout/browser_parity/xml/...`

- [ ] **Step 1: Run XML parse tests**

Run:

```sh
cargo test -p surgeist-layout --test layout layout::browser_parity::parses_all_checked_in_browser_parity_xml -- --nocapture
```

Expected: all checked-in browser parity XML files parse, including the calc XML files. `SURGEIST_PARITY_FILTER=calc` does not currently focus this parse-check test; only add that environment variable here if a filtered parse helper is implemented first. If this fails, fix the parser in `support.rs` rather than editing generated XML.

- [ ] **Step 2: Run focused parity test**

Run:

```sh
SURGEIST_PARITY_FILTER=calc cargo test -p surgeist-layout --test layout layout::browser_parity::runs_all_checked_in_browser_parity_xml -- --ignored --nocapture
```

Expected: either all calc fixtures pass, or failures identify real layout calc behavior gaps. If failures occur:

- If the failure is parser/lowering-related, fix this plan's fixture support.
- If the failure is production layout behavior, stop and write a follow-up layout implementation issue; do not mutate expected geometry.

- [ ] **Step 3: Add targeted support tests for any parser/lowering fix**

For every parser/lowering bug found in Step 2, add a focused `support.rs` unit test before fixing it. Example:

```rust
#[test]
fn to_node_input_lowers_calc_margin_with_fixture_store() {
    let raw = r#"
      <test name="calc_margin" use-rounding="true">
        <viewport width="200px" height="max-content"/>
        <input><div display="block" margin-left="calc(10% - 4px)"/></input>
        <expectations><node x="0" y="0" width="0" height="0"/></expectations>
      </test>
    "#;
    let golden = Golden::parse(raw).expect("calc XML should parse");
    let tree = TestTree::from_golden(&golden.root).expect("calc node should lower");
    assert!(matches!(tree.nodes[0].node_input.margin.left, layout::LengthAuto::Calc(_)));
}
```

- [ ] **Step 4: Commit active calc fixtures and support fixes**

```sh
git add tests/layout/browser_parity/support.rs tests/layout/browser_parity/html tests/layout/browser_parity/corpus.toml tests/layout/browser_parity/xml
git commit -m "layout: run calc fixtures through XML parity"
```

If focused parity exposes a real production layout calc behavior gap, do not commit active failing fixtures. Instead, leave the corpus changes unstaged, write a follow-up issue draft describing the failing fixture and expected/observed geometry, and stop for coordinator action.

## Task 6: Full Corpus Regeneration And Verification

**Files:**
- Generated: `tests/layout/browser_parity/xml/...`
- Optional modify: `tests/layout/browser_parity/README.md`

- [ ] **Step 1: Document fixture calc syntax**

If Task 2 introduced fixture-local calc syntax, add this short note to `tests/layout/browser_parity/README.md`:

```markdown
The XML fixture format accepts a deliberately small calc syntax for layout
parity inputs: `calc(<number>px +/- <number>%)`. This is fixture syntax only;
app-facing CSS parsing remains owned by `surgeist-css` and authored style calc
modeling remains owned by `surgeist-style`.
```

- [ ] **Step 2: Regenerate full browser parity XML**

Run:

```sh
cargo run -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate
```

Expected: generation exits `0`. Unsupported output may still mention `<br>`, mixed text/element content, and missing `#test-root`; those are unrelated to calc fixture support.

- [ ] **Step 3: Check corpus freshness**

Run:

```sh
cargo run -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- check-corpus
```

Expected: exits `0`.

- [ ] **Step 4: Run focused calc parity**

Run:

```sh
SURGEIST_PARITY_FILTER=calc cargo test -p surgeist-layout --test layout layout::browser_parity::runs_all_checked_in_browser_parity_xml -- --ignored --nocapture
```

Expected: exits `0`.

- [ ] **Step 5: Run full ignored XML parity**

Run:

```sh
cargo test -p surgeist-layout --test layout layout::browser_parity::runs_all_checked_in_browser_parity_xml -- --ignored --nocapture
```

Expected: exits `0`. If non-calc failures appear, compare against the prior clean baseline and stop for coordinator review before changing geometry or layout behavior.

- [ ] **Step 6: Run crate baseline checks**

Run:

```sh
cargo test -p surgeist-layout
cargo clippy -p surgeist-layout --all-targets -- -D warnings
cargo fmt --check
```

Expected: all pass.

- [ ] **Step 7: Commit regenerated corpus and docs**

```sh
git add tests/layout/browser_parity/README.md tests/layout/browser_parity/xml
git commit -m "test: refresh calc parity corpus"
```

Skip `tests/layout/browser_parity/README.md` from `git add` if no docs change was needed.

## Task 7: Final Review And Handoff

**Files:**
- No code changes expected.

- [ ] **Step 1: Request final clean review**

Ask a separate reviewer to inspect:

- calc XML grammar and rejection behavior,
- generator preservation of calc strings,
- layout calc store/resolver threading in XML parity,
- new fixture coverage,
- generated artifact consistency,
- crate boundary compliance.

- [ ] **Step 2: Address reviewer findings**

If findings are valid, send them back to an implementation worker. Do not self-edit code as coordinator.

- [ ] **Step 3: Final status and push**

After review is clean and checks pass:

```sh
git status --short --branch
git log --oneline -5
```

Push only when requested, when another repo/thread must fetch the commit, or when the top-level repo needs to update the `surgeist-layout` submodule pointer:

```sh
git push origin main
```

## Verification Summary Required From Implementation

The final implementation report must include:

- the commit range,
- all changed source, fixture, and generated-artifact paths,
- whether any sibling crate issue was discovered,
- generation report counts before and after,
- any relevant generated report path, especially `tests/layout/browser_parity/xml/generation-reports/all.json`,
- confirmation that `rg -n 'calc\\(' tests/layout/browser_parity/xml/...` finds calc input attributes,
- verification command outputs for:
  - `cargo run -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- check-corpus`,
  - `SURGEIST_PARITY_FILTER=calc cargo test -p surgeist-layout --test layout layout::browser_parity::runs_all_checked_in_browser_parity_xml -- --ignored --nocapture`,
  - full ignored XML parity,
  - `cargo test -p surgeist-layout`,
  - `cargo clippy -p surgeist-layout --all-targets -- -D warnings`,
  - `cargo fmt --check`.

## Risks And Open Questions

- CSS Typed OM representation of calc values must be verified in the pinned Chrome-for-Testing version before finalizing the helper implementation. If Chrome resolves calc values to px for a property, use inline authored fallback only for constrained fixtures and document the limitation.
- `surgeist-style::Length::Calc` carries authored style calc ASTs, not layout `CalcId` handles. Keep the XML support path style-facing until `LayoutLoweringSession` creates the layout handles.
- Track sizing calc support may expose real layout algorithm bugs. Treat those as valuable findings; do not encode wrong browser expectations or weaken fixtures.
- The first fixture grammar is intentionally tiny. Broader CSS calc grammar belongs in `surgeist-css` and should not be smuggled into this crate.
