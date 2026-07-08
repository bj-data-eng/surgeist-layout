# Surgeist Layout BR Complete Support Matrix

## Purpose

This matrix defines what complete `<br>` support means while keeping
`surgeist-layout` inside its boundary as a layout calculation engine.

Layout owns normalized layout-ready inputs, inline line construction, float and
writing-mode geometry, and output geometry. Layout does not own authored HTML
semantics, CSS parsing, style resolution, font selection, text shaping, DOM tree
classification, or root adapter orchestration.

## Boundary Rules

- `LayoutInputOf<S>::LineBreak(LineBreakInputOf<S>)` is the layout-owned node
  kind for a line break.
- `LineBreakInputOf<S>` contains only layout-ready data: display state,
  direction, writing mode, vertical alignment, clear behavior, and validated
  `InlineMetricsOf<S>`.
- `InlineMetricsOf<S>` is not a font model. It is resolved line box data in
  layout coordinates.
- Layout may reject unsupported layout-ready states explicitly. It must not
  infer authored CSS, query fonts, inspect DOM names, or call sibling crates to
  manufacture missing data.
- Browser parity XML may carry layout-ready fixture attributes, but those
  attributes are fixture data, not app-facing CSS syntax.

## Support Matrix

| Area | Browser Behavior Target | Current Layout Status | Layout-Owned Work | Non-Layout Owner |
| --- | --- | --- | --- | --- |
| Element classification | Real HTML `<br>` becomes a forced line-break participant, not a normal box. | Layout supports `LayoutInputOf::LineBreak`; retained/root integration still must classify real HTML. | Keep line-break as a separate layout input and reject box-style fallback paths. | Retained/root classify DOM nodes and construct layout inputs. |
| `display: none` | Hidden `<br>` contributes no line break and writes no visible geometry. | Supported for `LineBreakDisplay::None`. | Preserve hidden output and avoid box compute. | Style/root lower computed `display: none` to hidden line break. |
| Horizontal block-parent break | `<br>` in a supported horizontal block inline run splits the line. | Supported. | Maintain forced break participation in atomic inline layout. | Retained/root provide line-break children in block inline-run order. |
| Empty lines from consecutive breaks | Consecutive `<br>` creates line-height-bearing empty lines. | Supported through `InlineMetricsOf<S>`. | Preserve metrics contribution to baseline and descent. | Style/text/root provide metrics. |
| Inline metrics | `<br>` line height and baseline come from computed style and font metrics. | Layout consumes validated metrics; fixture generator approximates metrics for browser parity only. | Keep `InlineMetricsOf<S>` typed, scalar-generic, and layout-ready. | Style/text compute real font and line-height metrics; root adapter passes them through. |
| Mixed inline text and `<br>` | Text, atomic inline boxes, and `<br>` share one inline formatting context. | Partially supported for atomic inline boxes; general inline text is not complete in layout corpus. | Consume text as layout-ready inline participants once text integration supplies them; do not shape text here. | Text/style/root own text shaping, runs, and adapter lowering. |
| Outside block inline-run context | Browser accepts `<br>` in broader inline formatting contexts and anonymous boxes. | Browser parity currently buckets this as unsupported. | Define which layout-ready parent/run contexts can contain `LineBreakInputOf<S>` and calculate geometry for them. | Retained/root normalize DOM structure and anonymous inline/block wrappers. |
| `clear` on `<br>` | `clear` moves the break line below relevant floats. | Carried on `LineBreakInputOf<S>` but not applied to line-break layout. | Apply layout-ready `Clear` to line-break placement using existing float exclusion machinery. | Style/root lower computed `clear`. |
| Floats around line breaks | Line boxes before and after `<br>` honor active floats and clearances. | Block float exclusion exists for boxes; line-break-specific float behavior is incomplete. | Integrate forced line breaks with line placement and float exclusion without modeling CSS parsing. | Style/root provide float and clear inputs. |
| `direction` | RTL affects inline placement around the break. | Supported for current horizontal atomic inline cases. | Preserve direction-aware output placement for line-break output and following lines. | Style/root lower computed direction. |
| Horizontal `writing-mode` | `horizontal-tb` line breaks advance block direction vertically. | Supported. | Keep as the default supported writing mode. | Style/root lower computed writing mode. |
| Vertical writing modes | Vertical `<br>` advances in the relevant block/inline axes. | Explicitly unsupported; layout panics and browser parity buckets it. | Model vertical forced-break line progression in layout coordinates. | Style/root lower writing mode; text supplies vertical metrics if needed. |
| `vertical-align` | Inline alignment context affects inline-level participants around the break. | Parsed/carried; only narrow `baseline`/`top` behavior exists for atomic inline boxes. | Decide and implement the layout-ready effect of `LineBreakInputOf<S>::vertical_align()` in inline line metrics, if any. | Style/root parse and lower computed vertical-align. |
| Baseline reporting | Lines split by `<br>` report first/last baselines consistently. | Supported for current horizontal metric cases. | Maintain baseline reporting as broader inline/text contexts are added. | Text/root provide text metrics; layout reports geometry. |
| Output geometry | `<br>` has zero-size node output but participates in line construction. | Supported for current horizontal cases. | Preserve non-box zero-size output while expanding behavior. | Consumers interpret output tree. |
| Intrinsic sizing | Forced breaks split intrinsic inline contributions; widths reflect max line segment and sums per line. | Supported in atomic inline tests. | Extend only as new layout-ready inline participants are added. | Text/root provide measured inline participants. |
| Browser fixture generation | Browser-derived XML can express metric-bearing `<br>` cases. | Supported for layout-owned browser parity fixtures. | Keep XML parser strict: complete metric pairs only, fixture syntax only. | Root-owned generators/schemas coordinate with layout-ready contract. |
| Error classification | Unsupported `<br>` cases are visible and classified, not silently skipped. | Supported in browser parity report buckets for vertical and outside-context cases. | Keep unsupported buckets explicit until implemented. | Root may aggregate and prioritize cross-crate unsupported buckets. |

## Layout-Owned Completion Checklist

- [x] Separate line-break node kind from `NodeInputOf<S>`.
- [x] Carry scalar-generic inline metrics on `LineBreakInputOf<S>`.
- [x] Use metrics to create horizontal line-height-bearing breaks.
- [x] Keep line-break node output zero-size and non-box.
- [x] Parse layout-ready browser fixture metrics with complete-pair validation.
- [x] Generate browser parity fixtures for horizontal metric-bearing breaks.
- [ ] Apply `clear` semantics to line-break placement.
- [ ] Define and implement line-break behavior outside the current block
  atomic-inline-run context, using layout-ready inputs only.
- [ ] Define and implement vertical writing-mode line-break geometry.
- [ ] Decide the layout-ready meaning of `vertical-align` for line-break inputs.
- [ ] Expand fixture coverage for each newly supported layout-ready state.

## Cross-Crate Completion Checklist

- [ ] Style/text produce real `InlineMetricsOf<S>` from computed style and font
  metrics.
- [ ] Retained/root classify HTML `<br>` as `LayoutInputOf::LineBreak`.
- [ ] Root/style lower `display`, `direction`, `writing-mode`, `vertical-align`,
  and `clear` to `LineBreakInputOf<S>`.
- [ ] Text/root provide layout-ready inline text participants that can share an
  inline formatting context with `<br>`.
- [ ] Root-owned generators or fixture schemas emit complete layout-ready metric
  pairs where they own fixture production.

## Planning Notes

The next layout-owned implementation plan should probably start with `clear` on
line breaks. The input already carries `Clear`, block layout already has float
exclusion machinery, and the behavior stays squarely inside layout calculation.

Vertical writing and broader outside-context support are larger because they
touch inline formatting context modeling and axis mapping. Those should be
planned after the layout-ready parent/run model is explicit enough to avoid
turning `LineBreakInputOf<S>` into a proxy for DOM or CSS semantics.
