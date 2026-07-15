# FRI-03-C05 Flex Participation
Status: in_progress
Cycle ID: `FRI-03-C05`
Owning repository: `surgeist-layout`
Cycle base: `6f080db86a8f571ba3108771dfa49d95b46fd765`

Reviewed specification: `plans/specs/2026-07-15-surgeist-layout-fri-03-box-participation-contracts.md`
at `6ca195b4ba560ae49bc6963176234f8494cfb50a91674f6dcec358d19fa9769c`,
commit `52d87a75751f9987251ec2fdf8200e75eba3e17b`, sections `FRI-03.2`,
`E-FLEX-ORDER`, `E-FLEX-REPLACED`, the flex portions of `D-02` and `D-05`,
the flex rows and cases in `FRI-03.6`, relevant `FRI-03.8`, `FRI-03.9`, and
`FRI-03.11`, and acceptance items 2 and 6.

Reviewed sequence: `plans/sequences/2026-07-15-surgeist-layout-fri-03-box-participation-contracts.md`
at `d59317e1b80337ff4041a034c062867dc7e744048eb7047d2b2e7b412aea130a`,
commit `03e7582565fa2d4f3aa7f71973f6dfebe273c4fb`, entry `C05`.

C04 handoff: candidate `6f080db86a8f571ba3108771dfa49d95b46fd765`
was pushed to and read back from `origin/main`; local, tracking, and observed
remote `main` were equal and clean. Block/root participation is complete.

Bounded outcome: flex constructs lines from the canonical order-modified in-flow
sequence and selects the specified replaced/non-replaced automatic main minimum
without changing cross-axis stretch or source identity.

## Boundary
This cycle owns only flex consumption of the existing `ItemOrder` permutation,
the flex automatic-minimum replacedness branch, focused real-tree tests, and
read-only comparison of the four settled `fri03_order_modified_flex` XML
variants.

The exact writable allowlist is `src/flex.rs`, `src/flex_tests.rs`,
`tests/layout/browser_parity.rs`, and `src/node_input.rs` solely to remove the
obsolete `#[cfg_attr(not(test), expect(dead_code, ...))]` from the now-consumed
helper. `ItemOrder`, the helper body, and its existing exact test remain
byte-identical.

Visible in-flow flex items are reordered exactly once after collection and before
line construction. Negative, zero, positive, and equal values use the existing
stable `(ItemOrder, SourceIndex)` helper. Reverse flex direction changes physical
main progression only; it does not reverse the order-modified sequence again.
Hidden and absolute children remain excluded, and output storage keeps each
source sibling's `SourceIndex`.

For automatic main minimum with a transferred suggestion, replaced items select
the smaller content/transferred suggestion and non-replaced items select the
larger. Existing authored/max caps, explicit minimums, overflow-zero behavior,
padding/border floor, aspect-ratio transfer, flexing, and cross-axis stretch stay
independent. This does not globally exempt replaced flex items from stretch.

No generator command or browser capture is permitted. Parser grammar, HTML, XML,
reports, corpus metadata, provenance, and generator source remain byte-identical
to the base. `verify-generator` runs generator-feature check, test, and Clippy
verification without generation or capture; `corpus-check` invokes read-only
`check-corpus`. The ignored aggregate parity test remains visible and unclaimed.

Base evidence: `collect_items` in `src/flex.rs` returns visible in-flow items in
source order and `collect_flex_lines` consumes that vector directly.
`automatic_min_main_size` always takes `max(content, transferred)`. A read-only
filtered aggregate diagnostic for `fri03_order_modified_flex` reports exactly
four `x` mismatches: the first LTR child expected 40 and got 0, while the first
RTL child expected 20 and got 60. That diagnostic is RED context, not final
verification evidence.

Impacts: API - unchanged; dependencies/features and MSRV - unchanged; artifacts
- unchanged and read-only; docs/examples - unchanged; root follow-up - deferred
to C08; unsafe - none.

## Task
### C05-T1 - Enforce Flex Order And Replaced Automatic Minimum
Files: `src/flex.rs`, `src/flex_tests.rs`, `tests/layout/browser_parity.rs`, and
only the obsolete expectation on `item_order_permutation` in `src/node_input.rs`.
Dependencies: published/read-back C04 base and the existing C01 permutation.

Outcome: use `item_order_permutation` to order collected visible in-flow flex
items before any line construction, then carry source identity unchanged through
resolution and final output. Select `min(content, transferred)` only for replaced
automatic minimum and retain `max(content, transferred)` for ordinary items.
Remove the helper's fulfilled-by-C05 dead-code expectation without changing it.

RED: wrapped row/row-reverse geometry follows source order; all four settled
browser variants fail their first child's `x`; and a replaced aspect-ratio item
with content suggestion 90 and transferred suggestion 40 retains the ordinary
90 automatic minimum instead of shrinking within a 60-unit container.

Acceptance:
- both scalar lanes prove signed order, equal-order source ties, wrapping, and
  row-reverse progression use one order-modified sequence before lines;
- each final child keeps its enumerated `SourceIndex`, and hidden/absolute output
  scheduling remains outside the in-flow permutation;
- all four settled flex-order XML variants run nonignored, retain exact topology,
  and match without HTML or XML changes;
- paired aspect-ratio items with content suggestion 90 and transferred
  suggestion 40 finish at widths 60 and 90 in a 60-unit container for replaced
  and non-replaced roles respectively, while both retain the same cross-axis
  stretch;
- existing authored/max caps, explicit minimums, overflow-zero behavior,
  padding/border floor, and non-replaced transferred-size coverage remain green;
- the `src/node_input.rs` diff contains only removal of the obsolete expectation,
  while the helper body and its exact test remain unchanged; and
- no ordinary-grid, grid-lanes, public API, parser, fixture, or generator branch
  changes.

Exact library tests:
- `flex_tests::flex_order_modified_sequence_precedes_wrapping_and_preserves_source_identity_in_both_scalar_lanes`
- `flex_tests::flex_replaced_automatic_minimum_selects_smaller_suggestion_and_preserves_cross_stretch_in_both_scalar_lanes`
- `flex_tests::flex_row_stretched_aspect_ratio_item_does_not_shrink_below_transferred_size`
- `node_input::tests::item_order_permutation_is_signed_total_and_stable`

Exact integration test:
`layout::browser_parity::flex_item_order_variants_match_browser`.

For each library name, run inventory and exact execution separately:
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --lib -- --list | rg -x 'TEST_NAME: test'
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --lib TEST_NAME -- --exact
```
For the integration name:
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout -- --list | rg -x 'layout::browser_parity::flex_item_order_variants_match_browser: test'
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout layout::browser_parity::flex_item_order_variants_match_browser -- --exact
```

Task gates: `CARGO_NET_OFFLINE=true just fmt-check`;
`CARGO_NET_OFFLINE=true just verify`; `CARGO_NET_OFFLINE=true just verify-generator`;
`CARGO_NET_OFFLINE=true just corpus-check`; strict locked Clippy; rustdoc with
warnings denied; the exact repository-wide unsafe scan below; `git diff --check`;
exact `src/node_input.rs` diff inspection; byte identity for every other `src`
and `tests` path and protected artifact against the cycle base; clean status. No
generation or browser capture. The node-input diff fails this gate if it contains
any hunk other than deletion of the obsolete expectation.

Task unsafe gate:
```sh
! rg -n --pcre2 '#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{' --glob '*.rs' --glob '!target/**' .
```

Commit: `layout: enforce flex participation`.

## Completion
```sh
CARGO_NET_OFFLINE=true just fmt-check
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
CARGO_NET_OFFLINE=true RUSTDOCFLAGS='-D warnings' cargo doc --locked --offline --no-deps -p surgeist-layout
rg -n 'item_order_permutation|item_is_replaced' src/node_input.rs src/flex.rs
! rg -n --pcre2 '#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{' --glob '*.rs' --glob '!target/**' .
git diff --check
git diff --unified=0 6f080db86a8f571ba3108771dfa49d95b46fd765 -- src/node_input.rs
git diff --exit-code 6f080db86a8f571ba3108771dfa49d95b46fd765 -- src ':(exclude)src/flex.rs' ':(exclude)src/flex_tests.rs' ':(exclude)src/node_input.rs'
git diff --exit-code 6f080db86a8f571ba3108771dfa49d95b46fd765 -- tests ':(exclude)tests/layout/browser_parity.rs'
git diff --exit-code 6f080db86a8f571ba3108771dfa49d95b46fd765 -- Cargo.toml Cargo.lock Justfile README.md scripts tests/bin/surgeist-layout-generate.rs tests/bin/surgeist-layout-generate tests/layout/browser_parity/support.rs tests/layout/browser_parity/html tests/layout/browser_parity/xml tests/layout/browser_parity/corpus.toml tests/layout/browser_parity/README.md tests/layout/browser_parity/scripts tests/layout/browser_parity/generation-reports
test -z "$(git status --porcelain)"
```

Cycle acceptance: the task range is independently `CLEAN`; the complete cycle
range is holistic `CLEAN`; final commands pass on local `main`; the immutable
candidate is pushed to authority `origin/main`; a fresh fetch and remote query
prove local `HEAD`, local `main`, `origin/main`, and observed remote `main`
agree; and C06 receives the published SHA plus evidence that only C05-owned flex
behavior changed.

Genuine blockers are limited to unavailable required tooling without authorized
acquisition, unowned dirty state, contradictory reviewed requirements, or a
required unsafe/ownership violation. A failing test or review finding returns
this plan to `in_progress` and is corrected inside C05.
