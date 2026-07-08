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

- Status: completed in layout for browser parity fixtures; pending root coordination for any root-owned generated schema/artifacts.
- Required contract: emit complete layout-ready metric pairs for `<br>` fixtures when checking font-sensitive browser parity.
- Must not: emit partial metric pairs or root-private schema fields layout cannot parse.
