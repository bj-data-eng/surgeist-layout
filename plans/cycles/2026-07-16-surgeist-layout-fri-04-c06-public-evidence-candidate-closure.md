# FRI-04-C06 Public Evidence And Candidate Closure
Status: reviewed
Cycle ID: `FRI-04-C06`
Owning repository: `surgeist-layout`
Cycle base: `d3d7a65cc215c609fd32f5a102e9a30161a3a8a6`

Reviewed specification: `plans/specs/2026-07-15-surgeist-layout-fri-04-property-specific-sizing-values.md`
at SHA-256 `e0116f0e3dd28eafabe1ed31117a61ea208e97dee986887f5600d7cbd5a06db4`,
commit `5d33f3a4ab694f12985d713f7dbc74b251d55fb6`, sections `FRI-04.5`,
`FRI-04.8` through `FRI-04.10`, and all acceptance items in `FRI-04.12`.

Reviewed sequence: `plans/sequences/2026-07-15-surgeist-layout-fri-04-property-specific-sizing-values.md`
at SHA-256 `9a35a5cfef82fb5b6c5abc6fd9beee7c0a080f631fd392d2ffe7694e019c4f8b`,
commit `5543ef5e9273ee73c187803c79191b8b71949fc0`, entry `FRI-04-C06`.

## Outcome
Reconcile the already-implemented public sizing front door with compiled public
examples and exact negative-surface evidence, align crate and fixture docs with
the finite layout-ready grammar, prove every FRI-04 acceptance item from current
source/tests/artifacts, and publish the final independently reviewed leaf
candidate with the complete root integration handoff.

## Boundary
C01-C05 are published and remotely verified. At the cycle base, 44 default
FRI-04 unit tests, nine parser/fixture/inventory tests, and seven generator-feature
FRI-04 tests pass. Required reexports and existing compile-fail examples are
present. The remaining gaps are an aggregate root-front-door example, exhaustive
negative examples for the closed property domains, and documentation that still
describes only the old affine fixture syntax.

This cycle owns only public rustdoc examples in `src/sizing.rs`, focused
root-reexport evidence in `src/lib_tests.rs`, crate-level rustdoc in `src/lib.rs`,
`README.md`, and `tests/layout/browser_parity/README.md`. Reexport declarations,
public symbols, private representations, production behavior, algorithms, and
all prior focused matrices remain unchanged.

Parser/helper/serializer code, HTML, XML, reports, provenance, corpus manifest,
generator code/architecture, browser runtime, dependencies, features, MSRV,
lockfile, task-runner recipes, root, and siblings are read-only. No scoped or
full generation command is authorized: every C05 input and derived artifact is
settled. `just parity-all` remains the FRI-13 aggregate release gate; C06 uses
the complete nonignored 12-output FRI-04 comparison instead.

Impacts: public API and behavior - unchanged; docs/examples and compile-time
assurance - completed; dependencies/features/MSRV/generated artifacts - none;
root - handoff only; safety - all owned Rust remains free of `unsafe`.

## Existing Closure Map
| Contract | Current source and focused evidence | Artifact/root evidence |
| --- | --- | --- |
| `MODEL-005` | `src/sizing.rs`, property fields in `src/node_input.rs`, typed dispatch in `src/compute.rs` and format modules; `sizing_calculation_`, `calc_size_calculation_`, `property_sizing_`, `fri04_c03_`, and `fri04_c04_` tests | Three C05 sources and 12 outputs; root obligations 2-4 and 6-9 |
| `MODEL-007` | Closed box/flex wrappers and track-only `TrackFlexFactorOf`; `property_sizing_`, `track_sizing_`, and compile-fail evidence | Grid C05 source/output family; root obligations 1, 2, 5, and 6 |
| Acceptance 2-4 | C02 construction/default/legacy-removal source and tests | Breaking root migration is retained in the candidate report |
| Acceptance 5-7 | C01 model tests plus C03 numeric/track front doors | Symbolic percentage and validated track-flex obligations remain explicit |
| Acceptance 8-9 | C03/C04 real front doors and exhaustive D-06 table | Later-owner capability payload and owner are retained for root |
| Acceptance 10-12 | C05 parser/helper/serializer/digest/inventory tests and exact owned parity | 1,409 HTML, 5,280 XML, 356 canonical unsupported tuples, zero failures, sole `all.json` |
| Acceptance 13-14 | C06 compiled examples/docs plus final diff, gate, review, unsafe, publication, and readback evidence | Complete immutable-SHA candidate handoff |

## Task
### `C06-T1` Close Public Surface And Documentation Evidence
Files: `src/sizing.rs` rustdoc only, `src/lib_tests.rs`, crate-level `//!`
comments in `src/lib.rs`, `README.md`, and
`tests/layout/browser_parity/README.md`.

Outcome: add one focused `fri04_c06_public_surface_` characterization proving
the crate-root default/generic reexports and checked constructors compose; make
the public rustdoc example exercise ordinary min/max/clamp, property-specific
calc-size, flex `Content`, and validated maximum-track flex; complete negative
rustdoc evidence for every prohibited property-erasing state; and make all three
docs state the property-specific, layout-ready, symbolic-percentage boundary.

Baseline/RED: no production behavior changes. Record that all existing focused
matrices pass, while the named C06 characterization and complete docs predicates
are absent. Add the characterization and compile-pass/fail examples before
editing prose; a genuine public-surface failure becomes the focused RED and must
be corrected without adding compatibility API.

Acceptance:
1. Default and `*Of<f64>` forms for preferred/minimum/maximum/flex, ordinary and
   calc-size calculations, calc-size bases/errors, track flex/track breadths,
   and capability descriptors are usable from the crate root.
2. Compile-fail examples prove no box/flex `fr`, no maximum `Auto`, no non-flex
   `Content`, no property-erasing conversion, no raw-scalar track `fr`, no broad
   public sizing resolver, and no `Dimension`/`DimensionOf` reexport.
3. README and crate rustdoc explain the four property domains, iterative
   min/max/clamp values, canonical calc-size input, unresolved percentages,
   track-only validated flex, exact later-owner capability, and root-owned CSS
   lowering without presenting layout as an authored CSS parser.
4. The parity README documents the depth-64, property-specific fixture grammar:
   finite px/percentage and existing unitless values, affine calc, nested
   min/max/clamp, one-argument fit-content, canonical calc-size, valid keywords,
   and maximum-track-only finite non-negative `fr`.
5. Reexports and production source are unchanged; no generation input, generated
   artifact, ignored test, expected-failure, dependency, feature, or MSRV changes.

Commands:
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri04_c06_public_surface_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --doc
CARGO_NET_OFFLINE=true RUSTDOCFLAGS='-D warnings' cargo doc --locked --offline --no-deps -p surgeist-layout
CARGO_NET_OFFLINE=true just fmt-check
git diff --check
```
Dependency: published C01-C05 candidate at the cycle base.
Intended commit: `docs(layout): close FRI-04 sizing contract`.

## Cycle Acceptance
1. C06-T1 has a clean independent task review and its compiled examples agree
   with the unchanged crate-root reexports and closed property types.
2. The closure map is backed by current source and passing focused evidence;
   `MODEL-005` and `MODEL-007` each map to tests, artifacts, and root obligations.
3. All 14 specification acceptance items are satisfied without claiming any
   later format initiative or the FRI-13 aggregate gate.
4. Default, generator-feature, corpus, Taffy, doc, rustdoc, formatting, lint,
   exact FRI-04 parity, diff, and unsafe gates pass from the completed head.
5. No C05 input/artifact changes and no generator run enter the cycle.
6. The exact cycle range is holistic-clean before immutable-SHA publication and
   fresh remote-main readback.

## Final Verification
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri04_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri04_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --doc
CARGO_NET_OFFLINE=true RUSTDOCFLAGS='-D warnings' cargo doc --locked --offline --no-deps -p surgeist-layout
git diff --check d3d7a65cc215c609fd32f5a102e9a30161a3a8a6
git diff --name-only d3d7a65cc215c609fd32f5a102e9a30161a3a8a6
rg -n --pcre2 '#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{' --glob '*.rs' .
git status --short
```
The unsafe scan must return no executable match. Inspect the exact name-only
inventory and prove generation inputs/artifacts, production code, manifests,
and task-runner files are absent. All final commands are read-only.

## Handoff And Blockers
After publication, emit `SURGEIST_HANDOFF: CRATE_CANDIDATE` for
`surgeist-layout-fri-04-c06` with the six cycle candidates, reviewed planning
revisions, ordered task ranges, RED/characterization and GREEN evidence, all
final commands, reviews, breaking API classification, artifact account, unsafe
proof, push/readback SHAs, and temporary-resource cleanup.

Root actions are exactly: replace legacy dimensions by destination property;
lower computed preferred/min/max/flex/track values through checked APIs;
canonicalize authored calc without resolving percentages; simplify and check
calc-size bases/calculations; construct `fr` only as validated maximum-track
flex; reject style/cascade invalidity before layout; update facade/adapters/docs/
examples/tests; regenerate root-owned API artifacts after pinning the published
leaf; and retain named later-owner capabilities until their format initiatives.
Do not edit root. `FRI-05` is the next ready findings initiative.

A genuine blocker is a public-surface contradiction, focused FRI-04 failure,
artifact/count/digest drift, unsafe match, required generator or root change, or
out-of-bound diff. Return to planning or implementation without adding a legacy
shim, weakening evidence, running generation, or expanding architecture.
