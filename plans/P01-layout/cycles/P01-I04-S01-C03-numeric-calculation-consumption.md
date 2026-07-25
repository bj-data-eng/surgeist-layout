# P01-I04-S01-C03 Numeric Calculation Consumption
Status: complete
Cycle ID: `P01/I04/S01/C03`
Owning repository: `surgeist-layout`
Cycle base: `601f4ad4700827465096dc62c029b4d8147336b8593b6b80ae7abfb09fc22577`
Reviewed specification: `plans/P01-layout/initiatives/P01-I04-property-specific-sizing-values.md`
at SHA-256 `49ede2ba2672a91f99ba193651dbb1350ede7b80`, commit `5d33f3a4ab694f12985d713f7dbc74b251d55fb6`, sections `FRI-04.4 D-02`, `D-04`, `D-05`, the supported numeric rows of `D-06`, `FRI-04.6` missing-basis and calculation-status matrices, the front-door, track, and scalar evidence in `FRI-04.8`, the value and algorithm rows of `FRI-04.9`, and acceptance items 5, 7, 8, and the numeric portion of 9.

Reviewed sequence: `plans/P01-layout/sequences/P01-I04-S01-property-specific-sizing-values.md`
at SHA-256 `2e006d30b0250c526e10bba13a37e58e111ff60791b34f8d7c2e4d0e527db13f`, commit `0a666f8f698703cd7979194a7f75f834e4c9b522`, entry `P01/I04/S01/C03`.

## 1 Outcome
Consume affine, `min()`, `max()`, and `clamp()` sizing calculations at every
current leaf, root, block, flex, grid, grid-lanes, positioned, and track call
site, with the non-negative used range, exact missing-basis behavior, and
invalid numeric errors preserved without routing a valid calculation through
`NonNumeric`.

## 2 Boundary
The cycle starts from the published and remotely verified C02 property-field
migration. Task-clean T01 replaces affine-only property resolution with the
full program. Because that shared helper reaches downstream consumers, T02-T04
own persistent front-door characterization and change production only if a
test exposes a call-site defect. Each algorithm's distinct missing-percentage
behavior remains at its consuming call site; passing behavior is not forced red.

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

## 3 Impacts
Public API: unchanged from C02; this cycle completes behavior behind the
published calculation constructors. Dependencies, features, generated
artifacts, docs, examples, MSRV, root, and siblings: unchanged. Root migration
remains a later root-owned handoff. Safety: all owned Rust remains unsafe-free.

## 4 Tasks
### 4.1 `P01/I04/S01/C03/T01` Shared Numeric Resolution And Leaf/Root Consumption
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
**Current ordered range:** `ba8358e659dd8b546ffd93dd60f40d7dccec6909..c65f4d201696664cdfd220434af1f8d9e93adce7`.
**Dependency:** Published C02 property domains at the cycle base.
**Intended commit:** `fix(layout): consume numeric sizing calculations at leaf and root`.

### 4.2 `P01/I04/S01/C03/T02` Block And Positioned Numeric Evidence
**Files:** `src/block.rs`, `src/block_tests.rs`, and focused front-door tests
needed to exercise absolute-position sizing.
**Outcome:** Persist evidence that every block and positioned call site consumes
complete preferred/minimum/maximum calculations while retaining its existing
percentage-as-auto/indefinite or required-basis rule.
**RED:** Not applicable when T01 already supplies correct behavior. Add tests
named with the `fri04_c03_block_positioned_` prefix first and record their
characterization result; any substantive failure is the RED for a focused fix.
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
**Current ordered range:** `ec549a65dc182c6054b728b447ba1a6720124696..2ee3308ad7d4b888bd8911db2bb4cf8bb736319d`.
**Dependency:** `T01` supplies shared complete-program property resolution.
**Intended commit:** `test(layout): cover block and positioned sizing calculations`.

### 4.3 `P01/I04/S01/C03/T03` Flex Numeric Evidence
**Files:** `src/flex.rs` and `src/flex_tests.rs`.
**Outcome:** Persist evidence that every current flex call site consumes complete
preferred/minimum/maximum and flex-basis calculations across main/cross and
known/unknown paths while retaining the Flexbox-owned missing-basis rule.
**RED:** Not applicable when T01 already supplies correct behavior. Add tests
named with the `fri04_c03_flex_` prefix first and record their characterization
result; any substantive failure is the RED for a focused fix.
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
**Current ordered range:** `2ee3308ad7d4b888bd8911db2bb4cf8bb736319d..c36fed92cbcc8e180818681cb8b1fb23084e7ae3`.
**Dependency:** `T01` supplies shared complete-program property resolution.
**Intended commit:** `test(layout): cover flex sizing calculations`.
### 4.4 `P01/I04/S01/C03/T04` Grid/Lanes Evidence And Track Numeric Correction
**Files:** `src/value.rs`, `src/grid/mod.rs`, `src/grid/child.rs`,
`src/grid/lanes.rs`, `src/grid/tracks.rs`, and `src/grid_tests.rs`.
**Outcome:** Persist grid/grid-lanes property evidence and replace every track
classification and runtime consumer's affine-only coefficient/value fallback
with full-program dependency and resolution semantics.
**RED:** Add tests named with the `fri04_c03_grid_track_` prefix first. Grid and
grid-lanes paths may characterize T01 behavior; nested track and fit-content
programs must fail through the affine-only helper before its focused correction.
**Acceptance:** Real grid and grid-lanes layouts cover nested property values,
both axes, range clamping, missing/definite bases, and invalid numeric errors.
Track runtime resolves nested min/max breadths and fit-content limits, clamps
the complete result, preserves cyclic missing-basis behavior, and propagates
invalid numeric. Classification uses exact `depends_on_basis()` rather than a
percent coefficient; a track is definite only when its full program resolves
at the supplied basis. Tests cover exact static spans and intrinsic space/floor
decisions. No valid numeric uses `NonNumeric`; intrinsic/flex states are unchanged.
**Commands:**
```sh
cargo test --locked -p surgeist-layout fri04_c03_grid_track_
just verify
just verify-generator
```
**Current ordered ranges:** `c36fed92cbcc8e180818681cb8b1fb23084e7ae3..24fbdd097f815e19ae71029fa664de3160236e62`; `24fbdd097f815e19ae71029fa664de3160236e62..0bdd16a45438318120fc3663b2312ce2e693587a`.
**Dependency:** `T01` supplies shared complete-program property resolution.
**Intended commit:** `fix(layout): consume grid and track sizing calculations`.

## 5 Cycle Acceptance
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

## 6 Final Verification
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

## 7 Handoff And Blockers
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
