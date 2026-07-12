# FRI-01-C03 Root Compute Request And Diagnostics

Status: reviewed

Cycle ID: `FRI-01-C03`

Owning repository: `surgeist-layout`

Cycle base: `1568bdcf9e0509788a5413024f148a832b562b7d`

Reviewed specification:
`plans/specs/2026-07-11-surgeist-layout-fri-01-compute-resolution-diagnostics.md`
at `903ec9d170ea22aa1c2e6626bb0858d8ab79ed3c730ae68b333eb5ba3e84fc5b`
sections `FRI-01.1`, `FRI-01.2`, `D-04`, `D-05`, `D-09`, `D-10`,
`FRI-01.7`, `FRI-01.10`, `FRI-01.11`, compute/error/session portions of
`FRI-01.12`, `FRI-01.16`, `FRI-01.18`, and `FRI-01.19`.

Reviewed sequence:
`plans/sequences/2026-07-11-surgeist-layout-fri-01-compute-resolution-diagnostics.md`
at `fbaa3664ba91c56fdccd7c927c66ce56aa4a29f2463fdabe3ee4df70d574264f`,
entry `FRI-01-C03`.

Bounded outcome: tree-backed root computation starts from a validated public
request, recursive algorithm input and mutation hooks are private fallible
session details, and a root pass returns either one completed layout batch or
one site-aware typed error without exposing partial output or cache state.

## Boundary

This cycle owns `src/output.rs`, `src/compute.rs`, `src/traits.rs`, `src/lib.rs`,
block/flex/grid/hidden callers needed to thread the fallible private algorithm
contract, crate-local test support, and browser-parity support needed to call
the new root front door.

It does not change calc value semantics, C02 leaf measurement validation,
property-specific sizing families, later-FRI geometry behavior, README/MSRV,
generated XML, root adapters, root API artifacts, or sibling repositories.
C04 owns final README/MSRV/browser-parity documentation cleanup after the C03
front door exists.

Current evidence: `ComputeInputOf`, `RunMode`, `SizingMode`, `RequestedAxis`,
`Compute`, `Round`, `CacheAccess`, `compute_leaf`, `compute_root`, `compute_hidden`,
`round_layout`, `compute_block`, `compute_flex`, `compute_grid`, and
`compute_cached` are public or reexported; current tree computation mutates
unrounded output, final output, and cache state during traversal; root scroll
finalization uses `expect`; length resolution helpers silently zero missing or
invalid context; browser parity manually constructs root and flex-item root
algorithm inputs.

## Impacts

Public API: breaking pre-release change. New public surface includes
`LayoutRootRequestOf<S>`, `LayoutRootContextOf<S>`,
`FlexItemRootContextOf<S>`, `LayoutRoundingMode`,
`CompletedLayoutBatchOf<Node, S>`, `LayoutResultOf<Node, T, S, M>`,
`LayoutErrorOf<Node, S, M>`, `LayoutErrorSiteOf<Node>`, `LayoutOperation`,
`LayoutErrorKindOf<S, M>`, and the read/provider `LayoutTree` front door.
Recursive algorithm inputs, run modes, mutation traits, cache hooks, direct
algorithm entry points including the direct leaf helper, and root rounding
helpers stop being public surface. C02 measurement input and local measurement
error types remain available to the public `LayoutTree` provider contract.

Dependencies/features/artifacts/docs/MSRV: no new dependency, feature,
generated artifact, README, or MSRV change in this cycle. Rustdoc may be added
only where required for the new public types.

Root follow-up: root later constructs layout root requests, handles layout
errors, applies or translates completed batches, and refreshes root-owned API
artifacts after this candidate is published.

Unsafe: no Surgeist-owned unsafe may be added or retained.

## Tasks

| Task | Files/area | Intended behavior/outcome | RED evidence | Acceptance criteria | Commands | Depends on | Intended commit |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `C03-T1` | `src/output.rs`, `src/compute.rs`, `src/lib.rs`, `src/compute_tests.rs`, `src/root_tests.rs` | Add the public request, root-context, rounding-policy, completed-batch, and typed error model with private fields and validated constructors. | Focused tests fail because the C03 public types do not exist and root availability can only be supplied through raw algorithm input. | Root requests reject negative, NaN, and infinite definite availability; viewport and flex-item-under-viewport contexts are distinct validated states; `NearestCssPixel` names current rounding; completed batches expose read-only unrounded/final/cache entries; error accessors preserve site, operation, kind, scalar detail, and provider source without strings as control flow. | `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout root -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout compute -- --nocapture`; `cargo fmt --check` | Published `FRI-01-C02` | `compute: add root request and error model` |
| `C03-T2` | `src/output.rs`, `src/traits.rs`, `src/compute.rs`, block/flex/grid/hidden call sites, crate-local unit tests | Move recursive algorithm input and mutation hooks behind crate-private fallible contracts and thread `LayoutResultOf` through direct algorithm execution. | Public-surface absence gates fail because recursive input/run modes, mutation traits, and direct helpers are reexported; focused invalid-resolution tests fail because current helpers silently zero missing or invalid required context. | `ComputeInputOf`, run/sizing modes, requested axis, `Compute`, `Round`, `CacheAccess`, `compute_cached`, `compute_leaf`, and direct algorithm entry points are not public or reexported; algorithms receive only crate-private session-created input; child, hidden, intrinsic, and leaf paths return `LayoutResultOf`; required missing basis and invalid numeric resolution map to typed errors instead of panic or silent zero where the operation has no valid indefinite behavior. | `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout value -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout block -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout flex -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout grid -- --nocapture`; `bash -lc 'if rg -n -e "pub use .*\\b(ComputeInput|ComputeInputOf|RunMode|SizingMode|RequestedAxis|Compute|Round|CacheAccess|compute_cached|compute_leaf|compute_root|compute_hidden|round_layout|compute_block|compute_flex|compute_grid)\\b" src/lib.rs || rg -n -e "pub (struct|enum|trait|fn) (ComputeInputOf|RunMode|SizingMode|RequestedAxis|Compute|Round|CacheAccess|compute_cached|compute_leaf|compute_root|compute_hidden|round_layout|compute_block|compute_flex|compute_grid)\\b" src; then exit 1; else rc=$?; test "$rc" -eq 1; fi'`; `cargo fmt --check` | `C03-T1` | `compute: privatize fallible algorithm input` |
| `C03-T3` | `src/compute.rs`, `src/traits.rs`, `src/cache.rs`, `src/test_support/layout_tree.rs`, root/cache tests | Implement `compute_layout` over a read/provider `LayoutTree` contract and stage unrounded output, final output, cache stores, and cache clears into `CompletedLayoutBatchOf`. | Focused atomicity tests fail because current public root computation mutates tree output and cache state before success is known and has no completed batch. | Successful viewport and flex-item-under-viewport requests return one batch with expected unrounded output, final output under requested rounding policy, cache stores, and cache clears; provider errors and invalid provider output return no batch and no public partial state; cache hits and stores remain semantically equivalent to C02; crate test support observes batch contents without reintroducing public mutation traits. | `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout root -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout cache -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout leaf -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout compute -- --nocapture`; `cargo fmt --check` | `C03-T2` | `compute: return completed layout batches` |
| `C03-T4` | `src/compute.rs`, `src/scroll.rs`, block/flex/grid/leaf error call paths, diagnostics tests | Convert C03-owned root finalization, measurement, cache, unsupported, and internal-invariant failures into the unified error envelope. | Focused diagnostics tests fail because root scroll finalization still uses `expect`, leaf provider errors are leaf-local outside tree compute, and unsupported or invariant failures do not share one typed site/operation envelope. | Root scroll geometry construction and rounding failures return `InternalInvariant`; tree leaf provider errors return `Measurement`; invalid provider output returns `InvalidInput`; unsupported well-formed C03-represented requests return `UnsupportedCapability`; deep child errors propagate unchanged; tests assert exact site, operation, kind, and source for each owned class. | `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout diagnostics -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout root -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout scroll -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout block -- --nocapture`; `cargo fmt --check` | `C03-T3` | `compute: classify root diagnostics` |
| `C03-T5` | `src/lib.rs`, `src/test_support/layout_tree.rs`, `tests/layout/browser_parity/support.rs`, public-surface and parity tests | Migrate crate support and browser parity to `compute_layout` and verify the public front door is the only external tree-backed compute path. | Browser parity support fails on the cycle base when direct public root/flex-item compute entry points are removed. | Browser parity calls `compute_layout` for viewport and flex-item roots and reads/applies the returned batch locally; no test-local duplicate lowering or old public mutation trait remains; calc fixture families still reach comparison; public absence gates prove obsolete C03 surfaces are not reexported. | `SURGEIST_PARITY_FILTER=block/block_calc_width_margin CARGO_NET_OFFLINE=true cargo test -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored`; `SURGEIST_PARITY_FILTER=flex/flex_calc_basis_margin_gap CARGO_NET_OFFLINE=true cargo test -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored`; `SURGEIST_PARITY_FILTER=grid/grid_calc_track_and_item_margin CARGO_NET_OFFLINE=true cargo test -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored`; `bash -lc 'if rg -n -e "layout::(ComputeInput|RunMode|SizingMode|RequestedAxis|Compute|Round|CacheAccess|compute_leaf|compute_root|round_layout)|surgeist_layout::(ComputeInput|RunMode|SizingMode|RequestedAxis|Compute|Round|CacheAccess|compute_leaf|compute_root|round_layout)" tests; then exit 1; else rc=$?; test "$rc" -eq 1; fi'`; `cargo fmt --check` | `C03-T4` | `layout: use root compute request front door` |

## Completion

Cycle acceptance:

1. public tree-backed layout computation starts only from
   `LayoutRootRequestOf<S>` and `compute_layout`;
2. recursive run modes, algorithm inputs, direct algorithm entry points,
   mutation traits, and cache mutation hooks are not public surface;
3. a successful request returns a complete read-only batch containing all
   unrounded, final, cache-store, and cache-clear effects;
4. any C03-owned invalid, missing-context, provider, unsupported, or invariant
   failure returns one `LayoutErrorOf` without a completed batch or partial
   public state;
5. browser parity and crate-local tests use the new request/batch front door;
   and
6. root handoff names the new request, error, and batch contracts for adapter and
   facade integration.

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
bash -lc 'files=$(git ls-files "*.rs"; git ls-files --others --exclude-standard "*.rs"); test -n "$files"; printf "%s\n" "$files" | xargs rg -n --pcre2 '\''#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{'\''; rc=$?; test "$rc" -eq 1'
```

Required handoff: after publication, report the published SHA and root-facing
`compute_layout`, `LayoutRootRequestOf`, `LayoutErrorOf`, and
`CompletedLayoutBatchOf` contracts. No sibling or root edits occur in this
cycle.

Genuine blockers: if converting a specific algorithm path to fallible output
requires a later-FRI product decision rather than a C03 diagnostic wrapper,
return the exact call path, triggering input, observed behavior, and owning FRI
before narrowing the cycle.
