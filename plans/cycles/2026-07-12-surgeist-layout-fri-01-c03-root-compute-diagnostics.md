# FRI-01-C03 Root Compute Request And Diagnostics

Status: complete

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
at `ba49b2700cffa8a2cdc411e21e12a7b443ba1fd7a5914ab441bd3fcd171047af`,
entry `FRI-01-C03`.

Bounded outcome: tree-backed root computation starts from a validated public
request, recursive algorithm input and mutation hooks are private fallible
session details, and a root pass returns either one completed layout batch or
one site-aware typed error without exposing partial output or cache state.

## Boundary

This cycle owns `src/output.rs`, `src/compute.rs`, `src/traits.rs`, `src/lib.rs`,
block/flex/grid/hidden callers needed to thread the fallible private algorithm
contract, crate-local support, and layout-owned integration test support needed
to compile through the new root front door.

It does not change calc value semantics, C02 leaf measurement validation,
property-specific sizing families, later-FRI geometry behavior, README/MSRV,
generated XML, root adapters, root API artifacts, or sibling repositories.
C04 owns final README/MSRV/browser-parity documentation cleanup after the C03
front door exists.

Current evidence: `ComputeInputOf`, `RunMode`, `SizingMode`, `RequestedAxis`,
`Compute`, `Round`, `CacheAccess`, `compute_root`, `compute_hidden`,
`round_layout`, `compute_block`, `compute_flex`, `compute_grid`, and
`compute_cached` are public or reexported as recursive tree-compute surface;
the C02 `compute_leaf` helper remains public but its input is still publicly
field-constructible with recursive run-mode state; current tree computation
mutates unrounded output, final output, and cache state during traversal; root
scroll finalization uses `expect`; length resolution helpers silently zero
missing or invalid context; browser parity manually constructs root and
flex-item root algorithm inputs.

Revision evidence after `C03-T1`: an attempted standalone fallible
`Compute::compute_child` conversion showed that child result propagation fans
through grid child layout, grid lanes, grid tracks, block inline segmentation,
flex intrinsic sizing, and direct-helper tests before a completed session/batch
front door exists. This plan therefore treats private fallible algorithm
execution and completed-batch staging as one implementation boundary.

Revision evidence after a blocked `C03-T2` attempt: hiding the old recursive
public surface causes `tests/layout/browser_parity/support.rs` to fail
compilation before `C03-T4` migrates it. `C03-T2` and `C03-T3` therefore run
focused library tests with `--lib`; `C03-T4` remains the first integration-test
support compile gate.

## Impacts

Public API: breaking pre-release change. New public surface includes
`LayoutRootRequestOf<S>`, `LayoutRootContextOf<S>`,
`FlexItemRootContextOf<S>`, `LayoutRoundingMode`,
`CompletedLayoutBatchOf<Node, S>`, `LayoutResultOf<Node, T, S, M>`,
`LayoutErrorOf<Node, S, M>`, `LayoutErrorSiteOf<Node>`, `LayoutOperation`,
`LayoutErrorKindOf<S, M>`, and the read/provider `LayoutTree` front door.
Recursive algorithm input construction, run modes, mutation traits, cache hooks,
direct recursive algorithm entry points, and root rounding helpers stop being
public surface. The C02 direct `compute_leaf` helper remains public with a
validated leaf-measurement contract; `ComputeInputOf<S>` may remain public only
as an opaque leaf-helper input with validated leaf-only constructors and no
public recursive fields or modes. C02 measurement input and local measurement
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
| `C03-T2` | `src/output.rs`, `src/traits.rs`, `src/compute.rs`, `src/cache.rs`, block/flex/grid/hidden call sites, `src/test_support/layout_tree.rs`, root/cache/leaf/compute/value/block/flex/grid tests | Implement the private fallible compute session and completed-batch boundary together. Recursive algorithm input and mutation hooks become crate-private session details; child, hidden, intrinsic, tree-invoked leaf, cache, rounding, and direct recursive algorithm paths return `LayoutResultOf`; successful root requests return `CompletedLayoutBatchOf`; the public direct `compute_leaf` helper remains leaf-local. | Public-surface absence gates fail because recursive input/run modes, mutation traits, and direct recursive helpers are reexported; focused atomicity tests fail because current root computation mutates tree output and cache state before success; focused invalid-resolution tests fail because current helpers silently zero missing or invalid required context. | Public `ComputeInputOf` no longer exposes recursive fields or modes and is constructible only through validated leaf-helper entry points if retained for `compute_leaf`; `RunMode`, `SizingMode`, `RequestedAxis`, `Compute`, `Round`, `CacheAccess`, `compute_cached`, and direct recursive algorithm entry points are not public or reexported; algorithms receive only private session-created input; tree-invoked leaf errors are wrapped into `LayoutResultOf`; successful viewport and flex-item-under-viewport requests return one batch with expected unrounded output, final output under requested rounding policy, cache stores, and cache clears; provider errors and invalid provider output return no batch and no public partial state; cache hits and stores remain semantically equivalent to C02; required missing basis and invalid numeric resolution map to typed errors where the operation has no valid indefinite behavior. | `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout --lib value -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout --lib block -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout --lib flex -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout --lib grid -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout --lib root -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout --lib cache -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout --lib leaf -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout --lib compute -- --nocapture`; `bash -lc 'if rg -n -e "pub use .*\\b(RunMode|SizingMode|RequestedAxis|Compute|Round|CacheAccess|compute_cached|compute_root|compute_hidden|round_layout|compute_block|compute_flex|compute_grid)\\b" src/lib.rs || rg -n -e "pub (enum|trait|fn) (RunMode|SizingMode|RequestedAxis|Compute|Round|CacheAccess|compute_cached|compute_root|compute_hidden|round_layout|compute_block|compute_flex|compute_grid)\\b" src || rg -n -e "pub (run_mode|sizing_mode|axis|known|parent|available):" src/output.rs; then exit 1; else rc=$?; test "$rc" -eq 1; fi'`; `cargo fmt --check` | `C03-T1` | `compute: return fallible completed batches` |
| `C03-T3` | `src/compute.rs`, `src/scroll.rs`, block/flex/grid/leaf error call paths, diagnostics tests | Convert remaining C03-owned root finalization, measurement, cache, unsupported, and internal-invariant failures into the unified error envelope. | Focused diagnostics tests fail because root scroll finalization still uses `expect`, leaf provider errors are leaf-local outside tree compute, or unsupported/invariant failures do not share one typed site/operation envelope. | Root scroll geometry construction and rounding failures return `InternalInvariant`; tree leaf provider errors return `Measurement`; invalid provider output returns `InvalidInput`; unsupported well-formed C03-represented requests return `UnsupportedCapability`; deep child errors propagate unchanged; tests assert exact site, operation, kind, and source for each owned class. | `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout --lib diagnostics -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout --lib root -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout --lib scroll -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout --lib block -- --nocapture`; `cargo fmt --check` | `C03-T2` | `compute: classify root diagnostics` |
| `C03-T4` | `src/lib.rs`, `src/test_support/layout_tree.rs`, `tests/layout/browser_parity/support.rs`, public-surface and integration-test support | Migrate crate support and layout-owned integration test support to `compute_layout` enough that the public front door is the only external tree-backed compute path. Full ignored browser-parity fixture execution remains `FRI-01-C04`. | Integration test support fails to compile on the cycle base when direct public root/flex-item compute entry points are removed. | Layout-owned support calls `compute_layout` for viewport and flex-item roots and reads/applies the returned batch locally; no test-local duplicate lowering or old public mutation trait remains; normal integration test support compiles without old public tree-compute surface; public absence gates prove obsolete C03 surfaces are not reexported. | `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout --test layout --no-run`; `bash -lc 'if rg -n -e "layout::(RunMode|SizingMode|RequestedAxis|Compute|Round|CacheAccess|compute_root|compute_hidden|round_layout|compute_block|compute_flex|compute_grid)|surgeist_layout::(RunMode|SizingMode|RequestedAxis|Compute|Round|CacheAccess|compute_root|compute_hidden|round_layout|compute_block|compute_flex|compute_grid)" tests; then exit 1; else rc=$?; test "$rc" -eq 1; fi'`; `cargo fmt --check` | `C03-T3` | `layout: use root compute request front door` |

## Completion

Cycle acceptance:

1. public tree-backed layout computation starts only from
   `LayoutRootRequestOf<S>` and `compute_layout`;
2. recursive run modes, arbitrary recursive algorithm inputs, direct recursive
   algorithm entry points, mutation traits, and cache mutation hooks are not
   public surface, while the C02 direct leaf helper remains public;
3. a successful request returns a complete read-only batch containing all
   unrounded, final, cache-store, and cache-clear effects;
4. any C03-owned invalid, missing-context, provider, unsupported, or invariant
   failure returns one `LayoutErrorOf` without a completed batch or partial
   public state;
5. layout-owned test support compiles and uses the new request/batch front door;
   full ignored browser-parity fixture verification remains C04; and
6. root handoff names the new request, error, and batch contracts for adapter and
   facade integration.

Final command list:

```sh
CARGO_NET_OFFLINE=true cargo test -p surgeist-layout
CARGO_NET_OFFLINE=true cargo test -p surgeist-layout --features layout-golden-generate
CARGO_NET_OFFLINE=true cargo test -p surgeist-layout --test layout --no-run
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
