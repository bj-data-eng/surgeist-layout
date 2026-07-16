# FRI-04-C03 Numeric Calculation Consumption
Status: in_progress
Cycle ID: `FRI-04-C03`
Owning repository: `surgeist-layout`
Cycle base: `5c31ef95f22bc965d5af56be98f96e86300f0e83`
Reviewed specification: `plans/specs/2026-07-15-surgeist-layout-fri-04-property-specific-sizing-values.md`
at SHA-256 `e0116f0e3dd28eafabe1ed31117a61ea208e97dee986887f5600d7cbd5a06db4`, commit `5d33f3a4ab694f12985d713f7dbc74b251d55fb6`, sections `FRI-04.4 D-02`, `D-04`, `D-05`, the supported numeric rows of `D-06`, `FRI-04.6` missing-basis and calculation-status matrices, the front-door, track, and scalar evidence in `FRI-04.8`, the value and algorithm rows of `FRI-04.9`, and acceptance items 5, 7, 8, and the numeric portion of 9.

Reviewed sequence: `plans/sequences/2026-07-15-surgeist-layout-fri-04-property-specific-sizing-values.md`
at SHA-256 `9a35a5cfef82fb5b6c5abc6fd9beee7c0a080f631fd392d2ffe7694e019c4f8b`, commit `5543ef5e9273ee73c187803c79191b8b71949fc0`, entry `FRI-04-C03`.

## Outcome
Consume affine, `min()`, `max()`, and `clamp()` sizing calculations at every
current leaf, root, block, flex, grid, grid-lanes, positioned, and track call
site, with the non-negative used range, exact missing-basis behavior, and
invalid numeric errors preserved without routing a valid calculation through
`NonNumeric`.

## Boundary
The cycle starts from the published and remotely verified C02 property-field
migration. The iterative calculation evaluator is complete, but property and
track helpers still accept only one affine leaf and return `NonNumeric` for a
nested valid calculation. Current algorithms already own distinct
missing-percentage behavior that must remain at the consuming call site.

This cycle owns shared numeric property resolution and the existing numeric
consumers in leaf/root, block/positioned, flex, grid/grid-lanes, and track
sizing. A complete resolved sizing calculation is clamped to zero only after
its nested function evaluates. A non-finite intermediate or result remains an
invalid-input error. A missing basis retains each algorithm's existing
auto/indefinite/content/cyclic rule or produces `RequiredBasis` where that path
requires a definite value.

It does not implement contextual keyword routing, calc-size used-value
semantics, capability payloads, flex intrinsic-basis completion, or later
format-algorithm behavior. Those remain C04 and later initiatives. It does not
change the fixture parser, helper, serializer, HTML/XML sources, generated
reports, provenance, dependencies, features, MSRV, docs, root, or siblings.

No generation input changes are authorized and no generation command is
applicable. Scoped generation remains an optional diagnostic, never
verification evidence, but is unnecessary for this source-only cycle. The
ignored aggregate browser corpus remains the FRI-13 release gate and is not a
C03 task or final command.

## Impacts
Public API: unchanged from C02; this cycle completes behavior behind the
published calculation constructors. Dependencies, features, generated
artifacts, docs, examples, MSRV, root, and siblings: unchanged. Root migration
remains a later root-owned handoff. Safety: all owned Rust remains unsafe-free.

## Tasks
### `C03-T1` Shared Numeric Resolution And Leaf/Root Consumption
**Files:** `src/sizing.rs`, `src/compute.rs`, focused scalar/contract tests, and
`src/compute_tests.rs`, `src/leaf_tests.rs`, and `src/root_tests.rs`.
**Outcome:** Replace affine-only property calculation resolution with the full
validated sizing program, preserve exact resolution status, apply the
non-negative used range after complete evaluation, and connect every current
leaf and root preferred/minimum/maximum numeric path.
**RED:** Add tests named with the `fri04_c03_leaf_root_` prefix before
implementation. They fail because a nested valid calculation is reported as
`NonNumeric` or because a negative final value is not range-clamped. Record the
expected failures.
**Acceptance:** Both scalar lanes prove nested min/max/clamp resolution,
negative final clamping, missing basis, and overflow. Real leaf and root front
doors cover preferred/minimum/maximum calculations in both axes and in
compute-size and layout paths where applicable. Missing basis preserves the
existing path rule, invalid numeric reaches `InvalidNumeric`, and no valid
numeric calculation reaches `NonNumeric`.
**Commands:**
```sh
cargo test --locked -p surgeist-layout fri04_c03_leaf_root_
just verify
just verify-generator
```
**Current ordered range:** none.
**Dependency:** Published C02 property domains at the cycle base.
**Intended commit:** `fix(layout): consume numeric sizing calculations at leaf and root`.

### `C03-T2` Block And Positioned Numeric Consumption
**Files:** `src/block.rs`, `src/block_tests.rs`, and focused front-door tests
needed to exercise absolute-position sizing.
**Outcome:** Resolve complete preferred/minimum/maximum calculations at every
ordinary block and positioned call site while retaining each path's existing
percentage-as-auto/indefinite or required-basis rule.
**RED:** Add tests named with the `fri04_c03_block_positioned_` prefix before
implementation. They fail on nested calculations or expose the wrong missing,
negative, or invalid-numeric result. Record the expected failures.
**Acceptance:** Real block and positioned layouts cover nested min/max/clamp in
both axes, min/max constraint interaction, non-negative used values, missing
basis in intrinsic and definite-required paths, and invalid numeric errors.
Source inspection plus tests account for every block and positioned property
resolution call site without changing contextual keyword behavior.
**Commands:**
```sh
cargo test --locked -p surgeist-layout fri04_c03_block_positioned_
just verify
just verify-generator
```
**Current ordered range:** none.
**Dependency:** `C03-T1` supplies shared complete-program property resolution.
**Intended commit:** `fix(layout): consume block and positioned sizing calculations`.

### `C03-T3` Flex Numeric Consumption
**Files:** `src/flex.rs` and `src/flex_tests.rs`.
**Outcome:** Resolve complete preferred/minimum/maximum and flex-basis numeric
calculations at every current flex call site, including main/cross and
known/unknown sizing paths, while retaining the existing unresolved-percentage
content rule only at its Flexbox-owned site.
**RED:** Add tests named with the `fri04_c03_flex_` prefix before implementation.
They fail because nested property or flex-basis calculations are reported as
`NonNumeric` or silently take an intrinsic fallback. Record the expected
failures.
**Acceptance:** Real flex layouts cover nested min/max/clamp for all four
property roles, both physical axes, negative final clamping, definite and
missing main-size bases, and invalid numeric errors. A basis-dependent flex
calculation becomes content only under the existing missing-basis rule;
explicit contextual states remain for C04.
**Commands:**
```sh
cargo test --locked -p surgeist-layout fri04_c03_flex_
just verify
just verify-generator
```
**Current ordered range:** none.
**Dependency:** `C03-T1` supplies shared complete-program property resolution.
**Intended commit:** `fix(layout): consume flex sizing calculations`.
### `C03-T4` Grid, Lanes, And Track Numeric Consumption
**Files:** `src/value.rs`, `src/grid/mod.rs`, `src/grid/child.rs`,
`src/grid/lanes.rs`, `src/grid/tracks.rs`, and `src/grid_tests.rs`.
**Outcome:** Resolve complete preferred/minimum/maximum calculations throughout
ordinary grid and grid-lanes sizing, and complete numeric min/max track breadth
and track `fit-content()` limit resolution in existing track algorithms.
**RED:** Add tests named with the `fri04_c03_grid_track_` prefix before
implementation. They fail because nested grid or track calculations take a
`NonNumeric`/intrinsic fallback or because the complete track limit is not
applied. Record the expected failures.
**Acceptance:** Real grid and grid-lanes layouts cover nested property
calculations, both axes, negative final clamping, missing and definite bases,
and invalid numeric errors. Track tests cover nested fixed min/max breadths,
fit-content limits, dependency/definite classification, non-negative used
values, cyclic missing-basis handling, and invalid numeric propagation. No valid
numeric track calculation uses `NonNumeric`; intrinsic and flex tracks remain distinct.
**Commands:**
```sh
cargo test --locked -p surgeist-layout fri04_c03_grid_track_
just verify
just verify-generator
```
**Current ordered range:** none.
**Dependency:** `C03-T1` supplies shared complete-program property resolution.
**Intended commit:** `fix(layout): consume grid and track sizing calculations`.

## Cycle Acceptance
1. All four task ranges have independent clean task reviews and preserve their
   ordered compile-stable boundaries.
2. Every current leaf, root, block, flex, grid, grid-lanes, positioned, and
   track numeric sizing consumer evaluates affine/min/max/clamp programs.
3. Complete resolved preferred/minimum/maximum/flex/track calculations clamp
   negative final values to zero without clamping negative intermediates.
4. Missing-basis behavior remains property- and algorithm-specific, including
   required-basis errors, intrinsic/auto/content rules, and track cyclic rules.
5. Non-finite intermediates and results remain `InvalidNumeric`; no valid
   numeric calculation travels through `NonNumeric` or a keyword fallback.
6. Nested track breadths and fit-content limits resolve through the shared
   calculation substrate while intrinsic and flex tracks remain unchanged.
7. Contextual keywords, calc-size, capability routing, parser grammar, fixture
   inputs, generated artifacts, and later-owner algorithms remain outside the
   range.

## Final Verification
```sh
just verify
just verify-generator
just corpus-check
just taffy-check
git diff --check
rg -n --pcre2 '#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{' --glob '*.rs' .
```

The final `rg` command must report no executable unsafe match. The complete
cycle diff and artifact inventory must show no parser, helper, serializer,
HTML, XML, report, provenance, generator, dependency, feature, MSRV, docs,
root, or sibling change. Existing checked-in browser artifacts remain readable
through the read-only verification commands; `just parity-all` is not a C03
gate.

## Handoff And Blockers
The completed cycle hands C04 a remotely verified implementation in which all
ordinary numeric calculation rows are consumed explicitly and only contextual
keyword/calc-size dispatch remains. It does not emit the final FRI-04 root
handoff.

A genuine blocker exists only if a current numeric path lacks enough owning
context to preserve the reviewed missing-basis rule, or if a valid calculation
requires a new public state absent from the reviewed model. Such evidence
returns to planning review; it does not authorize generator expansion,
`NonNumeric` fallback, unsafe code, a dependency, or scope from a later
initiative.
