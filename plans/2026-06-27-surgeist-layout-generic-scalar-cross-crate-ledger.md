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

- Status: `open`
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
