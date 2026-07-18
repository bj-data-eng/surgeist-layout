# FRI-05-C05 Grid Family Scroll Geometry
Status: complete
Cycle ID: `FRI-05-C05`
Owning repository: `surgeist-layout`
Cycle base: `90c478c1b4449332c4ba411bf6869ef9fca74bec`
Reviewed specification:
`plans/specs/2026-07-16-surgeist-layout-fri-05-overflow-scroll-geometry.md`
at SHA-256
`747dcd6c12ae7d883999b5517572d6877d3c803bdb611143af7affc5afd44f39`,
commit `50c83f01ded0fe4a284e087ffcbd677bfc12af2a`, sections
`FRI-05.4 D-01`, `D-03`, `D-04`, and `D-06` through `D-11`;
`FRI-05.5`; the grid portions of `FRI-05.7` through `FRI-05.9`; and
acceptance items 3 through 7, 11, and 13 in `FRI-05.15`.
Reviewed sequence:
`plans/sequences/2026-07-16-surgeist-layout-fri-05-overflow-scroll-geometry.md`
at SHA-256
`45b66b5a47c3a1bd47e22869e4b841a46aef2ac0ffab37dce5b91e6fc2a996d0`,
commit `07ed42c2a832a7c6fccb11b5d77953fa9c159917`, entry
`FRI-05-C05`.

## Outcome
Move ordinary grid, intrinsic subgrid, and grid-lanes through the shared
reservation, contribution, origin, range, rounding, and cache contracts already
published by block and flex. Remove the final mutable scrollbar output field and
all three grid-family compatibility bridges after their producers emit canonical
geometry.

## Boundary
### Included
- computed-overflow automatic minimums for ordinary grid and grid-lanes;
- flow-aware used-overflow trapping for ordinary grid, intrinsic subgrid, and
  grid-lanes intrinsic, min-content, and percentage-track callers;
- effective classic-scrollbar reservation and monotone auto settlement for
  performed ordinary grid and grid-lanes layout;
- retained in-flow and current absolute child geometry, shared axis-independent
  contribution, nested propagation or trapping, and container-relative origins;
- flow-start grid scroll origins plus final inline and block track alignment
  subjects;
- canonical ordinary-grid, subgrid, and grid-lanes output, source-based
  rounding, ordinary final-cache publication, and removal of the last
  compatibility projections.

### Excluded
- authored CSS, root adapters, retained identity, current offsets, rendering,
  scrolling UI, snap selection, and later FRI behavior;
- new grid placement, track sizing, subgrid, alignment, baseline, or absolute
  positioning semantics not required to feed existing final geometry;
- parser, helper, HTML, fixture, XML, comparator, manifest, report, provenance,
  corpus, generator, dependency, feature, MSRV, documentation, root, or sibling
  changes;
- generator architecture work and every generation command.

### Current Evidence And Decisions
- C04 is published and remotely read back at the cycle base; flex owns no
  separate scroll path.
- `src/grid/tracks.rs` has multiple computed and used overflow selectors.
  Some select `Column => x` and `Row => y` without first projecting the
  logical grid axis through the container's `FlowAxes`.
- `src/grid/child.rs` and `src/grid/lanes.rs` retain local scrollbar
  projections, synthesize output with `scroll_geometry: None`, and reduce
  visible contribution to a size. Their current size helper returns zero when
  either contribution axis is non-positive and therefore cannot preserve
  independent intervals.
- Exactly three grid-family `scroll_geometry: None` output bridges remain at
  the cycle base. They are downstream-owned migration sites, not a reason to
  add another geometry model.
- `NodeOutputOf::scrollbar_size` remains a public mutable compatibility field.
  Its method already derives the authoritative value from canonical geometry.
- All grid formats use ordinary flow inline/block start as scroll origin.
  Existing final justified and aligned track rectangles are the only active
  grid alignment subjects.
- Each nested node starts local auto settlement at `INITIAL`; the immediate
  containing pass remains a separate cache discriminator. C05 reuses that
  reviewed C04 state model unchanged.
- Scoped test runs may diagnose an implementation while code is changing.
  They are not final verification evidence. No generator or generator-input
  change is authorized, so C05 runs no generation command.

## Impacts
- **Public API:** completes the specification's intentional breaking removal of
  the public mutable `NodeOutputOf::scrollbar_size` field. The existing
  `scrollbar_size()` and `content_box_size()` methods remain and derive only
  from canonical geometry.
- **Dependencies and features:** unchanged.
- **Generated artifacts:** unchanged; root still owns API audit artifacts.
- **Docs and examples:** unchanged in this cycle.
- **MSRV:** Rust 1.97 and edition 2024 remain unchanged.
- **Root follow-up:** none until the final FRI-05 leaf candidate handoff.
- **Unsafe:** prohibited in every tracked and non-ignored owned Rust file.

## Tasks
### `C05-T1` Grid Overflow Sizing Decisions
**Files:** `src/grid/tracks.rs`, narrowly required `src/grid/lanes.rs`, and
focused grid, subgrid, lanes, and static tests.

**Outcome:** Give ordinary-grid and lanes automatic minimums the corrected
computed-overflow predicate. Replace context-free intrinsic overflow selection
with one private helper that projects `GridAxisKind` through the consuming
container's `FlowAxes`, derives the used axis including replaced-hidden
conversion, and admits nested `content_size` only for used `Visible`.

**RED:** Add `fri05_c05_grid_auto_minimum_` and
`fri05_c05_grid_intrinsic_overflow_` tests first. They fail on `Auto`, on an
orthogonal physical-axis projection, or because trapped and replaced-hidden
cases still consume descendant content.

**Acceptance:** Real ordinary-grid and grid-lanes front doors cover all five
computed values for automatic minimums. Ordinary grid, intrinsic subgrid, and
lanes cover all five used values, replaced `Hidden => Clip`, both logical
axes, and every applicable `FlowAxes` projection. Only used `Visible`
admits descendant content; item-box and min-track priority remain otherwise.
No unrelated track algorithm changes.

**Commands:**
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c05_grid_auto_minimum_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c05_grid_intrinsic_overflow_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
```

**Dependency:** C04's canonical computed and used overflow model is published.

**Intended commit:** `fix(layout): make grid overflow sizing flow-aware`.

### `C05-T2` Grid Child Geometry And Contributions
**Files:** `src/grid/child.rs`, `src/grid/lanes.rs`, narrowly required
`src/grid/subgrid.rs` and `src/scroll.rs`, plus focused grid-family tests.

**Outcome:** Retain each laid-out in-flow and current absolute child's canonical
geometry and target metadata. Feed final container-local border and margin
boxes, translated nested geometry, and terminal padding through the shared
source-ordered contribution accumulator without reducing them to a size or
subtracting a grid-area origin.

**RED:** Add `fri05_c05_grid_child_geometry_` and
`fri05_c05_grid_contribution_` tests first. They fail because current bridges
discard child geometry, area-relative or zero-axis contribution loses a real
interval, or a trapped descendant reaches its parent.

**Acceptance:** Ordinary grid, subgrid, and grid-lanes retain in-flow and current
absolute child geometry exactly once in source/output identity order. Non-zero
container origins use final item ends; `0xN` and `Nx0` visible descendants
survive independently; `Clip`, `Hidden`, `Scroll`, and `Auto` trap each
physical axis independently; margins, terminal padding, negative positions, and
current absolute items contribute once. Tests use production formatting front
doors, not a parallel geometry simulator.

**Commands:**
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c05_grid_child_geometry_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c05_grid_contribution_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
```

**Dependency:** C05-T1 supplies the one flow-aware used-overflow decision.

**Intended commit:** `fix(layout): retain grid child scroll contributions`.

### `C05-T3` Ordinary Grid Reservation Origins And Output
**Files:** `src/grid/mod.rs`, narrowly required `src/grid/child.rs`,
`src/scroll.rs`, `src/compute.rs`, and focused grid/root/cache tests.

**Outcome:** Perform ordinary-grid sizing and placement in the effective content
box for the current scrollbar pass, settle `Auto` monotonically from pass-local
canonical geometry, and publish one canonical container output with flow-start
origins and final inline/block track alignment subjects.

**RED:** Add `fri05_c05_grid_geometry_`,
`fri05_c05_grid_auto_`, and `fri05_c05_grid_alignment_` tests first. They
fail because performed grid emits no canonical geometry, reserves against
legacy edges, or cannot express alignment-origin range.

**Acceptance:** Visible, clip, hidden, scroll, and auto outputs; replaced hidden;
forced, stable, both-edge, zero-thickness, tiny, x-only, y-only, and induced
auto cases; all ten writing-mode/direction mappings; final start/end/center,
safe fallback, and applicable distributed track subjects; nested local versus
containing pass state; signed ranges and target metadata all pass through the
ordinary-grid root/front door. Speculative passes do not publish output or
cache entries, and no later-owned grid sizing or placement behavior changes.

**Commands:**
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c05_grid_geometry_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c05_grid_auto_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c05_grid_alignment_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
```

**Dependency:** C05-T2 supplies retained child sources and complete contribution.

**Intended commit:** `feat(layout): emit ordinary grid scroll geometry`.

### `C05-T4` Subgrid And Grid-Lanes Canonical Output
**Files:** `src/grid/mod.rs`, `src/grid/lanes.rs`,
`src/grid/subgrid.rs`, narrowly required `src/grid/child.rs` and
`src/scroll.rs`, plus focused subgrid, lanes, root, and cache tests.

**Outcome:** Apply the same reservation, local auto settlement, contribution,
flow-start origin, track-subject, and canonical output contract to intrinsic
subgrid and grid-lanes paths without synthesizing a second range convention.

**RED:** Add `fri05_c05_subgrid_geometry_` and
`fri05_c05_grid_lanes_geometry_` tests first. They fail because those paths
still publish absent geometry, lose parent-local translation, or bypass the
ordinary grid pass state.

**Acceptance:** Intrinsic subgrid and grid-lanes cover every used overflow,
visible propagation and trapped alternatives, partial axes, zero-area
descendants, non-zero origins, current absolute children, forced/stable/auto
reservation, signed alignment ranges, all applicable flow mappings, rounded
output, and cold/warm cache equality. Their semantics reuse shared sources and
do not alter unrelated lane, placement, baseline, or subgrid inheritance
algorithms.

**Commands:**
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c05_subgrid_geometry_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c05_grid_lanes_geometry_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
```

**Dependency:** C05-T3 establishes the ordinary grid container integration.

**Intended commit:** `feat(layout): emit subgrid and lanes scroll geometry`.

### `C05-T5` Grid Rounding Cache And Legacy Closure
**Files:** `src/output.rs`, `src/compute.rs`, `src/cache.rs`, grid-family
modules only as narrowly required to remove migrated bridges, `src/lib_tests.rs`,
and focused grid/root/cache/rounding/contract/static tests.

**Outcome:** Rebuild every grid-family rounded output from retained canonical
sources, publish it through ordinary cache equality, remove the mutable
`NodeOutputOf::scrollbar_size` field and every remaining compatibility
projection, and leave canonical geometry as the only source of content-box and
gutter accessors.

**RED:** Add `fri05_c05_grid_round_cache_` and
`fri05_c05_grid_legacy_absence_` tests first. They fail because the public
field and three grid-family construction bridges remain or because rounded and
cached output can still depend on a projection.

**Acceptance:** Normal and rounded ordinary-grid, subgrid, and lanes geometry,
target, used axes, gutter, range, subjects, and content extent agree in `f32`
and `f64`; cached and uncached root/nested output is identical; controls and
measurement-only output remain absent; `content_box_size()` and
`scrollbar_size()` derive only from canonical geometry; all three base bridges
and the public mutable field are absent; aggregate `fri05_c05_` evidence
passes; and no block/flex behavior or source path regresses.

**Commands:**
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c05_grid_round_cache_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c05_grid_legacy_absence_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c05_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
```

**Dependency:** C05-T4 migrates the last grid-family producers.

**Intended commit:** `fix(layout): close grid scroll geometry paths`.

## Cycle Acceptance
1. All five task ranges have genuine RED/GREEN evidence, clean independent task
   reviews, and coordinator-rerun acceptance commands.
2. Ordinary-grid and lanes automatic minimums use computed scrollability; every
   intrinsic grid-family caller uses one flow-aware used-overflow selector.
3. Ordinary grid, intrinsic subgrid, and grid-lanes retain child and current
   absolute geometry, translate final container-local origins, preserve
   independent zero-axis contribution, and trap nested overflow per used axis.
4. Every performed grid-family box emits canonical geometry with effective
   reservation, monotone auto settlement, flow-start origins, applicable final
   track subjects, nested target metadata, and no speculative publication.
5. Normal/rounded and cached/uncached behavior agrees in both scalar lanes.
   Controls and measurement-only paths retain absent geometry.
6. The public mutable scrollbar field, three base grid-family bridges, local
   reservation/projection/contribution paths, and compatibility constructors
   are absent. Canonical accessors remain.
7. Grid-family portions of `OVERFLOW-001`, `OVERFLOW-002`,
   `OVERFLOW-003`, and `CORE-006` are traceable to focused production
   evidence; fixture/comparator closure remains assigned to C06.
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
range inspection proves no generator or generator-input change, no added lint
suppression, and no block/flex semantic delta beyond removal of the shared
compatibility field. No generation command is part of C05 verification.

## Handoff And Blockers
The completed, reviewed, published, and remotely read-back cycle hands C06 only
canonical output from every owned formatting context, no mutable scrollbar
field or compatibility bridge, and stable production decisions for the bounded
fixture/comparator work and sole final full regeneration.

A genuine blocker exists only if a current grid-family algorithm does not retain
the final container-local box or track subject required by the reviewed
contract, or if completion requires a new public model, dependency, unsafe code,
generator/fixture change, generator architecture, later-owned grid algorithm,
or external-repository mutation. Such evidence returns to planning review
rather than widening C05.
