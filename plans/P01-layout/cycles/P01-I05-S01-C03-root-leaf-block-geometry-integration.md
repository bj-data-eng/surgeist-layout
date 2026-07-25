# P01-I05-S01-C03 Root Leaf And Block Geometry Integration
Status: complete
Cycle ID: `P01/I05/S01/C03`
Owning repository: `surgeist-layout`
Cycle base: `885b1dbb5a7be8ff82fe76764038b1c99ed1c6ee7f299b2ea96aef0616d6e67d`
Reviewed specification:
`plans/P01-layout/initiatives/P01-I05-overflow-scroll-geometry.md`
at SHA-256
`0a666f8f698703cd7979194a7f75f834e4c9b522`,
commit `dd158dc598d453a1b9055cb285534717c9a2d4f0`, sections `FRI-05.4
D-03` through `D-11`, the root/leaf/block and output portions of `FRI-05.5`
through `FRI-05.8`, the `scroll.rs`, `output.rs`, `compute.rs`, `block.rs`,
focused-test, and `lib.rs` rows of `FRI-05.9`, `FRI-05.10`, and the applicable
parts of acceptance items 3 through 8, 11, and 13 in `FRI-05.15`.
Reviewed sequence:
`plans/P01-layout/sequences/P01-I05-S01-overflow-scroll-geometry.md`
at SHA-256
`a08dd4dc134ee512ceab9ff16fab12ba183219f3135a5b720c26c76b869494d6`,
commit `45e12b360ececa1ea4518c4da7400f58331b3a3a`, entry `P01/I05/S01/C03`.

## 1 Outcome
Switch root, measured leaf, and block production to the canonical C02 geometry
factory and shared contribution model. Implement flow-aware stable, both-edge,
forced, and monotone auto reservations; preserve only stable pass output; close
small-box, negative-margin, partial-axis, nested, line, float, absolute-child,
helper, rounding, and cache behavior; and remove the root/block legacy geometry
surface while leaving flex/grid producer migration and the shared mutable output
field to C04/C05.

## 2 Boundary
At the cycle base, C02's source record, factory, effective-edge derivation,
accumulator, target carrier, range derivation, and source-based rounding are
private and have no production caller. `ComputeOutputOf` and `NodeOutputOf` still
store arbitrary legacy `ScrollGeometryOf`; `NodeOutputOf` has an independent
public `scrollbar_size` field and an unsaturated content-box helper. Root rebuilds
legacy geometry from `content_size`; measured leaf uses a direction-hard-coded
reservation and emits no geometry; block owns a rectangle-union accumulator,
synthetic signed-margin boxes, independent child projections, and one-pass
forced-scroll reservation.

This cycle owns the canonical production surface in `src/scroll.rs`, output and
pass state in `src/output.rs`, root/leaf integration and rounding in
`src/compute.rs`, block integration in `src/block.rs`, intentional reexports in
`src/lib.rs`, and focused tests in the existing scroll, output/contract, leaf,
root, block, cache, and library test modules. The existing cache key includes the
complete private `ComputeInputOf`; the auto-state bits therefore travel in that
input, and speculative child cache entries are distinguishable from the ordinary
request. Staged node output is final-pass-only by node identity.

The exact canonical public geometry accessors are those listed in D-03.
`ScrollbarGutterRectsOf` becomes four-edge read-only output. During ordered
migration, one crate-private source-fact adapter may keep root/block compiling
against the one canonical factory; it is removed by T05 and never becomes a
public constructor, compatibility alias, or second derivation. The
`NodeOutputOf::scrollbar_size` field remains temporarily for flex/grid writers,
while the same-named method is canonical immediately and root/leaf/block keep the
field equal to the derived value. C05 removes the field after the last producers
migrate.

The no-public-construction rule applies only to canonical geometry, gutter,
clip, and target output carriers. `NodeOutputOf::new`, its real empty `Default`,
and its existing public layout fields remain available in C03; only its
independently mutable scrollbar field is scheduled for C05 removal. Existing
fixture/parity consumers may continue constructing an empty node output and
reading `location`, `size`, and other ordinary layout fields without a helper or
serializer migration.

Root/leaf/block use ordinary flow inline/block-start origins and no alignment
subject. Root proof in this cycle uses measured-leaf and block roots; flex and
grid internal reservation, accumulation, target retention, and auto reruns remain
exclusively C04/C05 even when their current output is wrapped by the root front
door. Existing inline construction, float exclusion, flex/grid algorithms,
positioned completeness, comparator activation, fixtures, and browser parity
remain with their named later initiatives or cycles.

No task changes authored CSS, parser/helper/serializer, HTML, manifest, XML,
report, provenance, generator logic, generator architecture, dependency, feature,
MSRV, README, root, sibling, or API artifact. No generation command is
applicable. Scoped generation is not verification evidence and is unnecessary
for this source-only cycle.

## 3 Impacts
Public API: intentional breaking pre-release replacement of arbitrary legacy
geometry with D-03 read-only output; physical-edge gutter accessors replace the
horizontal/vertical constructor surface; `ScrollRectOf::new`,
`ScrollOverflowExposure`, `ScrollContainerAxis`, `ScrollContainerFacts`,
`scroll_container_facts_from_overflow`, `ScrollGeometryOf::container`, and
`ScrollUnsupportedFeature` disappear without aliases; direct `compute_leaf`
measurement changes from `FnOnce` to `FnMut`; and canonical
`NodeOutputOf::scrollbar_size()` plus corrected `content_box_size()` are exposed.
The mutable scrollbar field remains only as the sequence-authorized C05 bridge.
Dependencies, features, generated artifacts, crate docs/examples, MSRV, root,
siblings, and root-owned API artifacts are unchanged. All owned Rust remains
free of `unsafe` and no lint suppression is added.

## 4 Tasks

### 4.1 `P01/I05/S01/C03/T01` Canonical Production Output And Public Geometry Surface
**Files:** `src/scroll.rs`, `src/output.rs`, `src/lib.rs`, and focused scroll,
contract, cache-model, and library surface tests; only the minimal root/block call
adjustment needed for the temporary crate-private source adapter.

**Outcome:** Promote the C02 canonical result to the sole
`ScrollGeometryOf<S>`, expose the exact D-03 accessors, reshape gutter output to
private physical edges, and make factory/source/accumulator operations available
only crate-wide to production algorithms. Route both output carriers to this
type; add canonical `NodeOutputOf::scrollbar_size()` and canonical-or-saturated
`content_box_size()`. Remove legacy public facts, arbitrary geometry/gutter
construction, and their reexports while retaining only the bounded private
root/block source adapter until T05.

**RED:** Add `fri05_c03_public_geometry_`, `fri05_c03_output_helper_`, and
`fri05_c03_legacy_surface_` tests first. They fail because canonical accessors are
private, helpers use mutable/raw facts, and legacy constructors/types remain
public.

**Acceptance:** Both scalar lanes compile and inspect every exact D-03 accessor;
present geometry always has its target; gutter edges are independently readable;
canonical geometry/gutter/clip/target carriers have no public fields, `Default`,
or public construction; `NodeOutputOf` retains its ordinary public construction
and layout fields; helpers agree for no/one/both/saturated gutters and
no-geometry saturation; compile-fail and static evidence rejects every removed
public name. Existing production compiles through one private source-fact adapter
and one factory only.

**Commands:**
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c03_public_geometry_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c03_output_helper_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c03_legacy_surface_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
```

**Dependency:** Published C02 candidate and the clean revised specification and
sequence revisions in the header.

**Intended commit:** `api(layout): expose canonical scroll geometry output`.

### 4.2 `P01/I05/S01/C03/T02` Cache-Keyed Auto State And Measured-Leaf Geometry
**Files:** `src/scroll.rs`, `src/output.rs`, `src/cache.rs`, `src/compute.rs`,
`src/leaf_tests.rs`, `src/root_tests.rs`, and focused cache tests.

**Outcome:** Add the private monotone physical x/y auto state and transition to
the complete compute input/cache identity. Change direct leaf measurement to
`FnMut`; run only measurement-required geometry-changing states; derive each
pass's flow-aware effective content box and canonical geometry; and publish the
first stable result with its target. Preserve the fully known `ComputeSize`
zero-call/no-geometry fast path and all other measurement-only absence. Copy the
state into `CacheKeyOf::from_input` and compare it in `matches_output` so
speculative child entries remain state-keyed, while the stable leaf result is
stored under the caller's unchanged ordinary request.

**RED:** Add `fri05_c03_leaf_geometry_`, `fri05_c03_leaf_auto_`, and
`fri05_c03_leaf_cache_` tests first. They fail because leaf emits no geometry,
the callback is one-shot, auto is reserved incorrectly or not iterated, and the
cache key has no settled state.

**Acceptance:** Direct and tree-backed leaves prove no overflow, x-only, y-only,
x-induces-y, y-induces-x, forced scroll, hidden stable, stable both-edges,
zero-thickness, all ten flow mappings, partial clips, target metadata, and exact
effective measurement inputs. Measurement occurs zero times for fully known
`ComputeSize`, otherwise one to three times with monotone bits; only the stable
output is stored under the ordinary request, every speculative lookup requires
the exact state bits in both key construction and matching, and cached/uncached
results agree.

**Commands:**
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c03_leaf_geometry_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c03_leaf_auto_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c03_leaf_cache_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
```

**Dependency:** T01 supplies the canonical carrier and factory front door.

**Intended commit:** `feat(layout): emit stable measured-leaf scroll geometry`.

### 4.3 `P01/I05/S01/C03/T03` Saturated Block Reservation And Monotone Auto Passes
**Files:** `src/scroll.rs`, `src/output.rs`, `src/compute.rs`, `src/block.rs`, and
focused block/root tests.

**Outcome:** Derive block constants and available content space from canonical
effective edges for the current auto state. Execute the same monotone transition
around complete block layout, key speculative child work by state, retain only
stable staged node output, and saturate every border/padding/gutter subtraction
before child layout or accumulator seeding.

**RED:** Add `fri05_c03_block_reservation_`, `fri05_c03_block_auto_`, and
`fri05_c03_block_tiny_` tests first. They fail because block reserves only forced
physical right/bottom scrollbars in one pass and the named 2px/15px family can
produce invalid inner geometry.

**Acceptance:** Root and nested block front doors prove forced, hidden stable,
stable both-edges, auto no/x/y/cross-axis induction, zero thickness, all ten flow
mappings, and a maximum of three geometry-changing evaluations. Requested
opposing edges saturate proportionally; tiny border boxes produce ordered zero
content/scrollport geometry without panic or unsupported result; speculative
node/cache output cannot replace the stable result under the ordinary request.

**Commands:**
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c03_block_reservation_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c03_block_auto_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c03_block_tiny_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
```

**Dependency:** T02 supplies the monotone state and cache identity.

**Intended commit:** `feat(layout): settle block scrollbar reservations`.

### 4.4 `P01/I05/S01/C03/T04` Shared Block Contribution And Retained Child Geometry
**Files:** `src/scroll.rs`, `src/block.rs`, `src/block_tests.rs`, and focused root
integration tests.

**Outcome:** Replace block's local rectangle accumulator and child projection
helpers with the shared C02 accumulator. Feed final direct lines, in-flow blocks,
current floats, inline boxes, and current absolute children exactly once; retain
each performed child's canonical geometry/target; propagate translated nested
intervals only on used-visible physical axes; and derive `content_size`
independently from the canonical anchor/overflow union.

**RED:** Add `fri05_c03_block_contribution_`, `fri05_c03_block_nested_`, and
`fri05_c03_block_negative_margin_` tests first. They fail because negative
margins construct inverted synthetic rectangles, zero on one axis erases the
other, nested geometry is projected from a full rectangle, and local helpers
duplicate interpretation.

**Acceptance:** The named negative-margin families complete without panic;
positive border area plus only positive margin outsets is exact; `0xN` and `Nx0`
descendant-only intervals survive independently; visible/clip and clip/visible
propagation is axis-specific; hidden/scroll/auto trap; lines, floats, in-flow,
inline, and current absolute children contribute once; terminal padding is
retained; every performed block-owned child has canonical target geometry; and
root/nested `content_size` matches the axis-independent union.

**Commands:**
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c03_block_contribution_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c03_block_nested_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c03_block_negative_margin_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
```

**Dependency:** T03 supplies final effective boxes and stable pass ownership.

**Intended commit:** `fix(layout): unify block scroll contribution geometry`.

### 4.5 `P01/I05/S01/C03/T05` Root Rounding Cache And Legacy-Path Closure
**Files:** `src/scroll.rs`, `src/output.rs`, `src/compute.rs`, `src/block.rs`,
`src/lib.rs`, and focused root, leaf, block, cache, rounding, contract, and static
surface tests.

**Outcome:** Make viewport and flex-item root publication preserve or canonically
rebuild complete root geometry and target from source facts; rebuild rounded
geometry only through retained canonical sources; keep the temporary scrollbar
field synchronized for migrated producers; and remove the C03 migration adapter,
legacy root/block accumulator/projection/union/constructor paths, obsolete error,
and compatibility wrappers. Reconcile aggregate C03 front-door and cache proof
without changing flex/grid internals.

**RED:** Add `fri05_c03_root_geometry_`, `fri05_c03_round_cache_`, and
`fri05_c03_root_block_legacy_absence_` tests first. They fail because root and
flex-item-root publication can rebuild or drop legacy geometry, rounding treats
derived values independently, cached output can omit the target, and legacy
root/block symbols remain.

**Acceptance:** Root, measured leaf, and block boxes always publish coherent
geometry/target; controls, hidden, and measurement-only outputs remain absent;
ordinary signed ranges and every helper agree before/after rounding across all
ten flows; cached/uncached results retain used axes and target metadata; final
output staging is stable-pass-only; root/block files have no legacy geometry
construction or local contribution path; the removed public symbols and
`ScrollRectOf::new`/`ScrollUnsupportedFeature` are absent. The temporary mutable
scrollbar field remains only at unmigrated flex/grid write sites and is assigned
to C05.

**Commands:**
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c03_root_geometry_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c03_round_cache_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c03_root_block_legacy_absence_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c03_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
```

**Dependency:** T04 completes migrated block source facts and child geometry.

**Intended commit:** `fix(layout): close root and block scroll geometry paths`.

### 4.6 `P01/I05/S01/C03/T06` Padding Seed And Top-Gutter Integration Closure
**Files:** `src/scroll.rs`, `src/block.rs`, and focused leaf, block, root,
rounding, and cache tests.

**Outcome:** Enforce D-08 in direct measured-leaf and block production by seeding
the shared accumulator with the canonical own padding box while retaining direct
content and terminal padding exactly once. Make the absolute containing area
start after an effective top gutter with the same saturating clamp used for the
left edge, without changing flex/grid internals or later positioned behavior.

**RED:** Add `fri05_c03_integration_padding_seed_` and
`fri05_c03_integration_absolute_top_gutter_` tests first. They fail because
direct block/leaf paths seed from the gutter-inset scrollport and the absolute
physical y area starts at the border edge when a valid flow maps a gutter to top.

**Acceptance:** Direct measured-leaf and block/root front doors distinguish the
padding box from the scrollport under forced, stable, and both-edge gutters;
overflow, signed ranges, helpers, rounding, and cache output remain coherent in
both scalar lanes and representative mappings for every physical gutter side.
An absolute child laid out with a top gutter starts after the gutter, uses the
reduced containing area, and contributes its final margin area exactly once.
Small boxes clamp both area origins without inversion. Existing C03 evidence and
the bounded flex/grid bridge remain unchanged.

**Commands:**
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c03_integration_padding_seed_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c03_integration_absolute_top_gutter_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c03_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
```

**Dependency:** T05 and the holistic findings against candidate `c5b20b354`.

**Intended commit:** `fix(layout): preserve padding seeds and top gutters`.

## 5 Cycle Acceptance
1. All six task ranges have genuine RED/GREEN evidence, clean independent task
   reviews, and coordinator-rerun acceptance commands.
2. Root, measured leaf, and block use one canonical source factory, one shared
   accumulator, flow-aware effective edges, ordinary origins, and required target
   geometry with no alternate root/block derivation.
3. Stable/forced/both-edge/auto behavior, cross-axis induction, tiny boxes,
   negative margins, partial axes, zero-area descendants, lines, floats, in-flow,
   inline, and current absolute children pass through real front doors without
   panic, unsupported capability, duplicate contribution, or speculative output.
4. Output helpers, target presence, physical ranges, rounding, and cache equality
   agree with canonical source facts in both scalar lanes and all ten flows.
5. `BLOCK-001`, `BLOCK-002`, the root/block portions of `OVERFLOW-002` and
   `OVERFLOW-003`, and the C03 portion of `CORE-006` are traceable to focused
   evidence; flex/grid and final initiative closure remain assigned downstream.
6. Root/block legacy facts, arbitrary constructors, local accumulation, and
   compatibility adapters are absent. Only the explicitly bounded flex/grid
   reservation and mutable output bridge remains for C04/C05.
7. Normal and generator-feature verification pass with no generator execution or
   parser, fixture, generated artifact, manifest, report, provenance, dependency,
   feature, MSRV, docs, root, sibling, lint-suppression, or unsafe delta.

## 6 Final Verification
```sh
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
git diff --check
git ls-files -co --exclude-standard -- '*.rs'
rg -n --pcre2 '#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{' $(git ls-files -co --exclude-standard -- '*.rs')
```

The first Git command is the explicit tracked and non-ignored owned-Rust
manifest. The scan consumes exactly that manifest; every textual match is
classified and no executable match may remain. Static range inspection also proves no added
`#[allow]`/`#[expect]`, no changed generator/fixture/artifact input, and no C04+
formatting implementation. The exact cycle inventory contains only the revised
FRI-05 specification and sequence, this plan, and the named Rust source/tests,
including `src/cache.rs`.

## 7 Handoff And Blockers
The completed, reviewed, published, and remotely read-back cycle hands C04 one
canonical root/leaf/block output contract, shared contribution behavior,
monotone pass/cache state, and bounded remaining flex/grid bridges. C04 may
integrate flex; it may not create another geometry, accumulation, origin, or auto
state path.

A genuine blocker exists only if root/leaf/block cannot consume the reviewed C02
source factory without changing a D-03-D-11 product decision, if stable-only
publication cannot be represented inside the existing compute/cache ownership,
or if completion requires a dependency, unsafe code, generator/fixture change,
generator architecture, later-cycle algorithm, or external-repository mutation.
Such evidence returns to the affected planning review rather than widening C03.
