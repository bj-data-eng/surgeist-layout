# FRI-05-C04 Flex Scroll Geometry And Main-Axis Semantics
Status: draft
Cycle ID: `FRI-05-C04`
Owning repository: `surgeist-layout`
Cycle base: `63f8823df0262f886b8031cf527021ded5f0098b`
Reviewed specification:
`plans/specs/2026-07-16-surgeist-layout-fri-05-overflow-scroll-geometry.md`
at SHA-256
`747dcd6c12ae7d883999b5517572d6877d3c803bdb611143af7affc5afd44f39`,
commit `50c83f01ded0fe4a284e087ffcbd677bfc12af2a`, sections `FRI-05.4 D-01`
and `D-06` through `D-11`, the flex contract in `FRI-05.7`, the flex rows
of `FRI-05.8` and `FRI-05.9`, `FRI-05.10`, and the flex portions of
acceptance items 3 through 7, 11, and 13 in `FRI-05.15`.
Reviewed sequence:
`plans/sequences/2026-07-16-surgeist-layout-fri-05-overflow-scroll-geometry.md`
at SHA-256
`45b66b5a47c3a1bd47e22869e4b841a46aef2ac0ffab37dce5b91e6fc2a996d0`,
commit `07ed42c2a832a7c6fccb11b5d77953fa9c159917`, entry `FRI-05-C04`.

## Outcome
Preserve the one correct canonical-pair classifier shared by both flex
automatic-minimum callers, and make the completed `FlexAxes` the sole owner of
flex scroll-origin progression. Integrate canonical effective boxes, retained
child geometry, shared contribution, monotone auto settlement, final content-
distribution subjects, source-based rounding, and cache-safe publication
through the real flex front door. Remove flex-local scrollbar and content-extent
projections while leaving grid-family migration and final mutable output-field
removal to C05.

## Boundary
The published C03 candidate supplies canonical root/leaf/block output, one
padding-box-seeded contribution accumulator, source-retained geometry and
rounding, target metadata, flow-aware effective gutters, and the cache-keyed
settled-auto state. C04 consumes those contracts; it does not add another box,
range, contribution, origin, or auto-state model.

At the cycle base, `src/flex.rs` has the complete `FlexAxes` main/cross physical
axes, sides, reversals, and progressions. Both automatic-minimum callers already
share one pair classifier. `ComputedOverflow::try_new` guarantees equal x/y
scrollability classes across all thirteen accepted pairs, and rejects all twelve
mixed-class pairs, so the classifier is correct without a physical-axis
projection. C04 preserves that invariant and does not manufacture invalid test
input. Flex constants derive one-pass right/bottom-style reservation through
`ScrollbarReservationOf`, store a size-only gutter point, and subtract it through
a legacy inset helper. Container output projects visible child and absolute
extents as sizes and emits no canonical geometry. Final in-flow and absolute
child outputs discard the child geometry and populate the temporary scrollbar
field through two flex-owned bridge sites. The C03 static accounting retains
exactly those two flex sites plus three grid-family sites.

C04 owns `src/flex.rs`, narrowly required shared-scroll integration in
`src/scroll.rs`, the affected flex/root/cache/contract/static tests, and no other
formatting algorithm. The task order may reshape private flex pass records so
final item and absolute-child geometry remains available to one accumulator.
It may not change flex sizing, line formation, item placement, or absolute
positioning except where the reviewed effective gutter changes available space.

The fixed-size `ComputeSize` fast path and every measurement-only result remain
geometry-free. Performed flex containers, in-flow items, and current absolute
items retain canonical geometry and target metadata. The existing public
`NodeOutputOf::scrollbar_size` field stays synchronized through the canonical
output helper until C05 removes it after grid migration; C04 removes only the
two flex-owned writer bridges.

No task changes HTML, parser, helper, serializer, fixture, XML, report,
provenance, manifest, generator code, dependency, feature, MSRV, public export,
README, root, or sibling repository. No generation command runs. Scoped
generation is neither needed nor evidence; `just verify-generator` only builds,
tests, and lints the existing generator feature.

## Impacts
- **Public API:** Internal-only implementation of the already reviewed geometry
  surface; no new export, constructor, field, error, or compatibility alias.
- **Dependencies and features:** Unchanged.
- **Generated artifacts:** None; all generator inputs and outputs remain byte
  unchanged and no regeneration is permitted.
- **Docs and examples:** Unchanged; aggregate public documentation remains C07.
- **MSRV:** Rust 1.97 remains unchanged.
- **Root follow-up:** None in this cycle; C07 owns the complete root handoff.
- **Safety:** Surgeist-owned code remains free of executable `unsafe` and of an
  allowance that would permit it.

## Tasks

### `C04-T1` Canonical Flex Box And Retained Child Output
**Files:** `src/flex.rs`, narrowly required `src/scroll.rs`, `src/flex_tests.rs`,
focused root tests, and flex/grid bridge-accounting tests in `src/lib_tests.rs`.

**Outcome:** Derive each performed flex pass's border, padding, effective gutter,
content box, and scrollport from the canonical source using the pass's settled
auto state. Replace the legacy reservation point/inset path. Seed one shared
accumulator with the canonical padding box, emit canonical container geometry
and target for a non-overflowing final pass, and retain each performed in-flow
and current absolute child's geometry when constructing its `NodeOutputOf`.
Synchronize the temporary scrollbar field through the canonical output helper
and remove both flex-owned direct writer bridges.

**RED:** Add `fri05_c04_flex_geometry_` and
`fri05_c04_flex_child_geometry_` tests first. They fail because flex containers
emit no geometry, child outputs discard it, and stable/both-edge placement uses
the old physical reservation path.

**Acceptance:** Empty and simple non-overflowing flex containers publish
canonical boxes, used axes, target, clip, gutter, and zero range for all ten
flow mappings under no gutter, forced scroll, stable, both-edges, zero thickness,
and saturated small boxes. In-flow and current absolute child output preserves
the child's canonical geometry and target. Static accounting reports zero
flex-owned bridge sites and exactly the three C05-owned grid-family sites.

**Commands:**
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c04_flex_geometry_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c04_flex_child_geometry_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c04_flex_bridge_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
```

**Dependency:** Published C03 candidate and the clean revised specification and
sequence revisions in the header. Existing C01 pair construction and flex
front-door controls account for the preserved automatic-minimum classifier.

**Intended commit:** `feat(layout): emit canonical flex scroll geometry`.

### `C04-T2` Shared Flex Contribution And Content Extent
**Files:** `src/flex.rs`, narrowly required `src/scroll.rs`, and focused flex and
root integration tests.

**Outcome:** Replace size-only visible-child and absolute-content projections
with the shared accumulator. Include every final in-flow item and current
absolute item exactly once from its retained geometry and final container-local
location; apply only positive margin outsets; translate nested descendant
intervals only on used-visible physical axes; record final in-flow ends and
terminal padding; and derive `content_size` from the canonical content-box
anchor and complete accumulated overflow.

**RED:** Add `fri05_c04_flex_contribution_`, `fri05_c04_flex_nested_`, and
`fri05_c04_flex_absolute_` tests first. They fail because flex collapses each
child to a size, loses negative origins and transitive geometry, treats one zero
axis as an all-axis exclusion, and cannot distinguish propagated from trapped
descendants.

**Acceptance:** Positive border area plus positive margin outsets is exact;
negative margins remain valid; `0xN` and `Nx0` used-visible descendant
intervals survive independently; `Clip`, `Hidden`, `Scroll`, and `Auto` trap on
each physical axis; partial-axis cases remain independent; in-flow and current
absolute children contribute once; terminal padding is retained; and root and
nested flex `content_size` equals the independent-axis anchor/overflow union.

**Commands:**
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c04_flex_contribution_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c04_flex_nested_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c04_flex_absolute_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
```

**Dependency:** C04-T1 retains exact child geometry and establishes the canonical
container source.

**Intended commit:** `fix(layout): unify flex scroll contributions`.

### `C04-T3` Monotone Flex Auto Settlement
**Files:** `src/flex.rs`, narrowly required `src/scroll.rs`, and focused flex,
root, and cache tests.

**Outcome:** Execute complete flex layout inside the shared monotone settled-auto
transition. Feed the current state into effective boxes and every speculative
child request, rerun only when a newly added non-zero reservation changes
available geometry, and publish/cache only the first stable pass under the
ordinary request. Keep state bits monotone and use the C03 cache identity rather
than a flex-local counter, tolerance, or retry limit.

**RED:** Add `fri05_c04_flex_auto_`, `fri05_c04_flex_reservation_`, and
`fri05_c04_flex_tiny_` tests first. They fail because flex performs one legacy
reservation pass and cannot settle cross-axis induction without exposing
speculative child output.

**Acceptance:** Root and nested flex front doors prove no overflow, x-only,
y-only, x-induces-y, y-induces-x, forced scroll, hidden stable, both-edges,
zero thickness, and tiny saturated boxes for representative physical gutter
sides and all ten flow mappings. At most three geometry-changing evaluations
occur; every child/cache lookup has exact state bits; only stable node output is
published; and cached and uncached results agree.

**Commands:**
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c04_flex_auto_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c04_flex_reservation_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c04_flex_tiny_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
```

**Dependency:** C04-T2 supplies complete provisional overflow for the state
transition.

**Intended commit:** `feat(layout): settle flex auto scrollbars`.

### `C04-T4` Flex Origins And Content-Distribution Subjects
**Files:** `src/flex.rs`, narrowly required `src/scroll.rs`, and focused flex and
root tests.

**Outcome:** Map the completed `FlexAxes` main and cross progression into the
one canonical `ScrollOriginAxes`: row/column reverse makes the main origin
flow-startward and wrap-reverse makes the cross origin flow-startward. Record
the final in-flow main subject only when `justify_content: Some` actually applies
without an origin-start safe fallback, and the final line cross subject only
when `align_content: Some` applies to the formed lines. Keep these subjects
separate from complete overflow and exclude current absolute and farther nested
start-side geometry from start reachability.

**RED:** Add `fri05_c04_flex_origin_` and `fri05_c04_flex_alignment_` tests
first. They fail because flex currently has ordinary size-only extent, no
reverse-origin range, and no bounded alignment subject.

**Acceptance:** Row/column reverse and wrap-reverse produce the specified signed
main/cross ranges across all ten flow mappings. Existing start, end, center,
space distribution, safe fallback, single-line/inapplicable align-content, and
normal `None` cases prove zero anchoring, one- and two-sided bounds, and actual
subject reach. A farther out-of-flow or nested start-side box does not enlarge
the start bound, while ordinary origin-end overflow remains included.

**Commands:**
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c04_flex_origin_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c04_flex_alignment_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
```

**Dependency:** C04-T3 supplies stable final placement and complete overflow.

**Intended commit:** `feat(layout): derive flex scroll origins and subjects`.

### `C04-T5` Flex Rounding Cache And Legacy Closure
**Files:** `src/flex.rs`, narrowly required `src/scroll.rs`, `src/lib_tests.rs`,
and focused flex, root, cache, rounding, contract, and static tests.

**Outcome:** Reconcile aggregate flex output through source-based rounding and
ordinary cache publication. Remove flex-local reservation, scrollbar-side,
size-projection, contribution, and geometry-discard helpers made obsolete by
T1 through T4. Prove the temporary output field is synchronized only by
canonical geometry and that only the three C05 grid-family bridges remain.

**RED:** Add `fri05_c04_flex_round_cache_` and
`fri05_c04_flex_legacy_absence_` tests first. They fail because rounded/cached
flex has no retained canonical source and the old helper/bridge surface remains.

**Acceptance:** Normal and rounded flex geometry, target, used axes, gutter,
range, subject bounds, and content extent agree in `f32` and `f64`; cached and
uncached root/nested results are identical; controls and measurement-only
outputs remain absent; aggregate `fri05_c04_` evidence passes; no flex-local
legacy reservation/projection/geometry-discard path remains; and grid-family
source and behavior is byte-unchanged except the static bridge count that names
its three downstream-owned sites.

**Commands:**
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c04_flex_round_cache_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c04_flex_legacy_absence_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c04_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
```

**Dependency:** C04-T4 completes the flex source facts rebuilt by rounding.

**Intended commit:** `fix(layout): close flex scroll geometry paths`.

## Cycle Acceptance
1. All five task ranges have genuine RED/GREEN evidence, clean independent task
   reviews, and coordinator-rerun acceptance commands.
2. Both automatic-minimum callers retain one canonical-pair classifier; the
   thirteen accepted and twelve rejected pair matrix plus real pair-group flex
   controls prove its result without fabricated mixed input.
3. Performed flex containers, in-flow items, and current absolute items retain
   canonical geometry and target metadata; partial-axis nested propagation,
   trapped values, zero-area descendants, margins, terminal padding, and
   content extent use the one shared accumulator.
4. Forced, stable, both-edge, auto coupling, tiny boxes, and zero thickness use
   canonical effective boxes and the monotone cache-keyed state; speculative
   output is unobservable.
5. Reverse and wrap-reverse origin progression plus applicable justify/align
   subjects produce zero-anchored signed ranges without admitting unrelated
   start-side out-of-flow overflow.
6. Rounding and cache equality preserve all source-derived geometry in both
   scalar lanes; no flex-local legacy geometry or scrollbar bridge remains.
7. The flex portions of `OVERFLOW-001`, `OVERFLOW-002`, `OVERFLOW-003`, and
   `CORE-006` are traceable to focused production evidence; grid-family and
   initiative closure remain assigned to C05 through C07.
8. Normal and generator-feature verification pass with no generation or parser,
   fixture, artifact, manifest, report, provenance, dependency, feature, MSRV,
   docs, root, sibling, broad lint-suppression, or unsafe delta.

## Final Verification
```sh
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
git diff --check
git ls-files -co --exclude-standard -- '*.rs'
git ls-files -co --exclude-standard -z -- '*.rs' | xargs -0 rg -n --pcre2 '#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{'
```

The owned-Rust manifest and scan cover every tracked and non-ignored Rust file.
Every textual match is classified and no executable match may remain. Final
range inspection also proves no generator or generator-input change, no added
lint suppression, and no grid implementation delta. No generation command is
part of C04 verification.

## Handoff And Blockers
The completed, reviewed, published, and remotely read-back cycle hands C05 one
canonical flex output path, preserved automatic-minimum pair classifier, shared
contribution, monotone auto settlement, and flex-origin/alignment-subject
contract. C05 may
integrate ordinary grid, subgrid, and grid-lanes and then remove the final shared
mutable output field; it may not reopen flex geometry or add another source
model.

A genuine blocker exists only if the completed `FlexAxes` cannot express the
reviewed main/cross axis or progression, if current flex placement does not
retain enough final subject geometry without changing later-owned sizing or
positioning semantics, or if completion requires a dependency, unsafe code,
generator/fixture change, generator architecture, later-cycle algorithm, or
external-repository mutation. Such evidence returns to planning review rather
than widening C04.
