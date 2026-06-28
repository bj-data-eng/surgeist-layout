# Surgeist Layout Generic Scalar Cross-Crate Ledger

This ledger records follow-up requirements discovered while implementing
`plans/2026-06-27-surgeist-layout-generic-scalar-implementation.md`.

Layout workers must not edit sibling crates from this repo. When generic scalar
support exposes work needed in another crate, record it here with enough detail
for the owning crate coordinator to plan and implement the change.

## Entry Status

- `open`: Confirmed cross-crate work remains.
- `reported`: The owning crate or root coordinator has been informed.
- `resolved`: The owning crate has landed the needed change and layout has
  verified against it.

## Entries

### `surgeist-style` declaration insertion visibility blocks layout integration test compile

- Status: `resolved`
- Owning crate: `surgeist-style`
- Affected API: `DeclarationBlock::insert`
- Observed command:
  `cargo test -p surgeist-layout --test layout layout::contract::layout_scalar_supports_f32_and_f64 -- --nocapture`
- Observed behavior: the `surgeist-layout` integration test target fails to
  compile because `tests/layout/browser_parity/support.rs` calls
  `declarations.insert(...)`, while `surgeist-style/src/declaration.rs` defines
  `DeclarationBlock::insert` as a private method.
- Expected behavior: layout browser parity support should have a public,
  supported way to construct declaration blocks for fixture lowering, or the
  support code should use the public replacement API chosen by
  `surgeist-style`.
- Required owning change: in `surgeist-style`, expose an intentional public
  insertion/builder path for declaration block construction or coordinate the
  replacement API with layout test support. Then rerun the layout integration
  test target from this crate.
- Resolution: `surgeist-layout` browser parity support now constructs
  declarations through the public fallible `surgeist-style`
  `Declarations::try_insert` API, and the layout integration verification in
  `plans/2026-06-27-surgeist-layout-style-fixture-integration.md` passed.

### `surgeist-style` grid and calc value API changes block browser parity support compile

- Status: `resolved`
- Owning crate: `surgeist-style`
- Affected APIs: grid line/name value constructors, subgrid/track repeat value
  constructors, and `CalcLength::sum`
- Observed commands:
  `cargo test -p surgeist-layout --test layout layout::contract::node_input_and_output_support_f64_scalar_lane -- --nocapture`
  and other focused `--test layout layout::contract::*` checks during generic
  scalar implementation.
- Observed behavior: after the layout-local scalar types compile, the
  `surgeist-layout` integration test target still fails in
  `tests/layout/browser_parity/support.rs` because the support code is written
  against older `surgeist-style` construction shapes. Repeated observed errors
  include `GridLineNameSet` versus `Vec<String>` mismatches,
  `SubgridTrack`/`TrackRepeat` result handling mismatches, grid line newtype
  construction mismatches, and `CalcLength::sum` now requiring separate
  `(first, rest)` arguments rather than a single iterable.
- Expected behavior: browser parity support should compile against the current
  intentional `surgeist-style` public value construction APIs before layout
  contract tests are blocked by adapter support code.
- Required owning change: in `surgeist-style` or in a coordinated layout
  support update after style publishes the intended APIs, provide a stable
  construction path for the browser parity lowering support and update the
  layout test-support call sites. Then rerun the layout integration test target
  from this crate.
- Resolution: `surgeist-layout` browser parity support now uses the intentional
  public `surgeist-style` constructors for grid line names, subgrid line-name
  repeats, grid lines, track repeats, and non-empty calc sums, and the layout
  integration verification in
  `plans/2026-06-27-surgeist-layout-style-fixture-integration.md` passed.
