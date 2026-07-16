# FRI-05-C02 Canonical Scroll Geometry Substrate
Status: in_progress
Cycle ID: `FRI-05-C02`
Owning repository: `surgeist-layout`
Cycle base: `a6a6011aedb952572b8c0eac6f2a67b94c219f1a`
Reviewed specification:
`plans/specs/2026-07-16-surgeist-layout-fri-05-overflow-scroll-geometry.md`
at SHA-256
`b2bc5d8cf1f7b65dcef74adf34b5b63ab1f8b519fcbd1094ff4e335ab419286f`,
commit `5a51d0f67ef781eef724f86a9232bc3616c3773f`, sections `FRI-05.4 D-03`
through `D-11`, the output/substrate rows of `FRI-05.5` and `FRI-05.6`, the
canonical-geometry, flow/origin, contribution, zero-area, rounding, target, and
public-surface rows of `FRI-05.8`, the `scroll.rs` and `lib.rs` rows of
`FRI-05.9`, `FRI-05.10`, and the substrate portions of acceptance items 3
through 8, 11, and 13 in `FRI-05.15`.
Reviewed sequence:
`plans/sequences/2026-07-16-surgeist-layout-fri-05-overflow-scroll-geometry.md`
at SHA-256
`2652a3a247aab69d005c0f5dbfd0a1fff002f00eb490b838ed0959b49dfa2524`,
commit `f479b5e5d23294eafb82f8ae7ee6c740ead752d0`, entry `FRI-05-C02`.
## Outcome
Add finite canonical rectangle, clip, target, box, gutter, contribution, range,
source-record, factory, and source-based rounding primitives in both scalar
lanes. Leave the current production geometry path intact so C03 through C05 can
replace each formatting family against one reviewed substrate.
## Boundary
At the cycle base, `ScrollRectOf::new` checks components but not finite ends;
the public geometry stores one optional clip rectangle and caller-supplied
derived parts; gutters can exceed small boxes; accumulation and range origin
logic are format-local; and rounding independently reconstructs derived parts.
C01 supplies canonical computed/used overflow and every D-02 input domain.
This cycle owns `src/scroll.rs`, its intentional additive reexports in
`src/lib.rs`, and focused `src/scroll_tests.rs`, `src/lib_tests.rs`, and
`src/contract_tests.rs` evidence. Private helper names and decomposition may
follow local source conventions, but one private source record and one private
factory own every newly derived canonical value.
The additive public substrate is `ScrollRectErrorOf<S>` and alias,
`PhysicalClipAxisOf<S>` and alias, `OverflowClipOf<S>` and alias, and
`ScrollTargetGeometryOf<S>` and alias. Clip-axis accessors are
`minimum()`/`maximum()`; clip accessors are `x()`/`y()`; target accessors are
`border_box()`, `scroll_margin()`, `flow_axes()`, `snap_align()`, and
`snap_stop()`. Fields are private and none of the output carriers implements
`Default` or exposes a public constructor.
`ScrollRectErrorOf<S>` distinguishes `NonFiniteOrigin`, `NonFiniteSize`,
`NegativeSize`, and `NonFiniteEnd`, each with `PhysicalAxis` and the rejected
value; `NonFiniteEnd` also retains the finite origin and size that overflowed.
`ScrollRectOf::try_new` is the only validation implementation. The existing
`new` remains only as the C03-C05 compatibility wrapper and maps its typed error
to `InvalidScrollRect`; its removal and the legacy geometry/facts constructors,
public `ScrollGeometryOf` replacement, output helper switch, and compatibility
projection removal remain assigned to the integration cycles.
This cycle does not change `NodeOutputOf`, `ComputeOutputOf`, cache publication,
root/leaf/block/flex/grid production calls, auto-gutter layout passes, final
format origins/subjects, browser comparison, fixture parser/helper/serializer,
HTML, manifest, XML, reports, provenance, docs, dependencies, features, MSRV,
root or siblings, unsafe code, or generator architecture. No generation or
`parity-all` command is applicable.
## Impacts
Public API: additive typed rectangle error and read-only clip/target carriers.
The final breaking replacement/removal of legacy geometry surfaces is deferred
exactly to C03-C05; no compatibility alias is added for a new type.
Dependencies, features, generated artifacts, docs, examples, MSRV, root, and
siblings: unchanged. Safety: all owned Rust remains unsafe-free.
## Tasks
### `C02-T1` Finite Rectangles And Read-Only Geometry Carriers
**Files:** `src/scroll.rs`, `src/lib.rs`, and focused model/public-contract tests.
**Outcome:** Add the exact typed rectangle error and `try_new`; canonicalize
signed zero and reject non-finite origin, size, end, or negative size atomically.
Add finite ordered physical clip-axis, per-axis clip, and nested target carriers
with the exact private construction and public accessors in the boundary. Route
legacy `new` through `try_new` without duplicating validation.
**RED:** Add `fri05_c02_rect_` and `fri05_c02_carrier_` tests first. They fail
because finite-end errors and the clip/target carriers are absent.
**Acceptance:** Exhaustive x/y error cases, ordering, signed zero, zero-area
rectangles, finite-end overflow, f32/f64 traits/accessors, target metadata, and
compile-fail/static evidence for no public field construction or `Default` pass.
The public reexports compose and legacy callers retain their mapped error only.
**Commands:**
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c02_rect_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c02_carrier_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
```
**Dependency:** Published C01 candidate.
**Intended commit:** `api(layout): add finite scroll geometry carriers`.
### `C02-T2` Canonical Boxes Clips And Saturated Gutters
**Files:** `src/scroll.rs` and focused substrate tests.
**Outcome:** Add private effective scrollbar-state and edge-reservation facts;
derive border, padding, content, scrollport, physical edge gutters, aggregate
reservation, per-axis overflow clips, resolved scroll padding, and optimal
viewing region from source facts. Select logical roles through `FlowAxes` and
proportionally saturate every opposing requested edge before rectangle inset.
**RED:** Add `fri05_c02_box_clip_gutter_` tests first. They fail because the
canonical derivation and effective-edge model do not exist.
**Acceptance:** Both scalar lanes cover all ten flow mappings, forced/stable/
both-edge/conditional state inputs, zero thickness, one/two opposing gutters,
the `2px`/`15px` case, zero boxes, independently saturated axes, all clip boxes,
clip-margin expansion on only the clipped axis, scrollport clips for hidden/
scroll/auto, visible absence, and non-negative optimal-viewing-region insets.
Auto state is supplied as settled bits; no formatting rerun or cache behavior lands.
**Commands:**
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c02_box_clip_gutter_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
```
**Dependency:** `C02-T1` supplies validated rectangles and clip carriers.
**Intended commit:** `feat(layout): derive canonical scroll boxes`.
### `C02-T3` Shared Contribution And Origin-Aware Range Substrate
**Files:** `src/scroll.rs` and focused accumulator/range tests.
**Outcome:** Add one private physical accumulator with independent x/y bounds,
propagatable descendant intervals, final in-flow ends, and optional alignment
subjects. Add `ScrollOriginAxes` and derive zero-anchored signed ranges from the
complete overflow, bounded active subject, used overflow, scrollport, and flow
projection without consuming a format algorithm.
**RED:** Add `fri05_c02_accumulator_` and `fri05_c02_range_` tests first. They
fail because no shared contribution or format-origin model exists.
**Acceptance:** In both scalar lanes, seed padding, direct line, positive-area
border/positive margin outsets, one current absolute contribution, terminal
padding, negative origins, transitive used-visible intervals, trapped axes, and
zero-area descendant-only propagation are exact and never all-axis short-circuit.
All progression combinations and ten flow mappings prove end/start extents,
safe zero, start/end/center subject bounds, both-sided ranges, reversed signs,
visible/clip zero range, unreachable farther start overflow, and axis independence.
**Commands:**
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c02_accumulator_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c02_range_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
```
**Dependency:** `C02-T2` supplies the canonical scrollport and used-axis clips.
**Intended commit:** `feat(layout): add scroll contribution and range substrate`.
### `C02-T4` Single Canonical Factory And Source-Based Rounding
**Files:** `src/scroll.rs` and focused composition/rounding tests.
**Outcome:** Compose T1-T3 behind one private source record and one private
factory. The private result carries every D-03 container field plus one concrete
`ScrollTargetGeometryOf`; rounding rounds source border/edge/overflow/subject/
target facts in cumulative-origin coordinates and invokes the same factory.
Existing public/production `ScrollGeometryOf` and `round_scroll_geometry` remain
the explicitly noncanonical compatibility path for C03-C05 and do not construct
or mutate the new result.
**RED:** Add `fri05_c02_factory_` and `fri05_c02_rounding_` tests first. They
fail because no single source record, composed result, or rebuild path exists.
**Acceptance:** Both scalar lanes prove all factory invariants, required target
presence, used-axis retention, snap/padding/margin metadata, clips/gutters/range
coherence, no default or alternate constructor, and before/after-rounding
equivalence through all ten flow mappings including finite failure. Static call
accounting finds one new factory and no second derivation or production caller.
**Commands:**
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c02_factory_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c02_rounding_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
```
**Dependency:** `C02-T2` and `C02-T3` complete every source derivation.
**Intended commit:** `feat(layout): add canonical scroll geometry factory`.
## Cycle Acceptance
1. All four task ranges have genuine RED/GREEN evidence and clean independent task reviews.
2. One private source/factory owns every new derived box, clip, gutter, range, target, and rounded value in f32/f64.
3. Rect errors, saturation, contribution, origin, projection, and target behavior match D-03 through D-11 without a panic, unsupported result, guessed context, or alternate constructor.
4. Current production geometry, outputs, caches, and format-local integration remain for C03-C05; no new canonical result is published from a formatting path.
5. Normal and generator-feature verification pass with no generation or generated/input artifact delta.
6. No later integration, comparator, fixture, documentation, root, sibling, dependency, feature, MSRV, lint-suppression, or unsafe change enters the range.
## Final Verification
```sh
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
git diff --check
rg -n --pcre2 '#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^\"]*")?\s*\{' --glob '*.rs' .
```
The unsafe scan covers tracked and non-ignored owned Rust and must report no
executable match. The cycle inventory contains only its plan and the named Rust
source/tests; no generator, fixture, artifact, doc, manifest, or external-repo change.
## Handoff And Blockers
The completed cycle hands C03 one reviewed source-fact factory and additive
public carriers. C03 may integrate root/leaf/block and begin legacy removal; C04
and C05 retain flex/grid integration and final compatibility removal.
A genuine blocker exists only if the reviewed substrate cannot coexist with the
current production path, if a D-03-D-11 invariant requires missing product
authority, or if completion requires a new dependency, unsafe code, generator
change, or later-cycle integration. Such evidence returns to planning review.
