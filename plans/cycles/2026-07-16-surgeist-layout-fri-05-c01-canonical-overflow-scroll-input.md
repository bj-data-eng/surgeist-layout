# FRI-05-C01 Canonical Overflow And Scroll Input
Status: draft
Cycle ID: `FRI-05-C01`
Owning repository: `surgeist-layout`
Cycle base: `f479b5e5d23294eafb82f8ae7ee6c740ead752d0`
Reviewed specification:
`plans/specs/2026-07-16-surgeist-layout-fri-05-overflow-scroll-geometry.md`
at SHA-256
`b2bc5d8cf1f7b65dcef74adf34b5b63ab1f8b519fcbd1094ff4e335ab419286f`,
commit `5a51d0f67ef781eef724f86a9232bc3616c3773f`, sections `FRI-05.4 D-01`
and `D-02`, the input rows of `FRI-05.5`, the computed/used and invalid-input
rows of `FRI-05.6`, the computed-overflow, property-model, and input portions
of the public-surface evidence in `FRI-05.8`, the `node_input.rs`, `lib.rs`, and
fixture-parser rows of `FRI-05.9`, `FRI-05.10`, and the input portions of
acceptance items 2, 4, 11, and 13 in `FRI-05.15`.
Reviewed sequence:
`plans/sequences/2026-07-16-surgeist-layout-fri-05-overflow-scroll-geometry.md`
at SHA-256
`2652a3a247aab69d005c0f5dbfd0a1fff002f00eb490b838ed0959b49dfa2524`,
commit `f479b5e5d23294eafb82f8ae7ee6c740ead752d0`, entry `FRI-05-C01`.
## Outcome
Replace the raw mutable overflow point with the validated thirteen-pair
computed model, add every closed layout-ready D-02 input and default, expose
phase-correct computed and private used predicates, migrate `NodeInputOf` and
all direct callers, and bridge the checked-in authored overflow attributes at
the fixture parser boundary with the exact finite CSS coupling rule.
## Boundary
At the cycle base, `Overflow` omits `Auto`; `NodeInputOf::overflow` is a public
`Point<Overflow>`; public per-axis predicates conflate computed and used phases;
the D-02 fields are absent; and the parity parser mutates axes independently,
maps `auto` to `Scroll`, and consumes 96 explicit cross-group node pairs in 24
XML files plus omitted-axis cases.
This cycle owns `src/node_input.rs`, input reexports in `src/lib.rs`, the minimum
existing `src/scroll.rs` adaptations required by the new input type, all direct
production and test constructions needed for the public break, and only the
overflow construction path and focused tests in
`tests/layout/browser_parity/support.rs`.
The temporary fixture transition parses the same five tokens faithfully,
treats an omitted authored axis as initial `Visible`, changes `Visible` to
`Auto` and `Clip` to `Hidden` only when the opposite axis is scrollable, then
calls `ComputedOverflow::try_new` once. Production layout exposes no specified
value normalizer. C06 removes this transition when computed-style lowering
lands.
This cycle does not add canonical rectangles, clips, gutters, ranges, targets,
accumulation, rounding, cache publication, auto-gutter iteration, format-local
integration, final flex/grid axis routing, helper or serializer changes, new
fixture tokens, HTML, manifest records, XML, reports, provenance, docs, root or
sibling work, dependencies, features, MSRV changes, unsafe code, or generator
architecture. Existing legacy output geometry remains for C02 through C05.
No generation command is applicable: the fixture consumer changes, but every
generation input and derived artifact remains unchanged. `just verify-generator`
is a compile/test gate, not regeneration. The sole full regeneration remains
C06 after its inputs settle. `just parity-all` remains FRI-13-owned.
## Impacts
Public API: intentional breaking replacement of the raw overflow point and
phase-unsafe predicates, plus additive `Auto`, `ComputedOverflow`, and D-02
input types, aliases, accessors, and `NodeInputOf` fields. The input-deferred
`ScrollUnsupportedFeature` variants and `ScrollOverflowCouplingPolicy` are
removed; C02 retains ownership of geometry-error replacement. Root migration
remains the later FRI-05 handoff.
Dependencies, features, generated artifacts, docs, examples, MSRV, root, and
siblings: unchanged. Safety: all owned Rust remains unsafe-free.
## Tasks
### `C01-T1` Canonical Computed Overflow Model
**Files:** `src/node_input.rs`, `src/scroll.rs`, `src/lib.rs`, and focused model
and public-contract tests.
**Outcome:** Add `Overflow::Auto`, `ComputedOverflow`, its exact construction
error, constant/default, private fields, accessors, computed scrollability and
pair-level independent-formatting-context predicate. Adapt only the exhaustive
existing overflow match required to compile; leave the private used phase and
`NodeInputOf` migration to T3.
**RED:** Add tests with the `fri05_c01_computed_overflow_` prefix first. They
fail because `Auto`, `ComputedOverflow`, and the typed error do not exist.
Record the expected compile/test failure before implementation.
**Acceptance:** An exhaustive 25-pair table proves exactly 13 accepted and 12
rejected pairs, `VISIBLE`, default, accessors, equality/debug/copy behavior,
and atomic errors. All five values prove computed scrollability and complete-pair
block behavior without a used type, normalizer, dead-code exception, or T3 work.
**Commands:**
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c01_computed_overflow_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
```
**Dependency:** Clean sequence and cycle base.
**Intended commit:** `api(layout): add canonical computed overflow`.
### `C01-T2` Closed Scroll Property Input Domains
**Files:** `src/node_input.rs`, `src/lib.rs`, and focused model and public-contract
tests.
**Outcome:** Add the exact D-02 clip-box/margin, gutter, physical padding,
physical signed margin, snap type, snap alignment, and snap-stop domains with
default-scalar aliases, private invariant-bearing fields where specified,
atomic constructors, accessors, and real initial defaults. Do not add them to
`NodeInputOf` until T3.
**RED:** Add tests with the `fri05_c01_scroll_input_` prefix first. They fail
because the D-02 types and aliases are absent. Record the expected failure.
**Acceptance:** `f32` and `f64` tests cover every closed enum state, clone/debug/
equality, all defaults, finite and signed-zero handling, negative/non-finite
clip-margin rejection, finite signed margin edges with edge-specific failure,
padding `Auto` and length-percentage construction, missing/invalid resolution,
negative used-value clamping, and explicit block/inline snap alignment. No
output geometry, runtime snap choice, heuristic padding, or deferred placeholder
is introduced.
**Commands:**
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c01_scroll_input_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
```
**Dependency:** `C01-T1` supplies the overflow phase conventions.
**Intended commit:** `api(layout): add scroll property input domains`.
### `C01-T3` Canonical Node Input And Legacy Fixture Transition
**Files:** `src/node_input.rs`, `src/lib.rs`, direct block/flex/grid/scroll/
compute callers and their tests/support, plus
`tests/layout/browser_parity/support.rs`.
**Outcome:** Change `NodeInputOf::overflow` to `ComputedOverflow`; add all D-02
fields and exact defaults; migrate every owned construction and accessor; remove
public phase-unsafe per-axis predicates; add and consume the crate-private used
pair with replaced-`Hidden` conversion; remove the eight input-deferred
`ScrollUnsupportedFeature` variants, `is_phase_one_deferred`,
`ScrollOverflowCouplingPolicy`, its reexport, and related tests; and add only the
bounded legacy fixture coupling before one atomic computed-overflow construction.
**RED:** Add `fri05_c01_node_input_` field/default/public-surface tests and
`fri05_c01_legacy_overflow_` parser tables first. They fail because the node
field remains mutable, D-02 fields and the private used phase are absent, the
deferred public surface remains, and cross-group fixture values cannot construct
canonical input. Record both expected failures.
**Acceptance:** Default and generic inputs prove every exact field type and
initial value. No owned source retains `Point<Overflow>`, mutable axis writes,
the removed public per-axis methods, input-deferred variants/policy/reexport, or
their public construction paths; compile-fail and static tests prove those
surfaces absent. All five used values prove ordinary identity,
replaced-hidden conversion, clipping, range, and gutter classification through
migrated consumers. Existing direct tests use already computed canonical pairs
without a production normalizer. The parser table covers all 25 authored pairs,
every omitted-axis orientation, exact `Auto`, and invalid tokens; all checked-in
XML, including the 96 explicit legacy pairs, recursively lowers every input node
without changing helper, serializer, HTML, manifest, XML, report, or provenance
bytes. Existing geometry tests remain green without claiming C02-C05 integration.
**Commands:**
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c01_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
```
**Dependency:** `C01-T2` completes every field domain before the atomic public
input migration.
**Intended commit:** `api(layout): migrate canonical scroll inputs`.
## Cycle Acceptance
1. All three task ranges satisfy RED/GREEN evidence and independent clean task reviews.
2. Computed and used overflow phases, all 25 pair outcomes, D-02 domains, defaults, and scalar failures match the reviewed matrices.
3. `NodeInputOf` and every owned caller carry only canonical input; no public or fixture path can construct a mixed pair, and no deferred input capability or coupling policy remains.
4. The finite legacy parser transition covers current explicit and omitted-axis cases and remains confined to test support for C06 removal.
5. Normal, generator, and corpus checks pass with no generation run or generated/input artifact delta.
6. C02-C07 geometry, integration, final fixture lowering, generation, docs, and handoff work remain outside the range.
## Final Verification
```sh
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
git diff --check
rg -n --pcre2 '#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{' --glob '*.rs' .
```
The unsafe scan must report no executable match. The complete cycle inventory
must contain no helper, serializer, HTML, manifest, XML, report, provenance,
dependency, feature, MSRV, root, sibling, or documentation change.
## Handoff And Blockers
The completed cycle hands C02 canonical input facts and a private used-overflow
phase, with the bounded fixture transition explicitly assigned for removal in
C06. It does not emit the final FRI-05 root handoff.
A genuine blocker exists only if an existing fixture cannot reach one of the 13
canonical pairs through the specified finite coupling, or if an exact D-02
domain requires a new dependency, unsafe code, generator expansion, or a
product decision absent from the reviewed specification. Such evidence returns
to planning review; it does not authorize a compatibility input or fallback.
