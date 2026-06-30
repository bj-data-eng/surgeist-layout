# Surgeist Layout BR Line Break Cross-Crate Ledger

This ledger records follow-up work outside `surgeist-layout` discovered while
implementing `plans/2026-06-29-surgeist-layout-br-line-break-implementation.md`.

## Entries

### HTML/style adapter needs to lower real `<br>` to `LayoutInput::LineBreak`

- Status: `open`
- Owning crate: root `surgeist` or future HTML/DOM adapter crate
- Affected API: `surgeist_layout::LayoutInputOf::LineBreak`
- Observed behavior: layout browser parity can lower `source-tag="br"` fixture
  metadata after style resolution, but production HTML tree construction outside
  this crate still needs to map real HTML `<br>` elements to layout input.
- Expected behavior: the real adapter should preserve normal style resolution
  for the element, then construct `LayoutInput::LineBreak(LineBreakInput)`.
  `display: none` should map to hidden line-break input.
- Required owning change: add a root or adapter implementation plan after this
  layout API lands. Do not implement that adapter from the layout crate project.
- Verification note: layout owns this repo's HTML/browser-parity corpus. The
  generator now separates remaining `<br>` cases into explicit vertical and
  outside-block unsupported buckets. At this commit, the checked corpus does not
  contain a generated supported `source-tag="br"` XML fixture because existing
  `<br>` source fixtures compute as vertical or as non-block-parent contexts
  after base CSS. Separate root/retained integration checks that exercise real
  application tree nodes are expected to remain blocked until this owning change
  lands.
