# P01-I08-S01-C07 Validated Whole-Crate Sprawl Containment And Final Candidate

Status: planned

Cycle ID: `P01/I08/S01/C07`

Owning repository: `surgeist-layout`

Cycle base: `dc71a5582ab0ef3925826dce09b93ee9fa6f49a1`

Reviewed specification:
`plans/specs/P01-I08-grid-subgrid-and-grid-lanes-completeness.md`, normalized
semantic-content SHA-256
`a636dd9c9b896e2986fd13ab303f8506fba7eec6b0ba909e542eee9dc39770e6`,
commit `09bab4edc2bbff4aad42469937a328d0724989c0`: the finite structural
invariants in `FRI-08.14`; verification, responsibility, finding closure,
handoff, and acceptance in `FRI-08.15` through `FRI-08.19`.

Reviewed implementation sequence:
`plans/sequences/P01-I08-S01-grid-subgrid-and-grid-lanes-completeness.md`,
normalized semantic-content SHA-256
`b8efd9c6ac235fa167fa06d80c8155b2d494187dac46c06f28985de77b02cbe9`,
commit `8778ae9487bc1880786a9710b00b7865755d8462`, entry
`P01/I08/S01/C07`.

Bounded outcome: disposition the complete accepted whole-crate sprawl report,
implement every authorized confirmed mechanical opportunity and both required
lint-policy corrections without changing behavior, retain the explicit
generator-architecture exclusion, and publish the final remotely verified
FRI-08 leaf candidate.

## 1 Boundary

The remotely verified C02R candidate at the cycle base is immutable. Local
`main`, its tracking ref, and observed remote `main` equal the base and the
worktree is clean. C01 through C06, C06R, and C02R close the eight FRI-08
behavior findings and carry 1,448 HTML sources, 5,776 comment-free XML outputs,
the exact 18-source/72-row FRI-08 browser surface, and the sole schema-3
`all.json` with 5,776 generated, 16 unsupported, three unchanged FRI-07
expected-fail, zero quarantined, and zero failed-to-generate rows.

The final initiative sprawl review was a read-only whole-crate review at the
cycle base, not a commit-range holistic review. It classified all 7,388 tracked
files and every one of the 1,515 behavior-bearing targets: 53 production Rust
files, six external test/generator Rust files, 1,448 authored HTML inputs, three
shell scripts, one JavaScript helper, one CSS baseline, two TOML manifests, and
the `Justfile`. Generated XML/report output, dependencies, build caches, and
planning documents were classified but excluded as refactor targets. Both
offline warning-denied Clippy feature rows passed; Advisory policy is absent;
owned Rust contained zero unsafe constructs. The review returned
`REPORT_WITH_ESCALATIONS` with eleven mechanical opportunities and two
lint-policy escalations.

The earlier ordinary inherited-placement estimator and alternate grid-lanes
final sizing findings are `resolved`. Current ordinary placement enters
`derive_grid_placement_demand`; the estimator helpers are absent. Ordinary and
grid-lanes final sizing share one policy-free phase owner; lanes-only final
sizing helpers and the collection-wide fit-content shortcut are absent. The
earlier initiative-range “sprawl” verdicts are superseded and supply no closure
evidence.

The accepted finite disposition set is:

| ID | Category and source | C07 disposition |
| --- | --- | --- |
| `SP-002` | Repeated visible inline-run transitions in `src/block.rs` around the text, boundary, line-break, and inline-box start branches. | Confirm and consolidate behind one private transition owner after role-specific validation. |
| `SP-003` | Repeated canonical scroll-source assembly and retained-child reconstruction in `src/block.rs`, `src/flex.rs`, and `src/compute.rs`; natural lower owner `src/scroll.rs`. | Confirm and introduce one policy-named crate-private builder while retaining algorithm-specific contributions and error mapping. |
| `SP-004` | Duplicate policy-free optional-size/minimum arithmetic in block, flex, and compute; natural owner `src/layout_math.rs`. | Confirm and move the two exact sealed operations to layout math. |
| `SP-005` | Duplicate root and contextual grid scrollbar-settlement loops in `src/grid/mod.rs`. | Confirm and consolidate through one settlement-loop helper while retaining caller-specific initial state and result projection. |
| `SP-006` | Duplicate recursive fixture traversal inside `tests/bin/surgeist-layout-generate/generator.rs`. | Confirmed but not authorized for implementation: the immutable initiative envelope permits generator changes only for parser updates, new fixtures, or confirmed genuine bugs. Preserve as a precise deferred generator-owned handoff; do not edit generator production. |
| `SP-007` | Duplicate ordinary and `calc-size` recursive expression structure in `tests/layout/browser_parity/support.rs`. | Confirm and parse one private structural operation representation with destination-specific lowering. This is an explicitly permitted parser update. |
| `SP-008` | Duplicate retained-state sinks and atomic batch plumbing in `src/grid_tests.rs`. | Confirm and consolidate test-only retained state and batch preparation. |
| `SP-009` | Independently reconstructed preorder node identities and duplicate recursive assertion walkers in `src/test_support/grid_layout_comparison.rs`. | Confirm and retain one construction-time identity map or one phase-parameterized walker. |
| `SP-010` | Duplicate block/flex/grid browser-family discovery, validation, and runners beside existing generic grid-lanes/subgrid helpers in `tests/layout/browser_parity.rs`. | Confirm and route all five families through the existing generic harness without changing fixtures or expected geometry. |
| `SP-011` | Exact duplicate scroll-geometry helpers and scroll-padding cases in block, flex, and leaf tests. | Confirm and move immutable generic fixture construction/invariant assertions to shared test support while keeping algorithm assertions local. |
| `SP-012` | `establishes_independent_formatting_context` always returns false, making the matching production subgrid ineligibility variant unreachable. | Confirm and remove the unreachable predicate, branch, and production-only variant; retain oracle capability. |
| `ESC-001` | `VerticalWritingModeUnsupported` is impossible, yet its `dead_code` expectation and error plumbing remain across grid axis/subgrid callers. | Correct by removing the impossible error/result plumbing and unsupported expectation while preserving every axis formula and eligibility result. |
| `ESC-002` | Fourteen bare source suppressions exist without adopted central policy: three module-level `dead_code`, ten oracle facade `unused_imports`, and one grid-test `clippy::too_many_arguments`. | Correct by removing unused facade exports/dead scaffolding and replacing the oversized test helper with a typed input; do not add allowances or policy. |

Every source change is behavior-preserving. If characterization exposes a
behavior, correctness, public-contract, artifact, or input defect, stop that
task and return it to the coordinator; do not hide it as mechanical cleanup.

Out of scope: new product behavior, public API or compatibility changes,
authored CSS semantics, root adapters or artifacts, dependency/feature/lockfile/
MSRV changes, browser execution, generation, HTML/XML/report/manifest edits,
new fixtures, generator architecture or traversal changes, later FRI-09/F10
behavior, new lint policy, unsafe, and unrelated cleanup.

## 2 Impacts

Public API and observable behavior: unchanged. All new helpers are crate-private
or test-private. Existing error identities remain unless the state is proven
unconstructable by `ESC-001`; removing that impossible internal-only error path
must not change any constructable result.

Dependencies, features, lockfile, MSRV, documentation, examples, root
integration, finding ownership, authored fixtures, and generated artifacts:
unchanged. No browser or generation command is authorized. `SP-006` records the
only generator opportunity and deliberately makes no generator source change.

Frozen artifact/input SHA-256 values are:

- `corpus.toml`:
  `c6e6f1422e14a5e4aa474c143998063ce0de4d0a9123b69875b35a4ed009a8f6`;
- helper:
  `c684c7f167d95997a4a9f0250467bbaf72c1b73e69e0f707a2ef32f4d25f7f36`;
- sole `all.json`:
  `c10dc550d260a239c8bf9dd553f5272ca3bcc2826099bc182f800986b8b94c0e`;
- complete XML inventory:
  `a98d1ccceaeeb336ee3cb3c0151607589c0a4ae0376a46c560ba4341f95ad6ae`;
- complete XML hash lineage:
  `bad8e418caee72cc62a123dc93efe89fdb07bfb5dee4345f3df7d8fd6fe44fdf`.

Owned Rust remains free of unsafe. C07 adds no `allow` or `expect`; every
affected existing marker is either retained with current evidence or removed.

## 3 Tasks

### 3.1 `P01/I08/S01/C07/T01` Unify Visible Inline-Run State Transitions

**Files/area:** `src/block.rs` and focused `src/block_tests.rs` characterization.

**Outcome:** retain role-specific scan and validation, then route the four
visible inline-run starts through one private transition that owns child layout,
collapsed-margin advance, content/scroll maxima, baselines, cursor/static
position, float transfer, and collapse-state reset.

**Evidence and acceptance:** first characterize text, explicit line break,
inline boundary, and inline-box paths in both scalars, including hidden break,
source indices, margins, floats, baselines, scroll, and invalid-input ordering.
The characterization passes before and after the refactor. No branch loses a
role-specific precondition and no new production surface is introduced.

**Intended commit:** `refactor(block): unify inline run transitions`.

### 3.2 `P01/I08/S01/C07/T02` Centralize Canonical Scroll Source Assembly

**Files/area:** `src/scroll.rs`, `src/block.rs`, `src/flex.rs`, `src/compute.rs`,
and focused block/flex/root tests.

**Outcome:** add one crate-private retained/source builder parameterized by a
named range-seed policy. Block, flex, and root retain contribution construction,
block-only reserved-gutter exclusion, and caller-local error mapping.

**Evidence and acceptance:** characterize existing-geometry fast paths, flow
axes, settled scrollbars, clips, padding, targets, snapping, origins, child
reconstruction, and both scalars. Exact public node output and failure identity
remain unchanged. No policy disappears into booleans or optional sentinel data.

**Intended commit:** `refactor(scroll): centralize geometry sources`.

### 3.3 `P01/I08/S01/C07/T03` Consolidate Optional Layout Arithmetic

**Files/area:** `src/layout_math.rs`, `src/block.rs`, `src/flex.rs`,
`src/compute.rs`, and directly affected tests.

**Outcome:** move the exact componentwise `Size<Option<S>>` and optional-minimum
operations to sealed layout-math extensions and delete local duplicate traits.

**Evidence and acceptance:** direct f32/f64 characterization proves `None`,
componentwise maximum, absent-minimum, and finite/non-finite error behavior.
Call sites retain identical results and evaluation order.

**Intended commit:** `refactor(math): centralize optional size floors`.

### 3.4 `P01/I08/S01/C07/T04` Unify Grid Scrollbar Settlement

**Files/area:** `src/grid/mod.rs` and focused `src/grid_tests.rs`.

**Outcome:** replace the root and contextual/subgrid settlement loops with one
private helper accepting explicit initial state, parent context, and measurement
boundary, returning `GridComputeResult`. Root result projection remains outside.

**Evidence and acceptance:** characterize caller-settled root state versus
contextual `INITIAL`, geometry/no-geometry termination, error mapping, cache,
both scalars, inherited contexts, and scrollbar convergence. Exact iteration and
publication semantics remain unchanged.

**Intended commit:** `refactor(grid): unify scrollbar settlement`.

### 3.5 `P01/I08/S01/C07/T05` Unify Browser Expression And Family Harnesses

**Files/area:** `tests/layout/browser_parity/support.rs` and
`tests/layout/browser_parity.rs`. Generator production and all fixtures are
frozen.

**Outcome:** parse ordinary and `calc-size` recursive min/max/clamp structure
through one private structural representation with destination-specific leaves;
route block, flex, grid, grid-lanes, and subgrid fixture families through one
generic discovery/validation/execution harness.

**Evidence and acceptance:** preserve keyword domains, recursion depth 64,
affine and omitted-endpoint rules, arity and exact diagnostics, output types,
canonical paths, duplicate/missing/misplaced rejection, family topology checks,
and exact comparison. All 1,448 HTML inputs and 5,776 XML outputs remain
byte-frozen; no generator/browser/generation command runs.

**Intended commit:** `refactor(parity): unify parser and family harnesses`.

### 3.6 `P01/I08/S01/C07/T06` Consolidate Atomic Grid Test Retention

**Files/area:** `src/grid_tests.rs` only.

**Outcome:** replace three duplicate retained-state stores and batch preparation
paths with one private generic retained store and shared preparation helper;
local trees retain measurement behavior and trait ownership.

**Evidence and acceptance:** preserve invalidation-first order, source
association, cache context, failure non-publication, commit replacement, and
cold/warm reuse in both scalars. This is test support only and must not weaken
assertions or expose production surface.

**Intended commit:** `refactor(grid-tests): share retained batch state`.

### 3.7 `P01/I08/S01/C07/T07` Retain One Grid Comparison Identity Walk

**Files/area:** `src/test_support/grid_layout_comparison.rs` and focused grid
comparison consumers in `src/grid_tests.rs` only when characterization requires.

**Outcome:** retain construction-time node identity once or use one
phase-parameterized recursive walker for unrounded and final expectations.

**Evidence and acceptance:** preserve preorder IDs, measurement attachment,
optional expectations, phase separation, tolerance, and diagnostics for nested
trees. The helper remains test-only and every existing comparison assertion is
equally strong.

**Intended commit:** `refactor(test-support): unify grid comparison walk`.

### 3.8 `P01/I08/S01/C07/T08` Share Scroll Geometry Test Fixtures

**Files/area:** existing shared test support plus `src/block_tests.rs`,
`src/flex_tests.rs`, and `src/leaf_tests.rs`.

**Outcome:** move exact immutable scroll-geometry input construction and
invariant assertions to shared generic test support while retaining each
algorithm's runner and behavior assertions locally.

**Evidence and acceptance:** preserve f32/f64 selection, every `ComputeInputOf`
field, site/operation/invariant identity, Auto/value edge cases, and exact
scroll-padding cases. No production code or behavior changes.

**Intended commit:** `refactor(test-support): share scroll fixtures`.

### 3.9 `P01/I08/S01/C07/T09` Remove Impossible Grid Scaffolding

**Files/area:** `src/grid/axis.rs`, `src/grid/subgrid.rs`, and only the direct
grid callers/tests needed to remove impossible result plumbing; oracle capability
remains frozen.

**Outcome:** make `map_grid_axis` infallible, remove
`VerticalWritingModeUnsupported`, its unsupported `dead_code` expectation, and
all now-impossible propagation. Remove the always-false production independent-
formatting-context predicate, branch, and production-only ineligibility variant.

**Evidence and acceptance:** first characterize every horizontal, vertical, and
sideways axis formula, both scalars, all current subgrid eligibility results, and
error ordering around independent checks. Structural evidence proves the removed
states are unconstructable. No oracle capability, public error, constructable
result, or production eligibility changes.

**Intended commit:** `refactor(grid): remove impossible axis states`.

### 3.10 `P01/I08/S01/C07/T10` Remove Unowned Source Suppressions

**Files/area:** `src/test_support/mod.rs`,
`src/test_support/oracle/grid/mod.rs`, `src/grid_tests.rs`, and the smallest
direct test-support consumers needed to eliminate the fourteen reported bare
allows. Stop before production behavior.

**Outcome:** delete unused oracle facade reexports and dead test scaffolding;
replace the oversized test helper argument list with one typed test input. Do
not add, move, narrow, or convert an `allow`; do not invent central policy.

**Evidence and acceptance:** the exact fourteen bare allows are absent, no new
allow/expect exists, the adopted Required matrix remains warning-free, and test
coverage/diagnostics are not weakened. Any still-needed facade capability must
be consumed directly rather than hidden by suppression.

**Intended commit:** `refactor(test-support): remove source suppressions`.

### 3.11 `P01/I08/S01/C07/T11` Final Disposition And Candidate Evidence

**Files/area:** no production implementation. Coordinator-owned cycle status and
final evidence only after T01 through T10 are reviewed and integrated.

**Outcome:** prove every whole-crate report row has exactly one final disposition:
implemented and characterized for `SP-002` through `SP-005` and `SP-007` through
`SP-012`; corrected for `ESC-001` and `ESC-002`; explicitly deferred by the
immutable generator boundary for `SP-006`; and resolved for the two prior
behavior findings. No unrecorded sprawl item remains.

**Acceptance:** full verification, task verdicts, one fresh holistic C07 review,
artifact/input hashes, exact scope, suppression/unsafe evidence, clean worktree,
publication, and remote readback all pass.

## 4 Verification

Each behavior-sensitive task proves pre-change characterization before source
editing and repeats it afterward. Each task runs its focused selector, directly
affected module controls, default package tests, strict warning-denied Clippy,
format, diff, exact scope, no-new-suppression, and unsafe evidence. Browser
parser/harness work additionally runs parser and layout parity tests without
browser or generation.

After every task review is clean and the cycle is integrated, run at minimum:

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri08_c07_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri08_c02r_lanes_track_phase_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri08_c06r_inherited_placement_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --test layout fri08_c0
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --features layout-golden-generate --all-targets -- -F unsafe-code -D warnings
cargo fmt --check
git diff --check
```

Also prove the exact reviewed task scopes, frozen browser tree and artifact/input
hashes, all 5,776 XML files comment-free, the exact report buckets, absence of
new `allow`/`expect`, removal of the fourteen reported bare allows and unsupported
grid-axis marker, zero unsafe matches across every owned Rust file, and a clean
worktree. No browser, generation, acquisition, artifact edit, dependency change,
or generator production change is permitted.

## 5 Completion

The canonical worker, task-review, correction/re-review, status, final-check,
holistic-review, publication, readback, and cleanup lifecycle applies to every
implementation task and the composed cycle. C07 acceptance is:

1. the accepted whole-crate inventory has complete, durable disposition evidence;
2. ten authorized task boundaries close all implementable opportunities and
   both escalations without behavior, API, artifact, dependency, or feature drift;
3. the sole generator opportunity is explicitly preserved as out-of-envelope
   handoff rather than implemented or silently lost;
4. every `FRI-08.14` invariant and all eight FRI-08 behavior closures remain true;
5. all 72 owned rows, FRI-09/F10 controls, centralized provenance, public API
   removal, and frozen C06 artifacts remain exact;
6. full verification and a fresh holistic review are clean without browser or
   generation; and
7. the final candidate is published and local `main`, tracking, and observed
   remote `main` are read back at one immutable SHA, followed by the complete
   59-finding audit and leaf/root handoff.

No blocker is currently known.
