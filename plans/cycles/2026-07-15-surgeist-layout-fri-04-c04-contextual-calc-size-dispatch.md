# FRI-04-C04 Contextual And Calc-Size Dispatch
Status: complete
Cycle ID: `FRI-04-C04`
Owning repository: `surgeist-layout`
Cycle base: `ab342ae57398edd1c5bedb1504b9c93d96829df4`
Reviewed specification: `plans/specs/2026-07-15-surgeist-layout-fri-04-property-specific-sizing-values.md`
at SHA-256 `e0116f0e3dd28eafabe1ed31117a61ea208e97dee986887f5600d7cbd5a06db4`, commit `5d33f3a4ab694f12985d713f7dbc74b251d55fb6`, sections `FRI-04.4 D-03`, `D-05`, `D-06`, the error front door in `FRI-04.5`, the calc-size rows of `FRI-04.6`, the capability, calc-size, flex, and algorithm evidence in `FRI-04.8`, the production algorithm rows of `FRI-04.9`, and acceptance items 6, 8, 9, and 14.

Reviewed sequence: `plans/sequences/2026-07-15-surgeist-layout-fri-04-property-specific-sizing-values.md`
at SHA-256 `9a35a5cfef82fb5b6c5abc6fd9beee7c0a080f631fd392d2ffe7694e019c4f8b`, commit `5543ef5e9273ee73c187803c79191b8b71949fc0`, entry `FRI-04-C04`.

## Outcome
Dispatch every current preferred, minimum, maximum, and flex-basis state
explicitly in its consuming algorithm. Implement calc-size `Any` and
`FullPercentage`, preserve the supported preferred intrinsic routes and flex
`Auto`/`Content` distinction, and return the exact closed D-06 capability
payload for every behavior retained by a later initiative.

## Boundary
The cycle starts from the published and remotely verified C03 numeric sizing
candidate. It owns the public sizing capability descriptors, crate-private
property dispatch needed to produce them, calc-size used-value consumption,
supported preferred intrinsic routing, and exact behavior at existing leaf,
root, block, flex, grid, grid-lanes, and positioned call sites.

Calc-size `Any` resolves without a basis size and uses zero for a missing
calculation percentage basis. `FullPercentage` first follows the consuming
property's ordinary 100-percent missing-basis rule and applies the calculation
only when that basis produces an original used size; its calculation
percentages independently use zero when missing. The consuming property clamps
the complete result to zero and propagates invalid numeric results.

Root paths report the actual leaf or inner-display algorithm. Absolute-position
sizing reports `Positioned`, including absolute grid children. Flex basis is
always consumed by `Flex`. Preferred `MinContent` and `MaxContent` are supported
only for leaf, block, grid, and grid-lanes intrinsic-availability routes. Every
other contextual behavior follows the exact D-06 property/behavior/algorithm/
axis mapping; no ordinary sizing path uses `NonNumeric` or an auto/max-content
fallback to erase the authored state.

This cycle does not implement the later FRI-06, FRI-07, FRI-08, or FRI-10
format algorithms, including `FLEX-004`. It does not change track semantics,
fixture parsing, helper/serializer code, HTML/XML sources, generated reports or
provenance, dependencies, features, MSRV, docs, root, or siblings. It does not
expand browser unsupported expectations.

No generation input changes or generation commands are authorized. Scoped
generation remains an optional diagnostic but is unnecessary here and cannot
serve as verification. `just parity-all` remains the FRI-13 aggregate release
gate and is not a C04 task or final command.

## Impacts
Public API: additive closed sizing descriptor enums and an output-only payload
through the existing non-exhaustive layout error front door; existing error
semantics become more precise. Dependencies, features, generated artifacts,
docs, examples, MSRV, root, and siblings: unchanged. Root migration remains a
later root-owned handoff. Safety: all owned Rust remains unsafe-free.

## Tasks
### `C04-T1` Typed Capability And Calc-Size Dispatch
**Files:** `src/sizing.rs`, `src/compute.rs`, `src/lib.rs`, and focused sizing,
contract, public-front-door, and error tests.
**Outcome:** Add the closed public D-06 descriptor model with private payload
fields, public accessors, and crate-private construction; add shared typed
property dispatch that preserves every direct and calc-size state and resolves
calc-size `Any`/`FullPercentage` without `NonNumeric`.
**RED:** Add tests named with the `fri04_c04_dispatch_` prefix first. They fail
because the descriptor front door is absent and calc-size currently returns
`NonNumeric` instead of a numeric or exact capability result.
**Acceptance:** Public types, derives, reexports, field privacy, and accessors
match D-06. Both scalar lanes cover `Any` and `FullPercentage` for all four
property roles with definite and missing percentage bases, size substitution,
final negative clamping, and invalid numeric propagation. A table over every
direct and keyword calc-size request produces the exact behavior, property,
algorithm, and physical axis; supported states cannot construct a capability.
**Commands:**
```sh
cargo test --locked -p surgeist-layout fri04_c04_dispatch_
just verify
```
**Dependency:** Published C03 numeric calculation consumption at the cycle base.
**Intended commit:** `feat(layout): add typed sizing capability dispatch`.

### `C04-T2` Leaf, Block, Root, And Positioned Consumption
**Files:** `src/compute.rs`, `src/block.rs`, `src/compute_tests.rs`,
`src/leaf_tests.rs`, `src/root_tests.rs`, and `src/block_tests.rs`.
**Outcome:** Replace ordinary sizing status/fallback dispatch in leaf, block,
root optimization, in-flow child, and absolute-position paths with the typed
dispatcher, including supported preferred intrinsic availability and calc-size
used values.
**RED:** Add tests named with the `fri04_c04_leaf_block_positioned_` prefix
first. They fail because contextual values are erased, calc-size is rejected,
or a later-owned value reports the undifferentiated capability.
**Acceptance:** Real standalone and tree front doors prove supported `Auto`,
`None`, numeric, calc-size `Any`/`FullPercentage`, and leaf/block preferred
intrinsic geometry in both axes. Every unsupported direct and keyword-basis
calc-size member returns its exact payload and node site. Root reports its
actual leaf or block algorithm; absolute block children report `Positioned`.
Missing basis and invalid numeric retain their ordinary typed errors.
**Commands:**
```sh
cargo test --locked -p surgeist-layout fri04_c04_leaf_block_positioned_
just verify
```
**Dependency:** `C04-T1` supplies the reviewed typed dispatcher.
**Intended commit:** `fix(layout): dispatch leaf block and positioned sizing states`.

### `C04-T3` Flex Property And Basis Semantics
**Files:** `src/flex.rs` and `src/flex_tests.rs`.
**Outcome:** Consume preferred/minimum/maximum states and flex basis through the
typed dispatcher, with explicit flex-basis `Auto` versus `Content`, supported
numeric/calc-size bases, and exact FRI-07 capability results.
**RED:** Add tests named with the `fri04_c04_flex_dispatch_` prefix first. They
fail because explicit `Content` is rejected, `Auto`/`Content` share fallback
behavior, or intrinsic/later-owned states silently become max-content.
**Acceptance:** `Auto` consults preferred main size and uses content only when
preferred is auto; `Content` bypasses preferred main size. Numeric, calc-size
`Any`, and calc-size `FullPercentage` use the flex container main-size basis,
including the missing-basis content rule. Every unsupported preferred,
minimum, maximum, direct flex-basis, and keyword calc-size request returns the
exact `Flex` payload and axis. `MinContent` and `MaxContent` remain distinct
unsupported bases and do not enter the content/max-content path.
**Commands:**
```sh
cargo test --locked -p surgeist-layout fri04_c04_flex_dispatch_
just verify
```
**Dependency:** `C04-T1` is task-clean; T2 establishes shared non-flex use.
**Intended commit:** `fix(layout): distinguish flex sizing and content bases`.

### `C04-T4` Grid And Grid-Lanes Dispatch Closure
**Files:** `src/grid/mod.rs`, `src/grid/child.rs`, `src/grid/lanes.rs`,
`src/grid_tests.rs`, and focused aggregate contract tests.
**Outcome:** Consume grid and grid-lanes property states explicitly, preserve
their supported preferred intrinsic routes, classify absolute grid children as
`Positioned`, and close the production/front-door D-06 matrix.
**RED:** Add tests named with the `fri04_c04_grid_dispatch_` prefix first. They
fail because contextual values can become max-content, calc-size is rejected,
or absolute grid children report an undifferentiated result.
**Acceptance:** Real grid and grid-lanes layouts prove supported direct and
calc-size geometry in both axes, including preferred `MinContent` and
`MaxContent`. Every unsupported direct or keyword-basis calc-size member returns
the exact `Grid`, `GridLanes`, or `Positioned` payload and node site. Aggregate
table-driven evidence accounts for every D-06 cell across all algorithms and
all grouped members. Production sizing call sites no longer depend on
`LengthResolutionStatus::NonNumeric` or a silent auto/max-content fallback;
track breadth dispatch remains unchanged.
**Commands:**
```sh
cargo test --locked -p surgeist-layout fri04_c04_grid_dispatch_
just verify
```
**Dependency:** `C04-T1` through `C04-T3` are task-clean.
**Intended commit:** `fix(layout): close grid sizing capability dispatch`.

## Cycle Acceptance
1. All four task ranges have independent clean task reviews and preserve their
   ordered compile-stable boundaries.
2. Calc-size `Any` and `FullPercentage` have the exact basis, percentage,
   used-range, missing-context, and invalid-numeric behavior in every property
   and consuming algorithm.
3. Leaf, block, grid, and grid-lanes preferred intrinsic requests produce
   geometry; flex `Auto` and `Content` follow distinct specified paths.
4. Every D-06 direct and keyword calc-size cell returns supported geometry or
   the exact property/behavior/algorithm/axis payload with its node site.
5. Root and positioned paths identify the actual consuming algorithm, and
   later FRI ownership remains unchanged.
6. No ordinary production sizing path uses `NonNumeric`, converts a contextual
   value to auto/max-content, or changes track behavior.
7. Parser, fixture, browser source, generated artifact, generator architecture,
   dependency, feature, MSRV, docs, root, and sibling state is unchanged.

## Final Verification
```sh
just verify
just verify-generator
just corpus-check
just taffy-check
git diff --check
rg -n --pcre2 '#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{' --glob '*.rs' .
```

The final unsafe scan must report no executable match. The complete cycle diff
and artifact inventory must show no parser, helper, serializer, HTML, XML,
report, provenance, generator, track, dependency, feature, MSRV, docs, root, or
sibling change. Checked-in browser artifacts remain read-only throughout C04.

## Handoff And Blockers
The completed cycle hands C05 a remotely verified production surface whose
supported sizing behavior and capability boundary are stable enough for the
bounded fixture grammar and one final full regeneration. It does not emit the
final FRI-04 or leaf-candidate handoff.

A genuine blocker exists only if an existing consumer cannot identify its
property, physical axis, or actual algorithm without a new public state absent
from the reviewed specification, or if the D-06 matrix contradicts observable
source-owned behavior. Such evidence returns to planning review; it does not
authorize fallback erasure, later-format implementation, generator expansion,
unsafe code, a dependency, or fixture work.
