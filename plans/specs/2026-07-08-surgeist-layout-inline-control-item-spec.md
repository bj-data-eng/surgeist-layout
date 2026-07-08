# Surgeist Layout Inline Control Item Specification

## Purpose

This specification defines the layout-owned inline control item model needed to
support browser-like `<br>` behavior without moving HTML, CSS, style, retained
tree, or text-shaping ownership into `surgeist-layout`.

The reference shape is WebKit's split between `HTMLBRElement`,
`RenderLineBreak`, layout tree `LineBreak`, inline `HardLineBreak`, and line-box
construction. The Surgeist model should follow the same ownership boundary:
`<br>` is normalized before layout, then layout receives a typed inline control
participant that forces line construction.

## Non-Goals

- Do not parse HTML tags, authored attributes, or CSS in layout.
- Do not compute font metrics, line-height, text shaping, or whitespace
  collapsing in layout.
- Do not model editing, selection, caret movement, accessibility, or painting.
- Do not treat `<br>` as a normal block box, atomic inline box, text run, or
  generic node with mostly-unused fields.
- Do not introduce compatibility aliases or fallback lowering paths.

## Ownership Boundary

Layout owns:

- resolved layout-ready inline control items;
- inline formatting context line construction;
- logical-to-physical writing-mode and direction mapping;
- float exclusion and resolved `Clear` effects;
- line metrics aggregation, baselines, and output geometry;
- strict validation of layout-ready invariants.

Other crates own:

- HTML element classification, including recognizing real `<br>` nodes;
- presentational hint lowering, such as `clear="both"`;
- CSS parsing and computed style resolution;
- font selection, text metrics, and line-height resolution;
- retained tree normalization and anonymous wrapper construction;
- root-owned adapter orchestration.

## Reference Mapping From WebKit

| WebKit Concept | Surgeist Layout Concept |
| --- | --- |
| `HTMLBRElement` | Non-layout owner classifies HTML and resolves style. |
| `RenderLineBreak` | Distinct layout-ready line-break node/input, not a box. |
| `Layout::Box::NodeType::LineBreak` | `LayoutInputOf<S>::LineBreak(LineBreakInputOf<S>)`. |
| `InlineItem::HardLineBreak` | `ForcedLineBreakControlOf<S>` inside an inline control stream. |
| `InlineLevelBox::createLineBreakBox` | Line builder creates a line-break participation record. |
| Parent inline box font metrics | `InlineMetricsOf<S>` supplied before layout. |
| `hasFloatClear` for line-break boxes | resolved `Clear` applies to forced line breaks. |

## Core Types

The implementation plans derived from this spec should introduce or evolve
types equivalent to the following semantic model. Exact file placement may vary
with the implementation plan, but the model must stay typed and layout-ready.

```rust
pub struct InlineFlowOf<S: LayoutScalar = DefaultScalar> {
    writing_mode: WritingMode,
    direction: Direction,
    available_inline_extent: AvailableOf<S>,
}

pub struct ForcedLineBreakControlOf<S: LayoutScalar = DefaultScalar> {
    order: u32,
    flow: InlineFlowOf<S>,
    metrics: InlineMetricsOf<S>,
    alignment: InlineControlAlignment,
    clear: Clear,
}

pub enum InlineControlItemOf<S: LayoutScalar = DefaultScalar> {
    ForcedLineBreak(ForcedLineBreakControlOf<S>),
}
```

`InlineControlItemOf<S>` is a closed layout-ready inline-control domain. It must
not become a transport bag for text, boxes, authored CSS values, generated
fixture attributes, or DOM state. If later controls such as word-break
opportunities need different invariants, they should get their own typed payload
rather than sharing unused forced-break fields.

## Forced Line Break Semantics

A forced line break control item:

- participates in inline line construction;
- has no children;
- has no border, padding, margin, scrollbars, paint, or layer;
- has zero output size;
- has no inline advance after the break is applied;
- terminates the current line at its position;
- contributes `InlineMetricsOf<S>` to the committed line's block-axis extent and
  baseline calculation;
- starts following inline content on a new line in the same inline formatting
  context;
- creates metric-bearing empty lines when consecutive forced breaks appear;
- is contentful for line construction even though its output box is zero-size.

Hidden line breaks do not become forced line break control items. A
`LineBreakDisplay::None` input should either be skipped before control item
construction or produce a hidden zero output with no line construction effect.

## Inline Metrics

`InlineMetricsOf<S>` is interpreted in logical line coordinates:

- `baseline` is the distance from the logical line block-start edge to the
  dominant baseline used for this control item.
- `line_extent` is the full logical block-axis extent contributed by this
  control item.
- `after_baseline()` is the logical block-axis distance from the baseline to
  the line block-end edge.

The metrics are already resolved. Layout must not derive them from `font-size`,
font tables, CSS `line-height`, or text content.

For a line containing only a forced break, the line baseline is
`metrics.baseline()` and the line block extent is `metrics.line_extent()`.
For a line containing boxes, text fragments, and a forced break, the line
baseline and line block extent are the aggregate of all participating
layout-ready inline participants.

## Vertical Writing Support

Inline control items are laid out in logical coordinates first, then mapped to
physical coordinates through `WritingMode` and `Direction`.

### Logical Axes

- The inline axis is the axis along which inline content advances within one
  line.
- The block axis is the axis along which successive lines are stacked.
- `InlineMetricsOf<S>` always measures the logical block axis, even in vertical
  writing modes.

### Physical Mapping

For the writing modes currently modeled by layout:

| Writing Mode | Logical Inline Axis | Logical Block Axis | New Lines Stack |
| --- | --- | --- | --- |
| `HorizontalTb` | physical x | physical y | downward |
| `VerticalRl` | physical y | physical x | right to left |
| `VerticalLr` | physical y | physical x | left to right |

`Direction` affects inline-order placement within the inline axis. It must not
be used as a substitute for writing-mode block-axis mapping.

### Forced Break In Vertical Modes

In `VerticalRl` and `VerticalLr`, a forced line break:

- terminates the current vertical line at the current logical inline position;
- commits the current line using logical block-axis metrics;
- advances the next line along the logical block axis, which maps to physical x;
- keeps the control item's output size at zero;
- places the output point at the forced break insertion point after logical to
  physical mapping;
- uses the same `InlineMetricsOf<S>` aggregation rules as horizontal layout.

Vertical support must be implemented by general inline formatting context axis
mapping. It must not introduce a separate "vertical br" input type.

## Clear Semantics

`Clear` is a resolved layout-ready property by the time it reaches layout.
For a forced line break:

- `Clear::None` does not alter line placement.
- `Clear::Left`, `Clear::Right`, and `Clear::Both` participate in the same
  float exclusion and clearance model used by block flow layout.
- Clearance applies after committing the line that contains the forced break and
  before placing the following line. The forced break's zero-size output remains
  at its insertion point in the committed line; following line construction
  occurs after the relevant floats.
- In vertical writing modes, clearance must be evaluated through the layout
  engine's physical float side model, then translated back into the logical line
  origin used by the inline formatting context.

Layout must not parse legacy HTML `clear` values. It only consumes `Clear`.

## Alignment Semantics

`InlineControlAlignment` should be a layout-ready alignment value, not authored
CSS syntax. The current `VerticalAlign` model can be used initially if the
implementation plan keeps its meaning narrow and explicit.

For forced line breaks:

- `Baseline` uses `InlineMetricsOf<S>::baseline()` as the control item's
  baseline participation.
- `Top` aligns the control item's logical block-start edge with the line's
  logical block-start edge when aggregating line metrics.
- Alignment never gives the forced break a painted box, margin, padding, or
  inline advance.
- Alignment must be expressed in logical line coordinates and then mapped to
  physical coordinates with the line.

If broader CSS `vertical-align` values are added later, they should be resolved
outside layout into typed layout-ready alignment data before they reach this
model.

## Output Geometry

The output for a forced line break node is intentionally minimal:

- `size = Size::ZERO`;
- `content_size = Size::ZERO`;
- `scrollbar_size = Size::ZERO`;
- `border = Edges::ZERO`;
- `padding = Edges::ZERO`;
- `margin = Edges::ZERO`;
- `order` is the inline item order;
- `location` is the physical zero-size insertion point generated by inline line
  construction.

The line report, block size, and baseline report carry the visible layout
effect. Consumers must not infer line height from the zero-size break node.

## Intrinsic Sizing

Forced line breaks split intrinsic inline contributions:

- min-content and max-content widths are computed per line segment;
- the resulting intrinsic inline size is the maximum segment contribution;
- consecutive forced breaks create empty metric-bearing segments but do not add
  inline advance;
- text and atomic boxes before and after the break remain separate line
  segments.

This behavior should be shared by horizontal and vertical writing modes in
logical inline-size terms.

## Validation Rules

Constructors or conversion functions must enforce:

- metrics are finite, non-negative, and `baseline <= line_extent`;
- the item is scalar-consistent with the containing layout tree;
- hidden line breaks are not represented as active control items;
- the inline flow's writing mode is one of the supported `WritingMode` values;
- the control item has no children or box decorations;
- unsupported combinations return a typed error or an explicit unsupported test
  classification, not a panic in normal fixture/app paths.

Panics may remain only for internal invariant violations that cannot be
constructed through public APIs.

## Fixture Contract

Browser parity fixtures may carry layout-ready attributes needed to construct
forced line break controls:

- display state;
- direction;
- writing mode;
- resolved clear;
- resolved alignment;
- inline baseline;
- inline line extent.

These attributes are fixture data. They are not an app-facing CSS or HTML
contract. Fixture parsing must continue to require complete metric pairs and
must reject partial or non-finite metric data.

## Implementation Implications

Plans derived from this spec should generally proceed in this order:

1. Introduce the typed inline flow/control-item contract without changing
   behavior.
2. Convert existing horizontal line-break handling to use the contract.
3. Apply resolved `Clear` to forced line breaks through existing float exclusion
   machinery.
4. Implement logical-axis line construction so horizontal and vertical writing
   modes share one path.
5. Add vertical writing-mode browser parity fixtures and remove the current
   vertical unsupported bucket only after checks pass.
6. Extend mixed inline participant support only with layout-ready text or inline
   fragment inputs supplied by the appropriate integration crates.

Each step should include focused tests and should preserve the crate boundary:
layout receives normalized participants; it does not manufacture them from DOM,
CSS, or font state.
