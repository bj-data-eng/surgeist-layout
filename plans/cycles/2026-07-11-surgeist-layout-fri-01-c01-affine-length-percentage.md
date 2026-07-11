# FRI-01-C01 Storeless Affine Length-Percentage Values

Status: in_progress

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

Bounded outcome: calc-capable storeless affine length-percentage values replace
calc IDs, stores, generations, resolver traits, and resolver-free calc panic
paths in the value model, direct value consumers, and current cache context.

## Boundary

This cycle owns `src/value.rs`, calc-related reexports in `src/lib.rs`, current
cache-context generation state in `src/cache.rs` and `src/traits.rs`, direct
value-resolution consumers in block, flex, grid, grid-lanes, and layout-owned
tests or parity support needed to compile and prove the affine value model.

It does not change cache storage shape, staged cache writes, measurement provider
input, numeric scrollbar or flex-factor wrappers, public root request/session/
batch/error APIs, docs, MSRV, root adapters, root API artifacts, or sibling
repositories. Existing public compute shape may remain until `FRI-01-C03`.

Current evidence: source exposes `CalcId`, `CalcGeneration`, `CalcResolver`,
`NoCalcResolver`, `LayoutCalcStore`, `CalcExpression`, `CalcTerm`, and calc
variants across `LengthOf`, `LengthAutoOf`, `DimensionOf`, track sizing,
algorithm helpers, tests, and browser-parity support.

## Impacts

Public API: breaking pre-release removal of calc identity/resolver/store/
generation representation types and replacement with calc-capable
`LengthPercentageOf<S>`, `PercentageBasisOf<S>`, and `NumericResolutionOf<S>`.

Dependencies/features/artifacts/docs/MSRV/root: no dependency, feature,
generated artifact, documentation, MSRV, or root change in this cycle; root
handoff is noted after publication.

Unsafe: no Surgeist-owned unsafe may be added or retained.

## Tasks

| Task | Files/area | Intended behavior/outcome | RED evidence | Acceptance criteria | Commands | Depends on | Intended commit |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `C01-T1` | `src/value.rs`, focused value tests | Add private-field `LengthPercentageOf<S>`, `PercentageBasisOf<S>`, `NumericResolutionOf<S>`, and finite construction/resolution behavior without changing existing value-family callers yet. | Focused tests fail because the new affine types, invalid basis rejection, missing-basis outcome, signed-zero canonicalization, and overflow outcome do not exist. | Tests cover `f32` and `f64`, px, percent, mixed coefficients, negative percent, zero canonicalization, invalid coefficients, invalid basis, missing basis only when needed, and overflow to `InvalidNumeric`. | `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout value -- --nocapture`; `cargo fmt --check` | Reviewed spec and sequence | `value: add affine length percentage model` |
| `C01-T2` | `src/value.rs`, `src/lib.rs`, `src/cache.rs`, `src/cache_tests.rs`, `src/traits.rs`, `src/compute.rs`, `src/block.rs`, `src/flex.rs`, `src/grid/**`, `src/test_support/layout_tree.rs`, `tests/layout/browser_parity/support.rs`, focused value/cache/direct algorithm/parity tests | Replace current calc variants and calc identity/store/resolver/generation public surface with `Value(LengthPercentageOf<S>)` across `LengthOf`, `LengthAutoOf`, `DimensionOf`, track-sizing helpers, current cache context, direct algorithm consumers, and layout-owned browser-parity support while preserving non-calc keyword behavior. Direct consumers and parity parsing resolve affine values with explicit percentage-basis outcomes and no resolver parameters or no-calc sentinels. | Removing the old value surface alone fails because cache context, `traits`, `compute`, block, flex, grid, grid lanes/tracks, layout test support, and browser-parity support still import resolver/store/generation APIs. Focused block, flex, grid, grid-lane, and active calc fixture tests fail on the cycle base because calc-bearing paths panic, degrade to zero, or require resolver composition. | `CalcId`, `CalcGeneration`, `CalcResolver`, `NoCalcResolver`, `LayoutCalcStore`, `CalcExpression`, and `CalcTerm` are absent from `src`, `src/lib_tests.rs`, `src/contract_tests.rs`, and `tests`; `CacheKeyContext` no longer stores calc generation, resolver identity, or external revision state; value-family tests cover construction, conversion, basis dependence, percent fraction, and resolution outcomes; block/flex/grid/grid-lane calc paths compile without resolver traits; browser-parity calc fixture parsing uses no resolver/store/type identity and reaches comparison or fails only on later-FRI geometry findings; missing percentage basis preserves the current operation result shape (`None` for optional resolution and zero only where the existing helper was already explicitly zero-fallback); invalid numeric resolution has focused tests and is not silently treated as a valid finite result; public accessor names match the reviewed specification. | `rg -n "CalcId|CalcGeneration|CalcResolver|NoCalcResolver|LayoutCalcStore|CalcExpression|CalcTerm" src src/lib_tests.rs src/contract_tests.rs tests`; `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout value -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout cache -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout calc -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout grid_lanes -- --nocapture`; `SURGEIST_PARITY_FILTER=block/block_calc_width_margin CARGO_NET_OFFLINE=true cargo test -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored`; `SURGEIST_PARITY_FILTER=flex/flex_calc_basis_margin_gap CARGO_NET_OFFLINE=true cargo test -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored`; `SURGEIST_PARITY_FILTER=grid/grid_calc_track_and_item_margin CARGO_NET_OFFLINE=true cargo test -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored`; `cargo fmt --check` | `C01-T1` | `layout: consume affine length percentage values` |

## Completion

Cycle acceptance:

1. obsolete calc identity/resolver/store/generation representation types are
   absent from C01-owned source and tests;
2. `LengthPercentageOf<S>`, `PercentageBasisOf<S>`, and
   `NumericResolutionOf<S>` satisfy the reviewed construction and resolution
   contract for both scalar modes;
3. current length, length-auto, dimension, track value, cache-context, and direct
   algorithm helpers consume affine values without resolver-free panic paths;
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
