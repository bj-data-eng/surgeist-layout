# FRI-05-C07 Public Evidence And Leaf Candidate Closure
Status: reviewed
Cycle ID: `FRI-05-C07`
Owning repository: `surgeist-layout`
Cycle base: `91866fd0c68796a71bf739c0e5155cbc420beefe`

Reviewed specification: `plans/specs/2026-07-16-surgeist-layout-fri-05-overflow-scroll-geometry.md`
at SHA-256 `747dcd6c12ae7d883999b5517572d6877d3c803bdb611143af7affc5afd44f39`,
commit `50c83f01ded0fe4a284e087ffcbd677bfc12af2a`, sections `FRI-05.5`,
`FRI-05.8` through `FRI-05.10`, `FRI-05.12`, `FRI-05.14`, and
`FRI-05.15`.

Reviewed sequence: `plans/sequences/2026-07-16-surgeist-layout-fri-05-overflow-scroll-geometry.md`
at SHA-256 `6a4fc9a417ff78a0a2c0b9335be514449dcc8a6979aba4259691d2a454a80e57`,
commit `a0aa010b185587cae56bbfc9b035783e4849c203`, entry `FRI-05-C07`.

## Outcome
Reconcile the implemented overflow and scroll public front door with aggregate
compile/static evidence, align crate and parity documentation with the finite
layout-ready and fixture boundaries, prove all ten FRI-05 finding rows and all
14 initiative acceptance items from current source/tests/artifacts, and publish
the final independently reviewed leaf candidate with the exact breaking root
integration handoff.

## Boundary
FRI-05-C01 through C06 are complete, published, and remotely read back. At the
cycle base the public input and output types are reexported, detailed per-cycle
contract and negative-surface tests pass, eleven settled HTML sources own 44
generated outputs, the canonical report records 5,324 passed and 356
unsupported tuples with no failure bucket, and the frozen corpus manifest hash
is `bc39d26ba27e64c85b743c577f20b3cb290fe78326432ad6210f2c2b44e5fbb1`.
The remaining work is aggregate closure evidence and durable public/fixture
documentation; no layout behavior or artifact derivation remains.

This cycle owns focused aggregate tests in `src/lib_tests.rs`; crate-level and
public rustdoc limited to `src/lib.rs`, `src/node_input.rs`, `src/output.rs`, and
`src/scroll.rs`; `README.md`; and
`tests/layout/browser_parity/README.md`. A missing specification-required crate
root reexport discovered by the aggregate characterization may be added only to
`src/lib.rs`. No other production behavior, representation, algorithm, or
private helper may change.

Parser/helper/serializer code, JavaScript/CSS helpers, HTML, XML, reports,
provenance, `corpus.toml`, generator code or architecture, browser runtime,
dependencies, features, MSRV, lockfile, task-runner recipes, root, siblings,
expected failures, quarantines, and the FRI-13 aggregate release gate are
read-only. No scoped or full generation command is authorized: C06 inputs and
derived artifacts are settled, and scoped runs would be diagnostic rather than
verification evidence even if generation were in scope.

Impacts: public API and behavior - unchanged unless a required reexport is
proven missing; docs/examples and compile/static assurance - completed;
dependencies/features/MSRV/generated artifacts - none; root - immutable handoff
only; safety - every tracked and non-ignored owned Rust file remains free of
executable `unsafe` and no lint suppression is added.

## Existing Closure Map
| Finding | Current implementation and focused evidence | Artifact or closure evidence |
| --- | --- | --- |
| `BLOCK-001` | Shared positive-outset accumulation and block negative-margin front-door tests | Named browser family completes without geometry error |
| `BLOCK-002` | Proportional gutter saturation and tiny block/root box tests | Saturated content geometry remains zero rather than failing |
| `GRID-011` | Hidden, scroll, and auto automatic-minimum tests in ordinary grid and lanes | Grid fixture and complete focused grid gates pass |
| `OVERFLOW-001` | Retained in-flow/current-absolute geometry and used-visible-only transitive propagation across block, flex, grid, subgrid, and lanes | Nested fixture families and composed C03-C05 gates pass |
| `OVERFLOW-002` | Complete input model, computed/used phases, canonical factory/range/clip/gutter/target output, and legacy removal | All eleven FRI-05 sources lower and compare through active corpus evidence |
| `OVERFLOW-003` | Independent axis accumulator plus `0xN`/`Nx0` propagation and trapped-value tests in every owned format | Flex/grid zero-axis fixture outputs pass |
| `OVERFLOW-005` | Finite rect construction, private derived carriers, canonical coherence properties, and retained coordinate validation | Public/static negative-surface evidence is aggregated in C07 |
| `CORE-006` | `content_box_size()` and derived `scrollbar_size()` agree with canonical geometry for ordinary, both-edge, and saturated cases | Root/cache/rounding publication tests pass |
| `GRID-009` | Ordinary-grid and lanes extents include the final container-local origin | Browser-backed non-zero-origin fixture passes |
| `TEST-002` | Comparator consumes range spans and rejects wrong x, wrong y, and missing geometry | All 44 generated FRI-05 outputs compare, including explicit zero spans |

## Task
### `C07-T1` Close Aggregate Public Surface And Documentation Evidence
Files: `src/lib_tests.rs`; crate-level and public rustdoc only in `src/lib.rs`,
`src/node_input.rs`, `src/output.rs`, and `src/scroll.rs`; `README.md`; and
`tests/layout/browser_parity/README.md`. `src/lib.rs` reexport declarations may
change only if the characterization proves a required FRI-05 export absent.

Outcome: add one focused `fri05_c07_public_surface_` characterization that
composes the complete default-scalar and `*Of<f64>` crate-root input, error, and
read-only output surface and fails closed on every removed phase-unsafe legacy
surface. Complete public docs for normalized computed overflow, explicit
scrollbar environment input, immutable canonical geometry, signed range spans,
nested target metadata, and the root-owned live offset/style/runtime boundary.
Document the exact finite parity adapter without presenting it as authored CSS
parsing or claiming snap selection/runtime behavior.

Baseline/RED: all existing FRI-05 focused evidence passes, but there is no one
C07 aggregate characterization and the two READMEs do not yet describe the
FRI-05 normalized scroll contract. Add the characterization and compiled
examples before prose. A genuine missing required reexport is the only allowed
behavioral RED; correct it narrowly without adding a constructor, compatibility
alias, legacy field, deferred capability, or new public phase.

Acceptance:
1. The crate root exposes every D-01 through D-04 input, error, coordinate,
   clip, range, geometry, gutter, and target type in default and generic forms
   where scalar-bearing; the public characterization uses checked constructors
   and read-only carrier signatures without manufacturing derived geometry.
2. Static/compile-fail evidence proves no raw overflow point, public mutable
   scrollbar field, public derived-geometry/gutter constructor or `Default`,
   legacy exposure/axis/facts carrier or conversion, unsupported-feature or
   coupling policy, deferred variant, or public live-offset state remains.
3. Public docs distinguish computed and used overflow, atomic canonical pairs,
   finite layout-ready scroll inputs, immutable physical geometry, signed
   zero-anchored range spans, and nested target metadata from root-owned
   authored CSS, style resolution, retained association, transforms, current
   offsets, snapping, CSSOM, host UI, and events.
4. The crate README describes explicit normalized inputs and derived output
   helpers without claiming root runtime support. Crate rustdoc and examples
   compile with warnings denied.
5. The parity README identifies the bounded FRI-05 attributes and computed-style
   lowering, `scroll_size` as canonical physical range spans including zero,
   the exact one-full-run ownership rule, and the post-C06 read-only artifact
   state; it does not widen the parser or generator architecture.
6. The ten-row closure map and all 14 FRI-05 acceptance items are backed by
   current source, tests, docs, artifacts, and final gates without claiming
   root integration or the FRI-13 aggregate release gate.
7. No parser, helper, fixture, generated artifact, manifest, report, generator,
   production algorithm, dependency, feature, MSRV, ignored test, expected
   failure, root, sibling, or unrelated file changes.

Commands:
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c07_public_surface_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --doc
CARGO_NET_OFFLINE=true RUSTDOCFLAGS='-D warnings' cargo doc --locked --offline --no-deps -p surgeist-layout
CARGO_NET_OFFLINE=true just fmt-check
git diff --check
```
Dependency: published and remotely verified C06 candidate at the cycle base.
Intended commit: `docs(layout): close FRI-05 scroll contract`.

## Cycle Acceptance
1. C07-T1 has a clean independent task review and its compiled examples agree
   with the final crate-root exports and private derived-carrier construction.
2. All ten `FRI-05.14` finding rows have implementation, focused test, and where
   applicable generated browser evidence in the closure map.
3. Every `FRI-05.15` acceptance item is satisfied without claiming upstream
   CSS/style/root work, later layout initiatives, or the FRI-13 aggregate gate.
4. Default and generator-feature verification, all FRI-05 focused tests,
   44-output focused parity, corpus/Taffy checks, docs, rustdoc, formatting,
   Clippy, diff/scope/provenance review, and repository-wide unsafe absence pass.
5. No C06 input or artifact changes and no generator run enter this cycle.
6. The exact cycle range is holistic-clean before immutable-SHA publication,
   fresh remote-main readback, and the final leaf candidate handoff.

## Final Verification
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri05_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout fri05_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --doc
CARGO_NET_OFFLINE=true RUSTDOCFLAGS='-D warnings' cargo doc --locked --offline --no-deps -p surgeist-layout
git diff --check 91866fd0c68796a71bf739c0e5155cbc420beefe
git diff --name-only 91866fd0c68796a71bf739c0e5155cbc420beefe
git ls-files -co --exclude-standard -- '*.rs'
! git ls-files -co --exclude-standard -z -- '*.rs' | xargs -0 rg -n --pcre2 '#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{'
git status --short
```

The owned-Rust manifest and unsafe scan cover every tracked and non-ignored Rust
file. Inspect the exact name-only inventory and prove parser/helper/serializer
code, JS/CSS/HTML/XML, reports, manifest, generator, production algorithms,
dependencies/features/MSRV, task runner, root, and siblings are absent. All
final commands are read-only and no browser or generation command is permitted.

## Handoff And Blockers
After publication, emit `SURGEIST_HANDOFF: CRATE_CANDIDATE` for
`surgeist-layout-fri-05-c07` with all seven cycle candidates, exact reviewed
planning revisions, ordered task ranges, characterization/GREEN evidence,
final commands, task and holistic verdicts, the ten-finding closure map,
artifact inventory and manifest hash, unsafe proof, push/readback SHAs, and
temporary-resource cleanup.

The breaking root handoff is exactly specification section `FRI-05.12`: publish
separate CSS and style candidates for authored grammar and resolver-owned
normalized values, promote those two candidates plus this immutable layout SHA,
lower only through typed style accessors and an explicit scrollbar environment,
migrate every removed leaf surface, preserve root ownership of live offsets,
association, transforms, host policy, snap selection, CSSOM, UI, and events,
then regenerate and check the root-owned API artifacts. Do not edit root or
duplicate that contract here.

A genuine blocker is a public-surface contradiction, focused FRI-05 failure,
artifact/count/hash drift, unsafe or lint-suppression match, required generator
or root change, or out-of-bound diff. Return to planning or implementation
without adding a compatibility path, weakening evidence, running generation,
or expanding generator architecture.
