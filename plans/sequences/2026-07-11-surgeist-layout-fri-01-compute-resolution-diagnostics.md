# FRI-01 Compute, Resolution, And Diagnostic Sequence

Status: draft

Specification:
`plans/specs/2026-07-11-surgeist-layout-fri-01-compute-resolution-diagnostics.md`

Specification revision:
`ba5db9358f4faa5c5ad5c93d5cf5b21a066a4e942c8f4b6cf79141fa523a4a70`

Design owner: `surgeist-layout`

## Ordered Cycles

| Cycle | Owning repository | Bounded outcome | Specification sections | Prerequisites | Entry state | Exit evidence | Handoff |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `FRI-01-C01` | `surgeist-layout` | Storeless affine length-percentage values replace calc IDs, stores, generations, resolver traits, and resolver-free calc panic paths in the value model and direct value consumers. | `FRI-01.1`, `FRI-01.2`, `FRI-01.3`, `D-01`, `D-02`, `D-03`, `FRI-01.5`, `FRI-01.6`, calc portions of `FRI-01.12`, `FRI-01.16`, `FRI-01.18`, and `FRI-01.19` | Reviewed specification only. | Current source exposes `CalcId`, `CalcGeneration`, `CalcResolver`, `NoCalcResolver`, `LayoutCalcStore`, `CalcExpression`, `CalcTerm`, and calc variants on current value families. | Source/API search shows obsolete calc identity/resolver types absent; value construction/resolution tests cover both scalars, missing basis, invalid basis, overflow, and current length/auto/dimension families. | Root later lowers style calc into affine coefficients; no root action until a published layout candidate exists. |
| `FRI-01-C02` | `surgeist-layout` | Cache storage, direct leaf measurement input/output validation, and scrollbar/flex numeric properties use validated layout-owned contracts while preserving current recursive tree compute shape. | `FRI-01.3`, `D-06`, `D-07`, `D-08`, `FRI-01.8`, `FRI-01.9`, numeric/cache/measurement portions of `FRI-01.12`, `FRI-01.16`, `FRI-01.18`, and `FRI-01.19` | `FRI-01-C01` published on local and remote `main`. | Current cache stores size-only compute-size entries; direct leaf measurement can receive negative definite availability; `NodeInputOf` exposes raw scrollbar width and flex factors. | Full-output cache cold/hit equivalence passes for both scalars; direct measurement receives non-negative finite content-space constraints and rejects invalid provider output; numeric wrappers reject negative/non-finite construction through public paths. | Root later constructs numeric wrappers and adapts measurement callers. |
| `FRI-01-C03` | `surgeist-layout` | Public root compute is request/result based, recursive algorithm input is private, computation produces a completed batch or typed error, current owned panic/report/silent paths compose under the unified diagnostic envelope, and layout-owned test support compiles through the new root front door. | `FRI-01.1`, `FRI-01.2`, `D-04`, `D-05`, `D-09`, `D-10`, `FRI-01.7`, `FRI-01.10`, `FRI-01.11`, compute/error/session portions of `FRI-01.12`, `FRI-01.16`, `FRI-01.18`, and `FRI-01.19` | `FRI-01-C02` published on local and remote `main`. | Current `ComputeInputOf`, run modes, compute traits, cache hooks, root compute, direct algorithm helpers, and rounding still leave tree computation mutation and diagnostic behavior outside the unified request/result envelope; layout-owned support still calls the old root compute surface. | Public recursive algorithm input construction is absent while the C02 direct leaf helper remains public; root and flex-item-under-viewport requests validate availability; failure tests prove no completed batch or partial layout/cache state; typed diagnostic tests cover owned invalid, missing context, provider, unsupported, and invariant classes; layout-owned support reaches the new root request/batch surface. | Root later translates or applies completed batches through root-owned retained/facade state. |
| `FRI-01-C04` | `surgeist-layout` | Browser-parity fixture verification, docs, public reexport cleanup, Rust 1.97 MSRV, and final initiative verification align with the new FRI-01 contracts without duplicating root adapters or retaining compatibility aliases. | `FRI-01.12`, `FRI-01.13`, `FRI-01.14`, `FRI-01.15`, `FRI-01.16`, `FRI-01.17`, `FRI-01.18`, and `FRI-01.19` | `FRI-01-C03` published on local and remote `main`. | Browser parity fixtures still need final C04 verification through public layout requests; current docs mention resolver-aware calc values; leaf manifest lacks `rust-version`. | Calc fixture families reach comparison through public layout requests; layout docs and reexports match implemented contracts; `rust-version = "1.97"` is present and the already-installed Rust 1.97 compiler passes the configured gates; final commands, generator-feature test, unsafe scan, task reviews, and holistic review are clean. | Complete crate candidate report to root with published SHA and root integration requirements for style calc lowering, numeric wrappers, root request construction, batch application, and root API artifact refresh. |

## Dependency Notes

- `FRI-01-C01` precedes every other cycle because calc identity removal changes
  value APIs consumed by cache keys, measurement paths, browser parity support,
  and root handoff.
- `FRI-01-C02` precedes the compute-session cycle so the session can stage
  already-correct cache, measurement, and numeric-property state instead of
  preserving invalid raw inputs.
- `FRI-01-C03` includes the minimum layout-owned support migration required to
  compile and exercise the new root request/batch front door. `FRI-01-C04`
  remains the browser-parity fixture verification and documentation closure
  cycle.
- `FRI-01-C04` is the initiative closure cycle. It verifies integration-facing
  layout-owned artifacts and produces the root handoff after all source
  contracts are already implemented.
