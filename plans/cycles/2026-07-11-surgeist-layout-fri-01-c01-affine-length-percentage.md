# FRI-01-C01 Storeless Affine Length-Percentage Values

Status: draft

Cycle ID: `FRI-01-C01`

Owning repository: `surgeist-layout`

Cycle base: `490a47305da5165dbe7241cdfa28c717616a8138`

Reviewed specification:
`plans/specs/2026-07-11-surgeist-layout-fri-01-compute-resolution-diagnostics.md`
at `e339511ec2938610080b2351cf964e9d0302313da263b9f48a7bde8a5505b795`
sections `FRI-01.1`, `FRI-01.2`, `FRI-01.3`, `D-01`, `D-02`, `D-03`,
`FRI-01.5`, `FRI-01.6`, calc portions of `FRI-01.12`, `FRI-01.16`,
`FRI-01.18`, and `FRI-01.19`.

Reviewed sequence:
`plans/sequences/2026-07-11-surgeist-layout-fri-01-compute-resolution-diagnostics.md`
at `8f59ee47df7312a01a4d7daeb37e78a8f0f35aefd567dcba5fedbf77a9cad715`,
entry `FRI-01-C01`.

Bounded outcome: storeless affine length-percentage values replace calc IDs,
stores, resolver traits, and resolver-free calc panic paths in the value model
and direct value consumers. Cache-context generation removal is sequenced in
`FRI-01-C02`.

## Boundary

This cycle owns `src/value.rs`, calc-related reexports in `src/lib.rs`, direct
value-resolution consumers in block, flex, grid, grid-lanes, and layout-owned
tests or parity support needed to compile and prove the affine value model.

It does not change cache storage or cache context generation, measurement
provider input, numeric scrollbar or flex-factor wrappers, public root
request/session/batch/error APIs, docs, MSRV, root adapters, root API artifacts,
or sibling repositories. Existing public compute shape may remain until
`FRI-01-C03`.

Current evidence: source exposes `CalcId`, `CalcGeneration`, `CalcResolver`,
`NoCalcResolver`, `LayoutCalcStore`, `CalcExpression`, `CalcTerm`, and calc
variants across `LengthOf`, `LengthAutoOf`, `DimensionOf`, track sizing,
algorithm helpers, tests, and browser-parity support.

## Impacts

Public API: breaking pre-release removal of calc identity/resolver/store types
and replacement with `LengthPercentageOf<S>`, `PercentageBasisOf<S>`, and
`NumericResolutionOf<S>`.

Dependencies/features/artifacts/docs/MSRV/root: no dependency, feature,
generated artifact, documentation, MSRV, or root change in this cycle; root
handoff is noted after publication.

Unsafe: no Surgeist-owned unsafe may be added or retained.

## Tasks

| Task | Files/area | Intended behavior/outcome | RED evidence | Acceptance criteria | Commands | Depends on | Intended commit |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `C01-T1` | `src/value.rs`, focused value tests | Add private-field `LengthPercentageOf<S>`, `PercentageBasisOf<S>`, `NumericResolutionOf<S>`, and finite construction/resolution behavior without changing existing value-family callers yet. | Focused tests fail because the new affine types, invalid basis rejection, missing-basis outcome, signed-zero canonicalization, and overflow outcome do not exist. | Tests cover `f32` and `f64`, px, percent, mixed coefficients, negative percent, zero canonicalization, invalid coefficients, invalid basis, missing basis only when needed, and overflow to `InvalidNumeric`. | `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout value -- --nocapture`; `cargo fmt --check` | Reviewed spec and sequence | `value: add affine length percentage model` |
| `C01-T2` | `src/value.rs`, `src/lib.rs`, value tests | Replace current calc variants and calc identity/store/resolver public surface with `Value(LengthPercentageOf<S>)` across `LengthOf`, `LengthAutoOf`, `DimensionOf`, and track-sizing value helpers while preserving non-calc keyword behavior. | Tests that construct calc through old ID/store APIs fail or are replaced by tests proving old resolver-free calc paths cannot exist. | `CalcId`, `CalcGeneration`, `CalcResolver`, `NoCalcResolver`, `LayoutCalcStore`, `CalcExpression`, and `CalcTerm` are absent from public exports and `src/value.rs`, `src/lib.rs`, and migrated value tests; value-family tests cover construction, conversion, basis dependence, percent fraction, and resolution outcomes. | `rg -n "CalcId|CalcGeneration|CalcResolver|NoCalcResolver|LayoutCalcStore|CalcExpression|CalcTerm" src/value.rs src/lib.rs src/lib_tests.rs src/contract_tests.rs`; `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout value -- --nocapture`; `cargo fmt --check` | `C01-T1` | `value: remove calc identity surface` |
| `C01-T3` | `src/block.rs`, `src/flex.rs`, `src/grid/**`, direct algorithm tests | Update direct consumers to use affine value resolution and explicit percentage basis outcomes without resolver parameters or no-calc sentinels. | Focused block, flex, grid, and grid-lane tests fail on the cycle base because calc-bearing public paths panic, degrade to zero, or require a resolver. | Block/flex/grid/grid-lane calc paths compile without resolver traits; missing basis and invalid numeric outcomes are handled by the current operation's temporary cycle-local policy without panics; later-FRI behavior is not claimed. | `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout calc -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout grid_lanes -- --nocapture`; `cargo fmt --check` | `C01-T2` | `layout: consume affine length percentage values` |
| `C01-T4` | `tests/layout/browser_parity/support.rs`, calc fixture tests, source search | Remove layout-local calc-store parsing/resolution from parity support and construct affine layout values directly from checked-in calc XML attributes. | Active calc fixture families fail on the cycle base before comparison because measured leaves or lane paths require resolver composition. | Calc fixture parsing uses no resolver/store/type identity; all active calc fixture families reach comparison or fail only on later-FRI geometry findings; final source search finds no obsolete value-level calc identity/resolver/store types in C01-owned source and tests. Cache `CalcGeneration` and cache-context uses are explicitly deferred to `FRI-01-C02`. | `SURGEIST_PARITY_FILTER=block/block_calc_width_margin CARGO_NET_OFFLINE=true cargo test -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored`; `SURGEIST_PARITY_FILTER=flex/flex_calc_basis_margin_gap CARGO_NET_OFFLINE=true cargo test -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored`; `SURGEIST_PARITY_FILTER=grid/grid_calc_track_and_item_margin CARGO_NET_OFFLINE=true cargo test -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored`; `rg -n "CalcId|CalcResolver|NoCalcResolver|LayoutCalcStore|CalcExpression|CalcTerm" src tests`; `cargo fmt --check` | `C01-T3` | `tests: migrate calc fixtures to affine values` |

## Completion

Cycle acceptance:

1. obsolete value-level calc identity/resolver/store types are absent from
   C01-owned source and tests, with cache `CalcGeneration` removal deferred to
   `FRI-01-C02`;
2. `LengthPercentageOf<S>`, `PercentageBasisOf<S>`, and
   `NumericResolutionOf<S>` satisfy the reviewed construction and resolution
   contract for both scalar modes;
3. current length, length-auto, dimension, and track value helpers consume affine
   values without resolver-free panic paths;
4. active calc browser fixture families reach comparison through layout-owned
   support; and
5. root handoff notes that root must lower style calc into affine coefficients
   after this cycle is published.

Final command list:

```sh
CARGO_NET_OFFLINE=true cargo test -p surgeist-layout
CARGO_NET_OFFLINE=true cargo test -p surgeist-layout --features layout-golden-generate
SURGEIST_PARITY_FILTER=block/block_calc_width_margin CARGO_NET_OFFLINE=true cargo test -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored
SURGEIST_PARITY_FILTER=flex/flex_calc_basis_margin_gap CARGO_NET_OFFLINE=true cargo test -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored
SURGEIST_PARITY_FILTER=grid/grid_calc_track_and_item_margin CARGO_NET_OFFLINE=true cargo test -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored
CARGO_NET_OFFLINE=true cargo clippy -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
cargo fmt --check
git diff --check
bash -lc 'mapfile -t tracked < <(git ls-files "*.rs"); mapfile -t untracked < <(git ls-files --others --exclude-standard "*.rs"); files=("${tracked[@]}" "${untracked[@]}"); test "${#files[@]}" -gt 0; rg -n --pcre2 '\''#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{'\'' "${files[@]}"; status=$?; test "$status" -eq 1'
```

Required handoff: after publication, report the published SHA and root-facing
calc contract change. No sibling or root edits occur in this cycle.

Genuine blockers: if an active calc fixture still fails because it exercises a
later-FRI geometry finding after resolver/store removal is complete, record the
exact fixture, observed failure, and owning later FRI before narrowing the cycle
acceptance through a reviewed plan revision.
