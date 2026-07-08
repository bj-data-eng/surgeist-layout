# Surgeist Layout Inline Control Item Sequencing Plan

## Purpose

This document sequences the layout-owned work needed to implement the inline
control item model described in
`plans/specs/2026-07-08-surgeist-layout-inline-control-item-spec.md`.

It is not an implementation plan. Each phase below should become its own
focused implementation plan before code changes begin. The sequence is designed
to keep reviews small, preserve the layout crate boundary, and avoid locking the
implementation to today's ad hoc atomic-inline shape.

## Current Code Starting Point

Relevant current files:

- `src/node_input.rs`
  - Owns `InlineMetricsOf<S>`, `LineBreakInputOf<S>`, `WritingMode`,
    `VerticalAlign`, `Clear`, and `LayoutInputOf<S>::LineBreak`.
  - `LineBreakInputOf<S>` already carries display, direction, writing mode,
    vertical alignment, clear, and metrics.
- `src/inline.rs`
  - Owns private atomic inline layout.
  - `AtomicInlineItem<S>` already has `Box` and `ForcedLineBreak` variants.
  - `layout_atomic_inline_items` supports horizontal forced breaks.
  - `layout_vertical_rl_atomic_inline_items` currently supports vertical boxes
    only and rejects forced breaks with an internal unreachable path.
  - `VerticalLr` is not separately handled by the atomic-inline path.
- `src/block.rs`
  - Builds atomic inline runs from block children.
  - Converts `LayoutInputOf::LineBreak` to `AtomicInlineItem::forced_line_break`.
  - Panics when a line-break input has a non-horizontal writing mode.
  - Carries float exclusion machinery for block flow, but does not apply
    `LineBreakInputOf<S>::clear()` to forced line breaks.
- `tests/layout/browser_parity/support.rs`
  - Parses layout-ready line-break fixture attributes including writing mode,
    vertical align, clear, and inline metrics.
- `tests/layout/browser_parity/scripts/gentest/test_helper.js`
  - Currently classifies some vertical `<br>` cases as unsupported.
- `src/inline_tests.rs` and `src/block_tests.rs`
  - Cover horizontal forced break behavior, baseline contribution, empty lines,
    intrinsic splits, and current vertical unsupported behavior.

## Non-Negotiable Boundaries

- Layout must consume layout-ready inline participants only.
- Layout must not parse HTML, authored CSS, legacy `clear` attributes, fonts,
  text shaping, anonymous DOM wrappers, or retained tree semantics.
- `LineBreakInputOf<S>` remains the layout input for a line-break node.
- Any broader inline control model must make invalid states hard to construct.
- Vertical writing support must be a logical-axis line layout feature, not a
  separate vertical `<br>` special case.
- No compatibility aliases or extra lowering layers should be added.

## Phase 1: Name The Inline Control Contract Internally

### Goal

Make the existing private forced-break behavior explicit as an internal
layout-ready control item model without changing observable behavior.

### Scope

- Introduce internal types equivalent to:
  - `InlineFlowOf<S>`
  - `ForcedLineBreakControlOf<S>`
  - `InlineControlItemOf<S>`
  - a narrow alignment type or explicit mapping from current `VerticalAlign`
- Keep these types private or crate-internal unless a later plan proves public
  exposure is needed.
- Preserve the existing `LineBreakInputOf<S>` public API.
- Convert `AtomicInlineItem::ForcedLineBreak { order, metrics }` to carry a
  forced-break control payload or be constructed from one.

### Why First

The current forced break is represented as an enum payload inside
`AtomicInlineItem`, but it does not carry flow, clear, or alignment. Naming the
control contract first creates the typed surface that later phases can extend
without repeatedly widening ad hoc tuples.

### Verification Gate

- Existing horizontal line-break tests still pass unchanged.
- New focused tests prove the control construction preserves order, metrics,
  writing mode, direction, clear, and alignment without changing layout output.
- Review confirms the model is not a generic transport bag.

### Resulting Implementation Plan

Create a focused plan for internal model extraction and no-behavior-change
conversion.

## Phase 2: Route Block Line Breaks Through The Control Contract

### Goal

Make `src/block.rs` construct inline control items through one named conversion
path from `LineBreakInputOf<S>`.

### Scope

- Replace direct calls to `AtomicInlineItem::forced_line_break(order, metrics)`
  with a conversion from `LineBreakInputOf<S>` plus the run flow context.
- Keep hidden line breaks skipped before active control construction.
- Preserve zero-size output for line-break nodes.
- Preserve existing horizontal behavior and existing vertical unsupported
  classification.
- Centralize the rule that a line-break node has no box decoration, margin,
  padding, border, scrollbars, or children.

### Why Second

This is the bridge between public `LayoutInputOf::LineBreak` and the private
inline engine. Once the conversion is named, later clear and vertical phases can
modify the control semantics without adding scattered matches in `block.rs`.

### Verification Gate

- Existing `src/block_tests.rs` line-break tests pass.
- New tests assert hidden line breaks do not create active controls.
- Review confirms `NodeInputOf<S>` is not reused or widened for line-break
  state.

### Resulting Implementation Plan

Create a focused plan for block-to-inline control conversion and output
preservation.

## Phase 3: Apply Resolved Clear To Forced Line Breaks

### Goal

Implement layout-owned `Clear` behavior for line-break controls in horizontal
block flow.

### Scope

- Use `LineBreakInputOf<S>::clear()` through the control item contract.
- Apply clearance using the existing float exclusion model in `src/block.rs`.
- Preserve `Clear::None` behavior.
- Add tests for `Clear::Left`, `Clear::Right`, and `Clear::Both` around floats.
- Keep presentational hint parsing and CSS lowering out of layout.
- Keep vertical clear either unsupported with explicit classification or routed
  through the same logical-axis abstraction only if that abstraction already
  exists.

### Why Third

`clear` is already present on `LineBreakInputOf<S>`, and WebKit treats
line-break boxes as float clearers. This phase closes the most direct correctness
gap without needing the larger vertical axis rewrite.

### Verification Gate

- Focused block/float tests prove line-break clearance moves the affected line
  below relevant floats.
- Browser parity fixture support can express the cases with layout-ready clear
  values.
- Review confirms no HTML `clear` parsing or style lowering appears in layout.

### Resulting Implementation Plan

Create a focused plan for horizontal line-break clear semantics and fixtures.

## Phase 4: Extract Logical Inline Axes

### Goal

Make inline line construction operate in logical inline/block coordinates before
physical placement.

### Scope

- Introduce a small internal axis-mapping helper for inline formatting contexts.
- Model:
  - logical inline advance;
  - logical block line stacking;
  - physical x/y placement for `HorizontalTb`, `VerticalRl`, and `VerticalLr`;
  - direction along the inline axis.
- Convert horizontal layout to use the helper without changing output.
- Keep text shaping and inline text runs out of scope.

### Why Fourth

Vertical `<br>` should not be bolted onto the current horizontal path. The axis
model must exist first so forced breaks, boxes, baselines, and output locations
share one coordinate story.

### Verification Gate

- Existing horizontal inline and block tests pass unchanged.
- New axis-mapping tests cover all current `WritingMode` variants.
- Review confirms `Direction` is not used as a substitute for writing-mode
  block-axis mapping.

### Resulting Implementation Plan

Create a focused plan for logical axis extraction with no intended behavior
change for horizontal cases.

## Phase 5: Support Vertical Forced Breaks In Inline Layout

### Goal

Allow forced line-break controls in vertical inline layout using the logical-axis
model.

### Scope

- Remove the current forced-break rejection in the vertical atomic-inline path.
- Support both `VerticalRl` and `VerticalLr`.
- Commit vertical lines using `InlineMetricsOf<S>` in logical block-axis terms.
- Place zero-size line-break outputs at the mapped insertion point.
- Preserve baseline reporting in the logical/physical convention already used
  by layout outputs.
- Keep the input type as `LineBreakInputOf<S>`; do not introduce a vertical line
  break type.

### Why Fifth

This is the first behavior expansion that depends on the axis model. Doing it
after Phase 4 prevents vertical support from becoming a special case that later
has to be unwound.

### Verification Gate

- Inline unit tests cover vertical forced breaks, consecutive vertical breaks,
  vertical break with atomic boxes, and zero-size break output.
- Block tests prove a vertical `LineBreakInputOf<S>` no longer panics when the
  parent writing mode supports it.
- Review confirms metrics remain layout-ready and are not derived from font
  data.

### Resulting Implementation Plan

Create a focused plan for vertical forced-break behavior in inline and block
integration.

## Phase 6: Expand Browser Parity Fixtures For Vertical Breaks

### Goal

Move vertical `<br>` fixture cases from unsupported classification into checked
browser parity where the layout-owned contract is sufficient.

### Scope

- Add or enable vertical writing-mode `<br>` fixture cases with complete
  layout-ready metric pairs.
- Regenerate XML using the documented fixture generator.
- Keep cases unsupported when they require unavailable cross-crate text,
  retained, or anonymous-wrapper integration.
- Do not hand-edit generated XML.

### Why Sixth

The generator and parser already understand layout-ready writing-mode and
metrics. Browser parity should be enabled only after the engine behavior exists,
so generated expectations validate behavior rather than define speculative
semantics.

### Verification Gate

- Fixture generation succeeds.
- Green-baseline XML corpus passes for newly supported vertical `<br>` cases.
- Unsupported buckets shrink only for cases now supported by layout-owned
  behavior.

### Resulting Implementation Plan

Create a focused plan for vertical browser parity fixture enablement.

## Phase 7: Define Mixed Inline Participant Contract

### Goal

Prepare layout to accept text and other inline fragments beside boxes and forced
breaks without owning text shaping.

### Scope

- Define layout-ready participant categories that can share an inline formatting
  context:
  - atomic boxes;
  - forced line-break controls;
  - future measured text fragments;
  - future inline fragment boundaries if required.
- Decide whether this belongs in a new internal inline module split before it
  becomes public API.
- Do not implement text shaping, font lookup, or style adapters.
- Record any cross-crate requirements in the existing ledger before execution.

### Why Seventh

This is larger than `<br>` alone. It is necessary for full browser-like inline
formatting, but it should not block vertical forced-break correctness when the
current atomic-inline model can still carry boxes and breaks.

### Verification Gate

- A written spec or implementation plan identifies which data layout needs from
  text/style/root and which data remains out of scope.
- Review confirms the contract does not recreate DOM or CSS semantics inside
  layout.

### Resulting Implementation Plan

Create a focused planning/spec task before any code implementation.

## Phase 8: Revisit Public API Exposure

### Goal

Decide whether any inline control types should become public, or whether
`LineBreakInputOf<S>` remains the only public layout-ready control surface for
now.

### Scope

- Review root/style/retained integration needs after prior phases.
- Prefer keeping implementation types private unless root needs to construct a
  richer inline participant stream directly.
- If public exposure is needed, expose invariant-preserving constructors and
  avoid public structs with combinable invalid states.

### Why Last

Public API should follow proven internal semantics. The project does not require
backwards compatibility yet, but public surface still becomes coordination
surface for other crates, so it should be intentional.

### Verification Gate

- Modeling review confirms the public/private boundary follows
  `guidance/surgeist-rust-modeling-guide.md`.
- Cross-crate ledger entries are updated for any root/style/retained follow-up.

### Resulting Implementation Plan

Create a focused public API review/update plan only if earlier phases prove it
is needed.

## Recommended First Implementation Plan

Start with Phases 1 and 2 together only if the implementation diff stays small:

1. internal inline control item model;
2. block-to-inline conversion through that model;
3. no behavior change.

If the first plan grows beyond a single clean reviewer pass, split Phase 1 and
Phase 2 into separate plans.

After that, implement Phase 3 as its own plan. It is the smallest behavior
change with clear correctness value.

Vertical work should not begin until the logical-axis extraction plan is written
and reviewed against the current `src/inline.rs` vertical path.

## Required Review Shape For Derived Plans

Each implementation plan derived from this sequencing document should use the
crate coordinator workflow from `AGENTS.md`:

- one scoped worker task at a time;
- separate reviewer for each worker result;
- logical commits after clean scoped review;
- final holistic reviewer that checks the code against the plan, the inline
  control item spec, and `guidance/surgeist-rust-modeling-guide.md`;
- final checks:

```sh
cargo test -p surgeist-layout
cargo clippy -p surgeist-layout --all-targets -- -D warnings
cargo fmt --check
git diff --check
git status --short --branch
```

Browser parity or generator commands should be added to the derived plan when
that plan touches fixture inputs, generated XML, parser support, or unsupported
bucket classification.

## Cross-Crate Ledger Expectations

Derived plans should log, but not block on, requirements owned by other crates:

- root/retained classification of real HTML `<br>` nodes;
- style lowering of computed `display`, `direction`, `writing-mode`,
  `vertical-align`, and `clear`;
- text/style production of real `InlineMetricsOf<S>`;
- root adapter decisions about whether a future inline participant stream is
  constructed directly or through retained tree children.

Layout implementation should proceed only for behavior expressible through
layout-ready inputs already present in this crate or deliberately introduced by
a reviewed layout plan.
