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
- Verification note: layout owns this repo's HTML/browser-parity corpus, which
  now includes generated supported `source-tag="br"` XML fixtures for
  layout-ready horizontal and constrained vertical cases. The generator still
  keeps remaining `<br>` cases in explicit unsupported buckets when their
  surrounding layout-ready contracts are not available. Separate root/retained
  integration checks that exercise real application tree nodes are expected to
  remain blocked until this owning change lands.

### Text/root need to provide layout-ready measured inline participants

- Status: `open`
- Owning crate: root `surgeist` plus `surgeist-text`
- Affected API: future mixed inline participant contract in `surgeist-layout`
- Observed behavior: layout can combine atomic inline boxes and forced
  line-break controls, but it does not yet have production layout-ready measured
  text participants that can share the same inline formatting context.
- Expected behavior: text/root should provide ordered measured text fragments
  with scalar-compatible logical advance, baseline, line extent, and any
  owner-approved wrap opportunities. Layout should consume those values without
  shaping text, parsing CSS, choosing fonts, or inspecting raw text.
- Required owning change: root should coordinate text/style/retained plans after
  the layout mixed inline participant contract is accepted.
- Verification note: layout-side runtime work should stay blocked on this
  contract until root/text can produce complete layout-ready participant data.

### Retained/root need to normalize mixed inline formatting contexts

- Status: `open`
- Owning crate: root `surgeist` plus retained tree integration
- Affected API: future mixed inline participant stream consumed by
  `surgeist-layout`
- Observed behavior: layout browser parity can test constrained inline runs, but
  production app trees still need root/retained ownership for inline formatting
  context boundaries, anonymous wrappers, and output association.
- Expected behavior: retained/root should present layout with normalized
  ordered inline participant streams, including atomic boxes, line-break
  controls, and measured text fragments, without requiring layout to inspect DOM
  structure.
- Required owning change: root should create integration plans once layout
  defines whether inline fragment boundaries are needed by the layout API.
- Verification note: layout must not implement fallback DOM normalization to
  unblock these cases locally.
