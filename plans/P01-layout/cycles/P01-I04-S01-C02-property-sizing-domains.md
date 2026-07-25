# P01-I04-S01-C02 Property Sizing Domains
Status: complete
Cycle ID: `P01/I04/S01/C02`
Owning repository: `surgeist-layout`
Cycle base: `601f4ad4700827465096dc62c029b4d8147336b8593b6b80ae7abfb09fc22577`
Reviewed specification: `plans/P01-layout/initiatives/P01-I04-property-specific-sizing-values.md`
at SHA-256 `49ede2ba2672a91f99ba193651dbb1350ede7b80`, commit `5d33f3a4ab694f12985d713f7dbc74b251d55fb6`, sections `FRI-04.4 D-01`, the property-basis construction in `D-03`, `D-04`, `D-05`, `FRI-04.5`, the construction and initial-value portions of `FRI-04.6`, the corresponding model, default, track, parser, and public-surface evidence in `FRI-04.8`, the value, node, front-door, and parser rows of `FRI-04.9`, and acceptance items 2 through 4 plus the construction portions of 6 and 7.

Reviewed sequence: `plans/P01-layout/sequences/P01-I04-S01-property-specific-sizing-values.md`
at SHA-256 `2e006d30b0250c526e10bba13a37e58e111ff60791b34f8d7c2e4d0e527db13f`,
commit `0a666f8f698703cd7979194a7f75f834e4c9b522`, entry `P01/I04/S01/C02`.

## 1 Outcome

Introduce the four closed property-sizing domains and property-specific
calc-size bases, revise track sizing around a validated track-only flex factor,
migrate `NodeInputOf` and every crate-owned construction and simple fixture
adapter path, and remove the complete legacy `Dimension` surface.

## 2 Boundary

The cycle starts from the published and remotely verified C01 calculation
substrate plus the orthogonal BLOCK-005 fixture-adapter prerequisite published
at the cycle base. `DimensionOf` still owns all four `NodeInputOf` fields, broad
resolution paths, track conversions, and fixture parsing. Current track flex is
an unchecked scalar and maximum-size defaults are `Auto`.

This cycle owns property and track value models, public reexports, field
defaults, all direct production and test constructions needed for the public
break, and only the existing fixture adapter's simple property and track paths.
It may expose crate-private exhaustive views so each consumer receives an
explicit property state. Existing affine and keyword behavior must remain
green.

It does not implement general min/max/clamp resolution, calc-size used-value
semantics, contextual capability payloads, new fixture grammar, helper or
serializer support, HTML/XML fixtures, documentation closure, root adapters,
or root API artifacts. Those remain assigned to C03 through C06. A newly
constructable calculation state must not be silently converted to an automatic,
zero, or intrinsic state while its production resolution remains assigned to a
later cycle.

No HTML, helper, serializer, fixture-source, or generated-artifact input changes
are authorized. No generation command is applicable. Scoped generation is not
verification evidence and is unnecessary for this cycle.

The ignored full browser corpus is not C02 acceptance evidence. A single aggregate diagnostic after the BLOCK-005 prerequisite exposed 298 distinct later-owned failures, beginning with `absolute_correct_cross_child_size_with_percentage__border_box_ltr` returning `MissingContext(RequiredBasis)` at node 3. C03 owns that missing-basis behavior, and the findings-resolution index assigns the final aggregate release gate to FRI-13. Do not rerun or use `just parity-all` as a C02 task or cycle gate.

## 3 Impacts
Public API: intentional breaking pre-release replacement of `Dimension` and its
broad conversions with property-specific values and revised track types.
Dependencies, features, docs, examples, MSRV, generated artifacts, root, and
siblings: unchanged. Root migration remains a later root-owned handoff.
Safety: all owned Rust remains unsafe-free.

## 4 Tasks

### 4.1 `P01/I04/S01/C02/T01` Closed Property Value Domains
**Files:** `src/sizing.rs`, `src/lib.rs`, and focused model and public-contract
tests.
**Outcome:** Publish `PreferredSizeOf`, `MinSizeOf`, `MaxSizeOf`, and
`FlexBasisOf` with their exact defaults, keyword domains, numeric and
fit-content-function constructors, property-specific calc-size bases, semantic
predicates, and crate-private exhaustive views. Publish the C01 calculation
types and exact construction errors. Reject a `size`-dependent calculation with
an `Any` basis and admit every other specified property/basis pair.
**RED:** Add tests named with the `property_sizing_` prefix before
implementation. They fail because the wrappers, basis enums, public aliases,
and construction error are absent. Record the expected failure.
**Acceptance:** Both scalar lanes prove construction, equality, clone/debug
traits, signed-zero `ZERO`, exact default/keyword matrices, every valid
calc-size basis, and exact `Any` rejection. Public compile-pass/fail examples
prove no box/flex `fr`, no maximum `Auto`, no non-flex `Content`, and no
property-erasing conversion. `DimensionOf`, `NodeInputOf`, tracks, algorithms,
and fixture parsing remain unchanged in this task.
**Commands:**
```sh
cargo test --locked -p surgeist-layout property_sizing_
just verify
just verify-generator
```
**Current ordered range:** `24350ee0d28f25ef2f1a3d3a3e478be2a079a17c`.
**Dependency:** Published C01 calculation substrate at the cycle base.
**Intended commit:** `api(layout): add property-specific sizing domains`.

### 4.2 `P01/I04/S01/C02/T02` Validated Track Sizing Domains
**Files:** `src/sizing.rs`, `src/value.rs`, `src/lib.rs`, direct grid modules,
their focused tests and test support, and track-only simple parsing in
`tests/layout/browser_parity/support.rs`.
**Outcome:** Publish `TrackFlexFactorOf` with finite non-negative validation and
canonical zero, replace minimum and maximum track numeric breadths with
`SizingCalculationOf`, keep the specified intrinsic/auto and maximum-only
fit-content states, and require validated flex only on maximum breadths. Migrate
every track caller and existing simple track parser without using a broad
property conversion.
**RED:** Add tests named with the `track_sizing_` prefix before implementation.
They fail because the validated factor and revised track constructors do not
exist. Record the expected failure.
**Acceptance:** Both scalar lanes cover valid, negative, non-finite, and signed
zero factors; the complete track construction matrix; existing affine,
intrinsic, fit-content, and flex grid behavior; and simple parser acceptance and
rejection. Minimum breadths cannot carry flex, raw scalar `fr`, `LengthOf::Normal`,
and broad property-to-track conversions are absent. `DimensionOf` remains only
for box/flex migration in T03.
**Commands:**
```sh
cargo test --locked -p surgeist-layout track_sizing_
just verify
just verify-generator
```
**Current ordered range:** `6969a6bc586a38965b7255aff71cac1ff22bec31`.
**Dependency:** `T01` supplies the public calculation surface.
**Intended commit:** `api(layout): migrate track sizing domains`.

### 4.3 `P01/I04/S01/C02/T03` Property Fields And Legacy Removal
**Files:** `src/node_input.rs`, `src/compute.rs`, `src/block.rs`, `src/flex.rs`,
affected grid modules, `src/lib.rs`, every direct crate test/support
construction, and simple preferred/minimum/maximum/flex parsing in
`tests/layout/browser_parity/support.rs`.
**Outcome:** Change `NodeInputOf` to property-specific preferred, minimum,
maximum, and flex-basis fields; set both default paths to preferred/min/flex
`Auto` and maximum `None`; and migrate every production consumer and
crate-owned construction by destination property. Remove `Dimension`,
`DimensionOf`, their broad resolver/conversions, public reexports, parser path,
and every remaining source or test reference.
**RED:** Add tests named with the `property_field_migration_` prefix and the
negative public-surface examples before implementation. They fail because the
field types/defaults remain broad and the legacy surface still compiles. Record
the expected failures.
**Acceptance:** Default and generic scalar inputs prove exact field types and
initial values. Existing affine and keyword geometry remains unchanged across
leaf, root, block, flex, grid, grid-lanes, positioned, and browser-parity paths.
Each consumer matches a property-specific state; no invalid cross-property
constructor or compatibility alias remains. Newly modeled calculation and
calc-size behavior is not misreported as C03/C04 completion.
**Commands:**
```sh
cargo test --locked -p surgeist-layout property_field_migration_
just verify
just verify-generator
```
**Current ordered range:** `f05dbe96c15aeb1bdcf7020652435d5562bad044`.
**Dependency:** `T02` completes the independent track migration first.
**Intended commit:** `api(layout): migrate property sizing fields`.

## 5 Cycle Acceptance

1. All three task ranges have independent clean task reviews and preserve their
   ordered compile-stable boundaries.
2. Preferred, minimum, maximum, flex-basis, minimum-track, and maximum-track
   construction domains match the reviewed matrices in both scalar lanes.
3. `NodeInputOf` and every crate-owned caller use the destination property's
   type; preferred/min/flex default to `Auto` and maximum defaults to `None`.
4. `TrackFlexFactorOf` is finite, non-negative, canonicalizes zero, and cannot
   enter a box, flex-basis, or minimum-track field.
5. `Dimension`, `DimensionOf`, broad resolution/conversions, raw track flex, and
   invalid cross-property constructors are absent from the public and internal
   source surface.
6. C02-owned focused behavior and the checked-in corpus and Taffy integrity
   checks pass with no generated artifact, fixture source, report, or
   provenance delta.
7. C03/C04 resolution and capability behavior, C05 generator inputs and final
   regeneration, and C06 docs/evidence remain outside the range.

## 6 Final Verification

```sh
just verify
just verify-generator
just corpus-check
just taffy-check
git diff --check
rg -n --pcre2 '#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{' --glob '*.rs' .
```

The final `rg` command must report no executable unsafe match. The artifact
inventory and complete cycle diff must show no HTML, XML, report, provenance,
helper, serializer, generator, dependency, feature, or MSRV change. Aggregate
browser parity remains diagnostic during this construction-only cycle and is
not part of this final command set.

## 7 Handoff And Blockers

The completed cycle hands C03 a remotely verified public field migration in
which every production consumer has an explicit property domain and no legacy
compatibility path. It does not emit the final FRI-04 root handoff.

A genuine blocker exists only if the reviewed public break cannot preserve the
existing simple corpus with current dependencies and standard library, or if a
production consumer requires a property or track state absent from the reviewed
construction matrix. Such evidence returns to planning review; it does not
authorize a compatibility alias, generator expansion, unsafe code, dependency,
or silent fallback.
