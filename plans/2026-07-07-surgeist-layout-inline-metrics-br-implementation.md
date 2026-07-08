# Surgeist Layout Inline Metrics BR Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a complete scalar-generic, typed layout-ready inline metrics contract and use it to make horizontal `<br>` line boxes behave like browser line breaks rather than zero-height punctuation.

**Architecture:** Layout owns the normalized inline layout contract: metric-bearing inline participants, forced break items, line boxes, and layout outputs. Style/text/root integration will later provide these metrics from computed style and text shaping, but layout must expose the contract now without depending on sibling crates. `NodeInputOf<S>` remains box-only; `LayoutInputOf<S>::LineBreak` carries `LineBreakInputOf<S>` with validated `InlineMetricsOf<S>`.

**Tech Stack:** Rust 2024, `surgeist-layout`, scalar-generic public `*Of<S>` APIs, `src/node_input.rs`, `src/inline.rs`, `src/block.rs`, source tests in `src/*_tests.rs`, browser parity fixture support in `tests/layout/browser_parity/support.rs`, modeling guidance in `guidance/surgeist-rust-modeling-guide.md`.

---

## Modeling Boundary

Backwards compatibility is explicitly not required. Choose the model that makes invalid inline layout states hard to express.

This plan supersedes the minimal `<br>` behavior from `plans/2026-06-29-surgeist-layout-br-line-break-implementation.md`. The earlier model correctly split line-break nodes away from `NodeInputOf<S>`, but it intentionally left empty-line and line-height behavior out. This plan completes that contract by adding typed inline metrics as the line-layout primitive.

Layout owns:

- `InlineMetricsOf<S>`: layout-ready inline line metrics in layout coordinates.
- `LineBreakInputOf<S>`: line-break-specific input that includes `InlineMetricsOf<S>`.
- Horizontal atomic inline line construction from boxes and metric-bearing forced breaks.
- Browser parity fixture parsing for layout-owned XML attributes that already encode layout-ready metrics.

Layout does not own:

- CSS parsing or authored `line-height` syntax.
- Font fallback, shaping, font metric extraction, or text measurement APIs.
- Retained tree identity or real HTML element classification.
- Root fixture metadata generation.

Root/style/text integration expectation:

```text
HTML <br> element
  -> retained/root classifies node as a line-break layout input
  -> style computes inherited font-size, line-height, direction, writing-mode, vertical-align, clear
  -> text or style/text adapter resolves font-derived inline metrics
  -> root/style adapter constructs surgeist_layout::LineBreakInputOf<S>
  -> layout consumes the layout-ready metrics without parsing CSS or shaping text
```

Do not add `surgeist-style`, `surgeist-text`, `surgeist-retained`, or root `surgeist` dependencies to this crate.

## Typed Contract

Add a public scalar-generic metrics type with private fields:

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InlineMetricsOf<S: LayoutScalar = DefaultScalar> {
    baseline: S,
    line_extent: S,
}

pub type InlineMetrics = InlineMetricsOf<DefaultScalar>;
```

Semantics:

- `baseline` is the distance from the line box block-start edge to the alignment baseline.
- `line_extent` is the total block-axis size required by this inline participant's line metrics.
- `after_baseline() == line_extent - baseline`.
- All values must be finite and non-negative.
- `baseline <= line_extent`. This keeps the baseline inside the metric-bearing line box.
- Metrics are layout-ready. They are not authored `font-size` or CSS `line-height`.

Constructors:

```rust
impl<S: LayoutScalar> InlineMetricsOf<S> {
    pub fn try_new(
        baseline: S,
        line_extent: S,
    ) -> Result<Self, InlineMetricsError<S>>;

    pub fn from_ascent_descent(ascent: S, descent: S) -> Result<Self, InlineMetricsError<S>>;

    pub fn from_line_height_and_baseline(
        line_height: S,
        baseline: S,
    ) -> Result<Self, InlineMetricsError<S>>;

    pub const fn baseline(self) -> S;
    pub const fn line_extent(self) -> S;
    pub fn after_baseline(self) -> S;
}
```

Error type:

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InlineMetricsError<S: LayoutScalar = DefaultScalar> {
    NonFinite { value: S },
    Negative { value: S },
    BaselineExceedsLineExtent { baseline: S, line_extent: S },
    BaselineExceedsLineHeight { baseline: S, line_height: S },
}
```

Default policy:

```rust
impl<S: LayoutScalar> Default for InlineMetricsOf<S> {
    fn default() -> Self {
        Self::from_line_height_and_baseline(S::from_f64(16.0), S::from_f64(12.0))
            .expect("default inline metrics are valid")
    }
}
```

The exact default is a layout fallback, not a browser font metric claim. Browser parity fixtures and integration adapters should provide explicit metrics when checking font-sensitive behavior.

## Task 1: Add Scalar-Generic Inline Metrics Contract

**Files:**

- Modify: `src/node_input.rs`
- Modify: `src/lib.rs`
- Modify: `src/contract_tests.rs`

- [ ] **Step 1: Add failing contract tests**

In `src/contract_tests.rs`, add:

```rust
#[test]
fn inline_metrics_validate_line_box_invariants() {
    let metrics = InlineMetrics::try_new(12.0, 18.0).unwrap();

    assert_eq!(metrics.baseline(), 12.0);
    assert_eq!(metrics.line_extent(), 18.0);
    assert_eq!(metrics.after_baseline(), 6.0);

    assert_eq!(
        InlineMetrics::try_new(19.0, 18.0),
        Err(InlineMetricsError::BaselineExceedsLineExtent {
            baseline: 19.0,
            line_extent: 18.0,
        })
    );
    assert_eq!(
        InlineMetrics::from_line_height_and_baseline(10.0, 12.0),
        Err(InlineMetricsError::BaselineExceedsLineHeight {
            baseline: 12.0,
            line_height: 10.0,
        })
    );
}

#[test]
fn inline_metrics_reject_non_finite_and_negative_values() {
    assert!(matches!(
        InlineMetrics::try_new(f32::NAN, 18.0),
        Err(InlineMetricsError::NonFinite { value }) if value.is_nan()
    ));
    assert_eq!(
        InlineMetrics::try_new(12.0, -18.0),
        Err(InlineMetricsError::Negative { value: -18.0 })
    );
}
```

Also add:

```rust
#[test]
fn inline_metrics_support_f64_scalar_lane() {
    let metrics = InlineMetricsOf::<f64>::from_line_height_and_baseline(
        9_000_000_000_000.0,
        8_000_000_000_000.0,
    )
    .unwrap();

    assert_eq!(metrics.after_baseline(), 1_000_000_000_000.0);
}
```

Remove any duplicate f64 test if it already exists after editing.

The final test set for this step must compile; do not leave both the `assert_eq!` and `matches!` versions of the `NaN` assertion.

Run:

```sh
cargo test -p surgeist-layout inline_metrics -- --nocapture
```

Expected: fail because the public types do not exist yet.

- [ ] **Step 2: Implement `InlineMetricsOf<S>` and error type**

In `src/node_input.rs`, add the types from **Typed Contract** near the other public layout-ready input value types. Keep fields private. Use the existing `LayoutScalar::is_finite()` method for finite-value validation.

Validation implementation:

```rust
fn validate_non_negative_finite<S: LayoutScalar>(value: S) -> Result<(), InlineMetricsError<S>> {
    if !value.is_finite() {
        return Err(InlineMetricsError::NonFinite { value });
    }
    if value < S::ZERO {
        return Err(InlineMetricsError::Negative { value });
    }
    Ok(())
}
```

`try_new` must validate both arguments and reject a baseline greater than the line extent:

```rust
if baseline > line_extent {
    return Err(InlineMetricsError::BaselineExceedsLineExtent {
        baseline,
        line_extent,
    });
}
```

Run:

```sh
cargo test -p surgeist-layout inline_metrics -- --nocapture
```

Expected: fail because the public types do not exist yet.

Do not use public fields and do not accept negative or non-finite metric values.

- [ ] **Step 3: Reexport metrics types**

In `src/lib.rs`, add:

```rust
InlineMetrics, InlineMetricsError, InlineMetricsOf,
```

to the `pub use node_input::{ ... }` list.

- [ ] **Step 4: Run focused tests**

Run:

```sh
cargo test -p surgeist-layout inline_metrics -- --nocapture
```

Expected: pass.

- [ ] **Step 5: Request scoped review**

Reviewer prompt:

```text
Review Task 1 against guidance/surgeist-rust-modeling-guide.md. Confirm InlineMetricsOf<S> is scalar-generic, layout-ready, privately validated, and not an authored CSS/font model. Reject public fields, unchecked constructors, f32-only APIs, or string-only errors.
```

- [ ] **Step 6: Commit**

```sh
git add src/node_input.rs src/lib.rs src/contract_tests.rs
git commit -m "Add typed inline metrics contract"
```

## Task 2: Make LineBreakInput Scalar-Generic And Metric-Bearing

**Files:**

- Modify: `src/node_input.rs`
- Modify: `src/lib.rs`
- Modify: `src/contract_tests.rs`
- Modify: `src/test_support/layout_tree.rs`
- Modify: tests that name `LineBreakInput`

- [ ] **Step 1: Add failing tests for line-break metrics**

In `src/contract_tests.rs`, extend the line-break contract:

```rust
#[test]
fn line_break_input_carries_inline_metrics() {
    let metrics = InlineMetrics::from_line_height_and_baseline(20.0, 15.0).unwrap();
    let input = LineBreakInput::new().with_metrics(metrics);

    assert_eq!(input.metrics(), metrics);
    assert_eq!(input.metrics().line_extent(), 20.0);
}

#[test]
fn line_break_input_supports_f64_metrics() {
    let metrics = InlineMetricsOf::<f64>::from_line_height_and_baseline(32.0, 25.0).unwrap();
    let input = LineBreakInputOf::<f64>::new().with_metrics(metrics);

    assert_eq!(input.metrics().baseline(), 25.0);
}
```

Run:

```sh
cargo test -p surgeist-layout line_break_input -- --nocapture
```

Expected: fail until `LineBreakInputOf<S>` exists.

- [ ] **Step 2: Replace `LineBreakInput` with `LineBreakInputOf<S>`**

In `src/node_input.rs`, change:

```rust
pub struct LineBreakInput {
    ...
}
```

to:

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LineBreakInputOf<S: LayoutScalar = DefaultScalar> {
    display: LineBreakDisplay,
    direction: Direction,
    writing_mode: WritingMode,
    vertical_align: VerticalAlign,
    clear: Clear,
    metrics: InlineMetricsOf<S>,
}

pub type LineBreakInput = LineBreakInputOf<DefaultScalar>;
```

Update `LayoutInputOf<S>`:

```rust
pub enum LayoutInputOf<S: LayoutScalar = DefaultScalar> {
    Box(std::boxed::Box<NodeInputOf<S>>),
    LineBreak(LineBreakInputOf<S>),
}
```

Preserve the existing boxed `Box` variant. Do not change it to `Box(NodeInputOf<S>)`; that would create unrelated churn in block, flex, grid, test support, and browser parity code. Update only the line-break variant and `as_line_break` so it returns `Option<LineBreakInputOf<S>>`.

- [ ] **Step 3: Add metric accessors and builder**

Add:

```rust
impl<S: LayoutScalar> LineBreakInputOf<S> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_metrics(mut self, metrics: InlineMetricsOf<S>) -> Self {
        self.metrics = metrics;
        self
    }

    pub const fn metrics(self) -> InlineMetricsOf<S> {
        self.metrics
    }
}
```

Keep existing builders for display/direction/writing mode/vertical align/clear, now generic over `S`.

- [ ] **Step 4: Update call sites**

Use `rg` to update every direct mention:

```sh
rg -n "LineBreakInput|LayoutInputOf::LineBreak|LayoutInput::LineBreak|as_line_break" src tests
```

Expected update patterns:

```rust
fn line_break(mut self, node: u32, input: LineBreakInputOf<S>) -> Self
```

for generic test support, and:

```rust
LineBreakInput::new()
```

for default scalar tests.

- [ ] **Step 5: Run focused tests**

Run:

```sh
cargo test -p surgeist-layout line_break_input -- --nocapture
cargo test -p surgeist-layout layout_input -- --nocapture
```

Expected: pass.

- [ ] **Step 6: Request scoped review**

Reviewer prompt:

```text
Review Task 2 for scalar-generic API correctness. Confirm no f32-only LineBreakInput remains in generic LayoutInputOf<S>, and the metrics field is typed, private, and validated only through InlineMetricsOf<S>.
```

- [ ] **Step 7: Commit**

```sh
git add src tests
git commit -m "Attach inline metrics to line breaks"
```

## Task 3: Teach Atomic Inline Lines About Metric-Bearing Forced Breaks

**Files:**

- Modify: `src/inline.rs`
- Modify: `src/inline_tests.rs`

- [ ] **Step 1: Add failing inline algorithm tests**

In `src/inline_tests.rs`, add:

```rust
#[test]
fn forced_line_break_metrics_give_empty_line_height() {
    let metrics = InlineMetrics::from_line_height_and_baseline(20.0, 15.0).unwrap();
    let report = layout_atomic_inline_items(AtomicInlineInput {
        available_width: Available::MAX_CONTENT,
        writing_mode: WritingMode::HorizontalTb,
        items: vec![
            AtomicInlineItem::forced_line_break(0, metrics),
            AtomicInlineItem::forced_line_break(1, metrics),
        ],
    });

    assert_eq!(report.size, Size::new(0.0, 40.0));
    assert_eq!(report.first_baseline, Some(15.0));
    assert_eq!(report.last_baseline, Some(35.0));
    assert_eq!(report.items[0].location, Point::new(0.0, 15.0));
    assert_eq!(report.items[1].location, Point::new(0.0, 35.0));
}

#[test]
fn forced_line_break_metrics_expand_line_with_boxes() {
    let metrics = InlineMetrics::from_line_height_and_baseline(30.0, 22.0).unwrap();
    let report = layout_atomic_inline_items(AtomicInlineInput {
        available_width: Available::MAX_CONTENT,
        writing_mode: WritingMode::HorizontalTb,
        items: vec![
            AtomicInlineItem::new(0, Size::new(20.0, 10.0), Edges::ZERO, Some(8.0)),
            AtomicInlineItem::forced_line_break(1, metrics),
            AtomicInlineItem::new(2, Size::new(10.0, 10.0), Edges::ZERO, Some(8.0)),
        ],
    });

    assert_eq!(report.size, Size::new(20.0, 40.0));
    assert_eq!(report.items[0].location.y, 14.0);
    assert_eq!(report.items[1].location, Point::new(20.0, 22.0));
    assert_eq!(report.items[2].location.y, 30.0);
}
```

Run:

```sh
cargo test -p surgeist-layout forced_line_break_metrics -- --nocapture
```

Expected: fail because forced breaks do not carry metrics yet.

- [ ] **Step 2: Add metrics to `AtomicInlineItem::ForcedLineBreak`**

In `src/inline.rs`, change:

```rust
ForcedLineBreak { order: u32 },
```

to:

```rust
ForcedLineBreak {
    order: u32,
    metrics: InlineMetricsOf<S>,
},
```

Update constructor:

```rust
pub(super) const fn forced_line_break(order: u32, metrics: InlineMetricsOf<S>) -> Self {
    Self::ForcedLineBreak { order, metrics }
}
```

- [ ] **Step 3: Let line metrics contribute to line height**

Add a line helper:

```rust
fn push_forced_line_break(&mut self, order: u32, metrics: InlineMetricsOf<S>) {
    self.baseline = self.baseline.max(metrics.baseline());
    self.descent = self.descent.max(metrics.after_baseline());
    self.items.push(PendingInlineItem::ForcedLineBreak {
        order,
        x: self.width,
    });
}
```

This makes a line containing only a forced break have real line height. It also makes a forced break with larger metrics expand a line that also contains atomic boxes. The alignment baseline must come from `metrics.baseline()`, not from `line_extent()` or `after_baseline()`.

- [ ] **Step 4: Preserve existing width behavior**

Keep `atomic_inline_min_content_width` and `atomic_inline_max_content_width` width logic unchanged except for matching the new forced-break fields:

```rust
AtomicInlineItem::ForcedLineBreak { .. } => None
```

for min-content and segment split for max-content.

- [ ] **Step 5: Run focused tests**

Run:

```sh
cargo test -p surgeist-layout forced_line_break -- --nocapture
cargo test -p surgeist-layout atomic_inline -- --nocapture
```

Expected: pass.

- [ ] **Step 6: Request scoped review**

Reviewer prompt:

```text
Review Task 3 for inline algorithm correctness. Confirm forced breaks now contribute line baseline/descent through InlineMetricsOf<S>, consecutive and leading breaks create line-height-bearing lines, widths remain segment-based, and vertical forced breaks still reject explicitly rather than silently dropping metrics.
```

- [ ] **Step 7: Commit**

```sh
git add src/inline.rs src/inline_tests.rs
git commit -m "Use inline metrics for forced break lines"
```

## Task 4: Thread Line-Break Metrics Through Block Inline Runs

**Files:**

- Modify: `src/block.rs`
- Modify: `src/block_tests.rs`

- [ ] **Step 1: Add failing block tests**

In `src/block_tests.rs`, add:

```rust
#[test]
fn block_line_break_metrics_create_empty_line_height() {
    let metrics = InlineMetrics::from_line_height_and_baseline(20.0, 15.0).unwrap();
    let mut tree = OracleTree::new()
        .style(0, NodeInput { display: Display::Block, ..NodeInput::default() })
        .children(0, [1, 2])
        .line_break(1, LineBreakInput::new().with_metrics(metrics))
        .line_break(2, LineBreakInput::new().with_metrics(metrics));

    let output = compute_block(&mut tree, 0, ComputeInput::ROOT);

    assert_eq!(output.size.height, 40.0);
    assert_eq!(output.first_baselines.y, Some(15.0));
    assert_eq!(output.last_baselines.y, Some(35.0));
    assert_eq!(tree.layout(1).unwrap().location.y, 15.0);
    assert_eq!(tree.layout(2).unwrap().location.y, 35.0);
}
```

Run:

```sh
cargo test -p surgeist-layout block_line_break_metrics -- --nocapture
```

Expected: fail until block passes metrics into forced break items.

- [ ] **Step 2: Pass metrics from `LineBreakInputOf<S>` to inline item**

In `src/block.rs`, inside `layout_atomic_inline_run`, change the line-break branch from:

```rust
items.push(AtomicInlineItem::forced_line_break(order));
```

to:

```rust
items.push(AtomicInlineItem::forced_line_break(order, input.metrics()));
```

Keep vertical writing mode rejection explicit. Do not call `compute_child` or `node_input` for line-break nodes.

- [ ] **Step 3: Keep write-back zero-sized**

Line-break `NodeOutputOf<S>` remains zero-sized. Metrics affect the containing line box, not the break node’s own border box:

```rust
size: Size::ZERO,
content_size: Size::ZERO,
margin: Edges::ZERO,
padding: Edges::ZERO,
border: Edges::ZERO,
```

- [ ] **Step 4: Run focused tests**

Run:

```sh
cargo test -p surgeist-layout block_line_break_metrics -- --nocapture
cargo test -p surgeist-layout line_break -- --nocapture
```

Expected: pass.

- [ ] **Step 5: Request scoped review**

Reviewer prompt:

```text
Review Task 4 for block/inline boundary correctness. Confirm LineBreakInputOf<S> metrics are consumed only by inline line construction, line-break nodes remain non-box zero-size outputs, hidden breaks do not create metrics, and line breaks cannot fall through to box compute.
```

- [ ] **Step 6: Commit**

```sh
git add src/block.rs src/block_tests.rs
git commit -m "Thread line break metrics through block layout"
```

## Task 5: Update Browser Parity Support For Layout-Ready Inline Metrics

**Files:**

- Modify: `tests/layout/browser_parity/support.rs`
- Modify: `tests/layout/browser_parity/README.md`

- [ ] **Step 1: Add support tests for explicit metrics attributes**

In `tests/layout/browser_parity/support.rs`, add tests near existing `source_tag_br_*` tests:

```rust
#[test]
fn source_tag_br_lowers_explicit_inline_metrics() {
    let input = to_layout_input(
        &StyleAttrs {
            attrs: BTreeMap::from([
                ("source-tag".to_string(), "br".to_string()),
                ("inline-baseline".to_string(), "15px".to_string()),
                ("inline-line-height".to_string(), "20px".to_string()),
            ]),
        },
        &mut layout::LayoutCalcStore::new(),
    )
    .expect("br metrics should lower");

    let layout::LayoutInput::LineBreak(input) = input else {
        panic!("br should lower to line break");
    };

    assert_eq!(input.metrics().baseline(), 15.0);
    assert_eq!(input.metrics().line_extent(), 20.0);
    assert_eq!(input.metrics().after_baseline(), 5.0);
}

#[test]
fn source_tag_br_rejects_partial_inline_metrics() {
    let error = to_layout_input(
        &StyleAttrs {
            attrs: BTreeMap::from([
                ("source-tag".to_string(), "br".to_string()),
                ("inline-baseline".to_string(), "15px".to_string()),
            ]),
        },
        &mut layout::LayoutCalcStore::new(),
    )
    .expect_err("partial br metrics should be rejected");

    assert!(error.to_string().contains("inline metrics require"));
}
```

Run:

```sh
cargo test -p surgeist-layout --test layout source_tag_br_lowers_explicit_inline_metrics -- --nocapture
```

Expected: fail until parser support exists.

- [ ] **Step 2: Parse layout-ready inline metrics attributes**

Add a helper:

```rust
fn inline_metrics(attrs: &StyleAttrs) -> Result<Option<layout::InlineMetrics>, Error> {
    match (
        attrs.get("inline-baseline"),
        attrs.get("inline-line-height"),
    ) {
        (None, None) => Ok(None),
        (Some(baseline), Some(line_height)) => {
            layout::InlineMetrics::from_line_height_and_baseline(
                parse_px_dimension(line_height, "inline-line-height")?,
                parse_px_dimension(baseline, "inline-baseline")?,
            )
            .map(Some)
            .map_err(|error| Error::new(format!("{error:?}")))
        }
        _ => Err(Error::new(
            "inline metrics require inline-baseline and inline-line-height",
        )),
    }
}
```

When lowering `source-tag="br"`, use explicit metrics when present:

```rust
if let Some(metrics) = inline_metrics(attrs)? {
    br = br.with_metrics(metrics);
}
```

Do not invent unavailable root metadata. Until root generation emits these attributes, default metrics keep existing XML parseable.

- [ ] **Step 3: Add README contract note**

In `tests/layout/browser_parity/README.md`, document:

```text
Inline metrics attributes are layout-ready fixture data. They are not CSS syntax. Root/style/text integration is expected to generate them from computed style and text/font metrics later.
```

- [ ] **Step 4: Run focused tests**

Run:

```sh
cargo test -p surgeist-layout --test layout source_tag_br -- --nocapture
cargo test -p surgeist-layout --test layout parses_all_checked_in_browser_parity_xml -- --nocapture
```

Expected: pass.

- [ ] **Step 5: Request scoped review**

Reviewer prompt:

```text
Review Task 5 for fixture boundary correctness. Confirm the XML parser consumes layout-ready inline metrics without reintroducing style/retained/text dependencies, does not require unavailable root metadata, and rejects partial metric pairs.
```

- [ ] **Step 6: Commit**

```sh
git add tests/layout/browser_parity/support.rs tests/layout/browser_parity/README.md
git commit -m "Parse layout-ready inline metrics fixtures"
```

## Task 6: Add Browser-Parity Fixture Evidence For Real BR Lines

**Files:**

- Modify or create: `tests/layout/browser_parity/html/**`
- Modify: `tests/layout/browser_parity/corpus.toml`
- Modify: `tests/layout/browser_parity/scripts/gentest/test_helper.js`
- Modify: `tests/bin/surgeist-layout-generate/generator.rs`
- Modify: `tests/layout/browser_parity.rs`
- Modify generated: `tests/layout/browser_parity/xml/**`
- Modify generated: `tests/layout/browser_parity/xml/generation-reports/*.json`
- Modify: `tests/layout/browser_parity/README.md` if counts or commands change

- [ ] **Step 1: Create narrow HTML cases**

Use existing block HTML cases as templates. Add at least:

```html
<div id="test-root" style="display: block; width: 120px; font-size: 20px; line-height: 30px;"><br><br></div>
```

and:

```html
<div id="test-root" style="display: block; width: 120px; font-size: 20px; line-height: 30px;"><span style="display: inline-block; width: 10px; height: 10px;"></span><br><span style="display: inline-block; width: 10px; height: 10px;"></span></div>
```

Keep this task focused on horizontal block-parent `<br>` behavior. The explicit `display: block` is required because `tests/layout/browser_parity/scripts/gentest/test_base_style.css` makes `div` flex by default, and the generator helper only supports `<br>` whose parent computed display is `block`. Keep the inline-level children adjacent in the HTML source; do not add formatting whitespace between `<br>` and inline-level siblings, because the current helper treats significant inline whitespace as unsupported mixed inline text.

- [ ] **Step 2: Add new local cases to the corpus manifest**

For every new Surgeist-authored fixture, add a `[[cases]]` entry in `tests/layout/browser_parity/corpus.toml`:

```toml
[[cases]]
id = "block/block_br_empty_lines_metrics"
source_root = "surgeist"
source = "block/block_br_empty_lines_metrics.html"
generator = "constrained-html"
status = "active"

[[cases]]
id = "block/block_br_inline_block_metrics"
source_root = "surgeist"
source = "block/block_br_inline_block_metrics.html"
generator = "constrained-html"
status = "active"
```

Use the actual filenames chosen in Step 1. Do not add local HTML outside `corpus.toml`; the generator rejects unlisted local fixtures.

- [ ] **Step 3: Teach the generator to emit layout-ready metrics**

In `tests/layout/browser_parity/scripts/gentest/test_helper.js`, include metric data for `<br>` elements in the returned `style` object from `describeElement`. Use computed layout/style data only; do not add style or text crate dependencies. The helper should prepare metrics before the `return { ... style: { ... } }` object:

```javascript
const brInlineMetrics = brInlineMetricsForElement(e, computedStyle);
```

and include these fields in the returned `style` object:

```javascript
inlineBaseline: brInlineMetrics?.baseline ?? "",
inlineLineHeight: brInlineMetrics?.lineHeight ?? "",
```

Add helpers:

```javascript
function brInlineMetricsForElement(e, computedStyle) {
  if (e.tagName === 'BR') {
    const fontSize = parseCssPx(computedStyle.fontSize);
    const lineHeight = resolveLineHeightPx(computedStyle.lineHeight, fontSize);
    const baseline = estimateInlineBaselinePx(fontSize, lineHeight);
    return {
      baseline: `${baseline}px`,
      lineHeight: `${lineHeight}px`,
    };
  }
  return undefined;
}
```

If the helper does not already have `parseCssPx`, add it as a local helper that accepts only computed pixel values:

```javascript
function parseCssPx(value) {
  if (!value.endsWith("px")) {
    throw new Error(`expected computed px value, got ${value}`);
  }
  return Number(value.slice(0, -2));
}
```

If Chromium reports `line-height: normal`, keep the case explicit:

```javascript
function resolveLineHeightPx(lineHeight, fontSize) {
  if (lineHeight === "normal") {
    return fontSize * 1.2;
  }
  return parseCssPx(lineHeight);
}
```

Add a named helper for the baseline estimate so root/text can replace the generator approximation later without changing the XML attribute contract:

```javascript
function estimateInlineBaselinePx(fontSize, lineHeight) {
  const fontBaseline = fontSize * 0.8;
  const leading = Math.max(0, lineHeight - fontSize);
  return leading / 2 + fontBaseline;
}
```

This generator approximation is fixture tooling, not the product contract. The product contract remains `InlineMetricsOf<S>` provided by integration layers.

In `tests/bin/surgeist-layout-generate/generator.rs`, add serialization support in `input_attrs_with_parent_writing_mode` so `style.inlineBaseline` becomes XML `inline-baseline` and `style.inlineLineHeight` becomes XML `inline-line-height`.

Add focused tests that assert generated BR XML includes `inline-baseline` and `inline-line-height`, and that non-BR fixture elements do not receive these attributes merely because they have text styles. Update `tests/layout/browser_parity.rs` expected generated/unsupported counts after generation so `browser_parity_generation_report_counts_full_scope` reflects the new active cases.

- [ ] **Step 4: Run focused generator tests**

Run:

```sh
cargo test -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate br_inline_metrics -- --nocapture
```

Expected: pass. If the exact test filter differs after adding named tests, run each new generator test by name and record the commands in the worker result.

- [ ] **Step 5: Generate XML using the existing crate command**

Run the currently documented generator command from this repo. If the command is not documented in README, use:

```sh
cargo run --features layout-golden-generate --bin surgeist-layout-generate
```

Do not hand-edit generated XML.

- [ ] **Step 6: Run generated fixture checks**

Run:

```sh
cargo test -p surgeist-layout --test layout parses_all_checked_in_browser_parity_xml -- --nocapture
cargo test -p surgeist-layout --test layout browser_parity_generation_report_counts_full_scope -- --nocapture
```

Expected: generated XML parses and reports classify unsupported vertical/outside-context `<br>` separately from supported horizontal block-parent fixtures.

- [ ] **Step 7: Request scoped review**

Reviewer prompt:

```text
Review Task 6 generator and generated artifacts. Confirm generated files came from the generator, new local HTML cases are listed in corpus.toml, supported horizontal BR cases include complete inline metric attributes, unsupported BR buckets remain explicit, non-BR nodes do not receive BR-only inline metric attrs, and no generated XML was hand-edited.
```

- [ ] **Step 8: Commit**

```sh
git add tests/layout/browser_parity/html tests/layout/browser_parity/corpus.toml tests/layout/browser_parity/scripts/gentest/test_helper.js tests/bin/surgeist-layout-generate/generator.rs tests/layout/browser_parity.rs tests/layout/browser_parity/xml tests/layout/browser_parity/README.md
git commit -m "Add browser parity br metrics fixtures"
```

## Task 7: Document Cross-Crate Contract For Root Integration

**Files:**

- Modify: `README.md`
- Create: `plans/2026-07-07-surgeist-layout-inline-metrics-cross-crate-ledger.md`

- [ ] **Step 1: Add README public contract section**

In `README.md`, add:

```text
## Inline Metrics Contract

`InlineMetricsOf<S>` is layout-ready line box data. Layout consumes it for inline line construction and does not derive it from authored CSS or fonts. Integration layers should provide metrics from computed style and text/font measurement before constructing `LineBreakInputOf<S>`.
```

- [ ] **Step 2: Create cross-crate ledger**

Create `plans/2026-07-07-surgeist-layout-inline-metrics-cross-crate-ledger.md` with:

```markdown
# Surgeist Layout Inline Metrics Cross-Crate Ledger

## Style/Text Adapter Work

- Status: pending root coordination
- Required contract: produce `surgeist_layout::InlineMetricsOf<S>` for line-break nodes from computed font and line-height context.
- Must not: make layout parse authored CSS, depend on `surgeist-style`, or depend on `surgeist-text`.

## Retained/Root Tree Work

- Status: pending root coordination
- Required contract: classify real HTML `<br>` as `LayoutInputOf::LineBreak(LineBreakInputOf<S>)`.
- Must not: model `<br>` as a normal block/flex/grid/leaf `NodeInputOf<S>`.

## Fixture Generator Work

- Status: pending root coordination if not completed in layout
- Required contract: emit complete layout-ready metric pairs for `<br>` fixtures when checking font-sensitive browser parity.
- Must not: emit partial metric pairs or root-private schema fields layout cannot parse.
```

- [ ] **Step 3: Run doc grep**

Run:

```sh
rg -n "InlineMetrics|LineBreakInputOf|surgeist-style|surgeist-text|surgeist-retained" README.md plans/2026-07-07-surgeist-layout-inline-metrics-cross-crate-ledger.md
```

Expected: docs mention sibling crates only as pending integration owners, not dependencies.

- [ ] **Step 4: Request scoped review**

Reviewer prompt:

```text
Review Task 7 docs and ledger. Confirm the contract is sufficient for root/style/text/retained planning, keeps layout ownership clear, and does not imply layout depends on sibling crates.
```

- [ ] **Step 5: Commit**

```sh
git add README.md plans/2026-07-07-surgeist-layout-inline-metrics-cross-crate-ledger.md
git commit -m "Document inline metrics integration contract"
```

## Final Verification

- [ ] **Step 1: Run required checks**

Run:

```sh
cargo test -p surgeist-layout
cargo clippy -p surgeist-layout --all-targets -- -D warnings
cargo fmt --check
git diff --check
git status --short --branch
```

Expected: all pass, and status shows only intentional committed work or a clean tree.

- [ ] **Step 2: Run dependency boundary search**

Run:

```sh
rg -n "surgeist_style|surgeist-style|surgeist_text|surgeist-text|surgeist_retained|surgeist-retained|style::|retained::" src tests Cargo.toml README.md plans/2026-07-07-surgeist-layout-inline-metrics-cross-crate-ledger.md
```

Expected: no source/test/Cargo dependency edges. README/ledger hits are acceptable only when they describe external integration ownership.

- [ ] **Step 3: Final holistic review**

Assign a clean-context reviewer with this prompt:

```text
You are reviewing the complete inline metrics BR implementation in /Users/codex/Development/surgeist-layout. Do not edit files. Review against plans/2026-07-07-surgeist-layout-inline-metrics-br-implementation.md, guidance/surgeist-rust-modeling-guide.md, AGENTS.md, and the code itself.

Check:
- InlineMetricsOf<S> is a typed layout-ready contract, not authored CSS or text/font modeling.
- LineBreakInputOf<S> is scalar-generic and metric-bearing.
- NodeInputOf<S> remains box-only.
- LayoutInputOf<S>::LineBreak is the only line-break node-kind boundary.
- Forced breaks contribute line height/baseline through metrics, including leading and consecutive breaks.
- Line-break node output remains zero-sized and is never sent through box compute.
- Browser parity support consumes layout-ready metrics without sibling crate dependencies or partial metric states.
- Cross-crate ledger gives root/style/text/retained enough information to plan their side.
- The implementation is correct even if it differs from this plan in a justified way.

Return findings first. If clean, say "No findings."
```

Completion gate: the goal is complete only when this final holistic review is clean or every finding has been reconciled and re-reviewed clean.
