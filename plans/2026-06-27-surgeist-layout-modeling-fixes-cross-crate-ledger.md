# Surgeist Layout Modeling Fixes Cross-Crate Ledger

This ledger accompanies
`plans/2026-06-26-surgeist-layout-modeling-fixes-implementation.md`.

Use it to record requirements discovered while implementing layout-local tasks
that require follow-up in another Surgeist crate or the root integration
workspace. These entries are coordination handoffs, not reasons to silently
weaken the layout model. They let layout proceed with reviewed crate-local
changes while preserving the missing cross-crate work for the root coordinator.

Do not edit sibling crates from this repo. When a dependency is discovered:

- Record the layout task and local commit or pending state.
- Name the owning crate/repo.
- Link the GitHub issue or provide a complete issue draft.
- Record the exact verification that remains pending in this crate.
- Revisit the entry when the owning crate lands its change.

## Entry Status

- `open`: Owning crate work is not yet available to this repo.
- `ready-to-retest`: Owning crate work is available locally; rerun pending
  layout verification.
- `closed`: Layout verification passed and the local task no longer depends on
  the cross-crate handoff.

## Entries

### LAYOUT-XCRATE-0001: Style Adapter Must Lower Validated Aspect Ratio

- Status: `open`
- Layout task: Task 3, `Model Aspect Ratio As A Validated Semantic Type`
- Layout commit: `fb04e1e0` (`Model aspect ratio as a validated value`)
- Layout state: implemented locally, clean-context reviewed, and committed;
  focused tests are pending because the style dev-dependency does not yet
  compile against the new layout API.
- Owning crate: `surgeist-style`
- Owning issue: https://github.com/bj-data-eng/surgeist-style/issues/2
- Required owning change: update
  `surgeist-style/src/adapters/layout.rs` so its layout adapter assigns
  `surgeist_layout::NodeInput::aspect_ratio` as
  `Option<surgeist_layout::AspectRatio>` instead of `Option<f32>`, using
  `surgeist_layout::AspectRatio::new(...)` or an equivalent validated path.
- Observed failure:

```text
error[E0308]: mismatched types
   --> /Users/codex/Development/surgeist-style/src/adapters/layout.rs:156:23
    |
156 |         aspect_ratio: aspect_ratio(resolved),
    |                       ^^^^^^^^^^^^^^^^^^^^^^ expected `Option<AspectRatio>`, found `Option<f32>`
```

- Pending layout verification:

```sh
cargo test -p surgeist-layout tests::aspect_ratio_rejects_non_positive_or_non_finite_values -- --nocapture
cargo test -p surgeist-layout --test layout layout::leaf -- --nocapture
cargo test -p surgeist-layout --test layout layout::block -- --nocapture
cargo test -p surgeist-layout --test layout layout::flex -- --nocapture
cargo test -p surgeist-layout --test layout layout::grid -- --nocapture
```

- Notes:
  - Layout-local checks passed before this entry was opened:
    `cargo check -p surgeist-layout --lib`, `cargo fmt --check`, and
    `git diff --check`.
  - Task 3 reviewers returned clean for the layout-side model.
