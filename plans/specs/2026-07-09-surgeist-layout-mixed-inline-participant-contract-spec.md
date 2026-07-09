# Surgeist Layout Mixed Inline Participant Contract Specification

## Purpose

This specification defines the layout-owned contract for mixed inline
formatting contexts that contain atomic inline boxes, forced line-break controls,
typed inline boundaries, and measured text fragments in one line-building
stream.

The contract is layout-ready by construction. `surgeist-layout` may consume
measured inline participants and calculate line geometry, baselines, intrinsic
sizes, wrapping, and logical-to-physical placement. It must not classify DOM
nodes, parse CSS, shape text, choose fonts, collapse whitespace, perform bidi
segmentation, synthesize anonymous DOM wrappers, or call sibling crates to fill
missing data.

## Ownership Boundary

Layout owns:

- typed layout-ready inline participant inputs;
- inline line construction over those participants;
- line metric aggregation and baseline reporting;
- intrinsic inline-size calculation over participant advances and forced breaks;
- logical-to-physical placement for supported writing modes;
- validation that layout-ready invariants are present and internally coherent.

Other crates own:

- DOM/retained tree classification and anonymous wrapper normalization;
- CSS parsing, cascade, inheritance, and computed style resolution;
- font selection, glyph shaping, text segmentation, whitespace collapsing, and
  bidi reordering;
- conversion from shaped text runs and style data into layout-ready measured
  inline participants;
- root-owned orchestration between retained, style, text, and layout.

Layout must reject or leave unsupported states explicit when these upstream
inputs are absent. It must not recover by deriving font metrics from CSS strings,
inspecting text content, or treating general inline DOM as block or atomic box
fallbacks.

## Current Starting Point

Current inline layout is centered on `src/inline.rs`:

- `InlineRunInput<S>` contains one ordered list of `InlineParticipant<S>`.
- `InlineParticipant<S>` currently has `Box(AtomicInlineBoxParticipant<S>)`,
  `Boundary(InlineBoundaryControlOf<S>)`, and
  `ForcedLineBreak(ForcedLineBreakControlOf<S>)`.
- `ForcedLineBreakControlOf<S>` already carries order, flow, metrics, alignment,
  and clear.
- `InlineBoundaryControlOf<S>` carries order, start/end kind, flow, metrics,
  and alignment.
- `layout_inline_run` supports horizontal and vertical inline runs containing
  atomic boxes, line-break controls, and boundary controls.
- `src/block.rs` builds atomic inline runs from inline-level box children and
  `LayoutInputOf::LineBreak` plus `LayoutInputOf::InlineBoundary`.

Current layout does not have a public or internal measured text participant
type. Browser parity support has fixture-only text measurement helpers, but
those are test support and must not become the production layout/text contract.

## Participant Categories

The mixed inline stream is a closed typed domain. The semantic categories are:

```rust
pub(crate) enum InlineParticipantOf<S: LayoutScalar = DefaultScalar> {
    AtomicBox(AtomicInlineBoxParticipantOf<S>),
    ForcedLineBreak(ForcedLineBreakControlOf<S>),
    Boundary(InlineBoundaryControlOf<S>),
    MeasuredText(MeasuredTextParticipantOf<S>),
}
```

`InlineParticipantOf<S>` is not a generic transport bag. Each variant must carry
only data needed by layout for that participant's line construction behavior.

## Atomic Box Participant Contract

Atomic inline boxes are existing layout-owned box outputs adapted into an inline
formatting context.

Required layout-ready data:

- stable order;
- border-box size;
- content size;
- margin, padding, border, and scrollbar size;
- first or last baseline participation, when available;
- resolved vertical alignment in layout-ready form.

Layout-owned behavior:

- contributes inline advance and block-axis metrics;
- participates in wrapping and intrinsic inline-size calculation;
- maps logical line placement to physical output location;
- preserves box decorations and overflow contribution already computed by the
  box layout path.

Non-goals:

- atomic boxes must not carry DOM tag names or CSS syntax;
- atomic boxes must not be used as a fallback representation for measured text;
- text that should shape with adjacent text must not be coerced into atomic
  boxes to avoid implementing mixed inline layout.

## Forced Line-Break Participant Contract

Forced line breaks are already modeled by `ForcedLineBreakControlOf<S>` and
`LayoutInputOf<S>::LineBreak(LineBreakInputOf<S>)`.

Required layout-ready data:

- stable order;
- `InlineFlowOf<S>` containing writing mode, direction, and available inline
  extent;
- validated `InlineMetricsOf<S>`;
- layout-ready alignment;
- resolved `Clear`.

Layout-owned behavior:

- terminates the current line;
- contributes line metrics and baseline data;
- creates metric-bearing empty lines when consecutive breaks occur;
- has zero output size and no box decorations;
- applies clear only from resolved `Clear`, never from HTML attributes or CSS
  strings.

Non-goals:

- no separate vertical line-break type;
- no DOM `<br>` classification in layout;
- no inference of line metrics from `font-size` or `line-height` text.

## Inline Boundary Participant Contract

Inline boundaries model layout-relevant inline wrapper start/end items. They are
not DOM nodes and they are not CSS style objects. Retained/style/root decide
which wrapper boundaries exist and provide layout-ready metrics; layout consumes
the resulting typed participants.

Required layout-ready data:

- stable order;
- start or end kind;
- `InlineFlowOf<S>` containing writing mode, direction, and available inline
  extent;
- validated `InlineMetricsOf<S>`;
- layout-ready alignment.

Public node input:

- `InlineBoundaryInputOf<S>` is the public scalar-generic layout-ready input
  for tree-shaped callers.
- `LayoutInputOf<S>::InlineBoundary(InlineBoundaryInputOf<S>)` carries boundary
  inputs into block inline-run collection.
- `InlineBoundaryInputOf<S>` requires explicit `InlineMetricsOf<S>` at
  construction and does not have a display, DOM, CSS, or text field.

Layout-owned behavior:

- contributes line metrics and baseline data;
- preserves start/end ordering in the output stream;
- has zero inline advance, zero size, and no decorations;
- does not force a line break;
- does not affect intrinsic inline-size calculations;
- participates in horizontal and vertical logical-to-physical placement;
- rejects boundary writing mode or direction that does not match the containing
  inline flow.

Non-goals:

- no anonymous wrapper synthesis in layout;
- no CSS inheritance or style propagation in layout;
- no raw text, DOM tag, selector, or font data in boundary inputs.

## Measured Text Participant Contract

Measured text participants are future layout-ready inline participants produced
outside `surgeist-layout` after style resolution and text shaping.

Required layout-ready data:

- stable order;
- logical inline advance;
- logical block-axis metrics:
  - baseline;
  - line extent;
  - after-baseline extent, either explicit or derivable from the validated pair;
- optional ink/content overflow in logical coordinates if root/text expects
  layout to include text overflow in `content_size`;
- break behavior already resolved into participant boundaries and opportunities
  that layout is allowed to consume.

Data layout must not require:

- raw text strings;
- font family names;
- font handles;
- glyph IDs;
- grapheme clusters;
- bidi levels;
- CSS `white-space`, `text-transform`, `letter-spacing`, or `word-break` syntax;
- DOM node identities beyond a stable output order or an explicit owner-provided
  output association.

Layout-owned behavior:

- treats measured text as an inline participant with advance and metrics;
- wraps only at owner-provided boundaries/opportunities;
- aggregates text metrics with atomic boxes and forced line breaks;
- places the output association point or fragment geometry if the later public
  contract requires text output nodes.

Non-goals:

- no shaping or measuring text in layout;
- no whitespace collapsing in layout;
- no bidi reordering in layout;
- no font fallback or glyph-level overflow calculation in layout.

## Remaining Measured Text Decisions

Measured text runtime work must answer these before changing Rust APIs:

1. Whether measured text participants are internal-only data supplied by root, or
   whether a public layout-ready text fragment type is needed.
2. Whether text output geometry belongs in layout outputs now, or whether root
   keeps text fragment output association outside the initial layout contract.
3. How owner-provided wrap opportunities are represented without pulling
   Unicode line breaking or CSS white-space handling into layout.
4. How scalar-generic text metrics are produced for both `f32` and `f64` layout
   lanes without narrowing.

## Verification Requirements For Measured Text Runtime Plans

Measured text runtime implementation plans should include:

- unit tests proving mixed participant line metric aggregation;
- unit tests proving forced breaks split measured text segments;
- intrinsic-size tests over text, atomic boxes, boundaries, and forced breaks;
- horizontal and vertical writing-mode tests when measured text metrics are
  layout-ready;
- tests proving layout rejects missing or invalid metrics instead of inferring
  them;
- browser parity fixtures only after root/text/style can provide complete
  layout-ready measured participants.

## Cross-Crate Requirements

Root/style/text/retained follow-up work must provide:

- retained/root classification of inline formatting contexts and anonymous
  wrappers;
- style-owned computed values for display, writing mode, direction, alignment,
  clear, and text-related properties;
- text-owned shaping and measurement into scalar-compatible logical advances
  and metrics;
- root-owned conversion into ordered layout-ready participant streams;
- root-owned fixture metadata for layout-ready inline boundaries when browser
  parity cases require anonymous or explicit wrapper start/end items;
- fixture or integration tests proving the single production path does not
  duplicate browser-parity fixture lowering.
