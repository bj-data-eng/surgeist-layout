# P01-I08-S01-C01 Canonical Grid Topology And Placement

Status: draft

Cycle ID: `P01/I08/S01/C01`

Owning repository: `surgeist-layout`

Cycle base: `238df34a713db4f90d7f194f6fdf89a994d34fa2`

## 1 Authority

This just-in-time plan implements the reviewed specification
`plans/specs/P01-I08-grid-subgrid-and-grid-lanes-completeness.md`, SHA-256
`150c26e6c5b5fa703f090e861261ea2f03a7662caf4f83dfa52f49e40accb0ba`,
committed as `c7d10c23c0cdfebfba6a6606d9ea5b89352572f5`. Its controlling sections are
D-01 through D-06, D-12, D-13, sections 7 and 9, and the relevant topology,
placement, names, error, architecture, finding-closure, and acceptance portions
of sections 12 and 14 through 17. It closes `GRID-001`, `GRID-005`, and
`GRID-008` only.

The durable sequence is
`plans/sequences/P01-I08-S01-grid-subgrid-and-grid-lanes-completeness.md`,
SHA-256 `62e6b43402a038e7df5bc22e5c28ee40b7e7ae1a1ac6fc28224c12626cc9ca7c`,
committed as `75801ea77e37af28c0dda32a28fd1647123e1293`. This is its C01 entry.

The specification owns behavior. The sequence owns cycle order. This plan owns
only C01 execution detail and may not reinterpret either reviewed source.

## 2 Entry State And Bounded Outcome

The current grid path expands sized track lists, derives some named-line facts
from template areas, guesses implicit demand from visible-child and span sums,
sizes tracks, and then resolves placement against fixed scalar dimensions.
Valid placement beyond that guessed matrix can become a zero-area sentinel.
Duplicate name tokens on one line multiply occurrences. An empty area-only
template has no durable explicit topology.

C01 introduces one private canonical expanded topology before scalar sizing.
It owns, per axis, explicit track identity, template-area extent, named-line
membership, track origin, auto-track pattern phase, and integer half-open placed
areas. Placement uses growable scalar-free occupancy and creates exactly the
leading or trailing implicit tracks demanded by cursor semantics. Geometry is
materialized only after the resulting tracks are sized.

The new private module is `src/grid/topology.rs`. It is the sole owner of the
expanded topology. `src/grid/named.rs` continues to own name extraction and
occurrence resolution, but consumes topology line membership and returns each
matching physical line at most once. `src/grid/placement.rs` owns scalar-free
cursor phases and growable occupancy. `src/grid/mod.rs` orchestrates the phase
order and materializes settled integer areas into geometry. Track expansion
helpers may remain beside existing track definitions when they are pure inputs
to topology construction; they do not become a second expanded-topology owner.

C01 retains enough origin metadata for C02 auto-fit collapse, then invokes the
existing ordinary sizing path after settled placement. It does not change that
sizing path's policy or algorithms and does not implement auto-fit collapse.
Fit-content, stretch, all grid-lanes policies, standalone subgrid traversal,
browser inputs, generator behavior, provenance, artifacts, public APIs, and
documentation remain with their sequenced owners. C01 adds no dependency,
feature, allow/expect attribute, unsafe code, expected-fail row, quarantine,
generated artifact, or compatibility shim.

## 3 Compatibility And Frozen Evidence

Public API, documented error shape, dependencies, features, MSRV, and root
follow-up remain unchanged. C01 changes only valid ordinary-grid topology,
placement, names, and their existing layout results. It owns no documentation
or example update; the initiative documentation and root handoff remain with
their sequenced owners. Owned code remains free of `unsafe`. Invalid input
remains an error, fallible mutations remain atomic, cache results remain
equivalent, and f32/f64 behavior remains supported.

Browser and generator inputs are frozen throughout C01. The published
schema-3 report remains the sole provenance record with:

- 5,736 generated rows, 16 unsupported rows, 3 expected-fail source records,
  zero quarantined rows, and zero failed rows;
- corpus SHA-256
  `4419c4aab9429d1f81ac46426095719e19cf92cfbf51caf66d4f737c07c452cc`;
- helper SHA-256
  `caafa5a48787c9b80a45d8b2c8ac6f91b8ad7ab14a85e5bcdf3a3e922ebce019`;
- report SHA-256
  `5c560f240d27ad28d00023156b0bf2744aa8392d34fe916d800e02894e10353f`;
- 1,438 HTML inputs and 5,736 generated XML files.

The browser-free `grid` diagnostic currently selects 90 rows. C01 must improve
its owned behavior without promising an aggregate count reduction because
later-cycle failures and cross-feature composition remain visible.

## 4 Task Order

Tasks execute sequentially. Each task receives a fresh implementation worker,
a task-specific RED proof before production edits, focused GREEN evidence, one
exact commit, and a fresh exact-range task review before the next task starts.

### 4.1 `P01/I08/S01/C01/T01` Establish Canonical Explicit Topology And Named Membership

**Owned files:** new `src/grid/topology.rs`; topology wiring in
`src/grid/mod.rs`; only the track-expansion support needed by the topology;
`src/grid/named.rs`; focused tracked tests in the existing grid test module;
and `src/lib_tests.rs` only to add `src/grid/topology.rs` to its exhaustive
production-source audit manifest. No other `src/lib_tests.rs` behavior changes.

**Outcome:** Build one canonical per-axis topology from expanded template track
lists, template-area dimensions, authored named lines, area-derived names, and
auto-repeat origin. The explicit count is the maximum of the sized-list and
area dimension. Area-only tracks use the corresponding `grid-auto-*` pattern
with stable phase. Each physical line exposes set-like name membership while
retaining all authored and area-derived origin evidence. Named occurrence
lookup counts a matching physical line once even when the token is duplicated
or multiple origins collide there.

**Required RED prefix:** `fri08_c01_topology_` proves at least:

- an empty three-column template-area rectangle creates three explicit columns
  and uses the authored auto-column size pattern;
- a sized-list/area dimension disagreement chooses the larger explicit count
  in each axis without discarding the smaller source's names;
- a duplicated token on line one plus the same token on line two resolves its
  second occurrence to line two, not line one again;
- authored and area-derived identical names on one line remain one occurrence;
- positive, negative, and span-based lookup sees the same topology.

The corrected `GRID-005` oracle is a `120px × 20px` area-only result, not the
stale `10px × 10px` expectation.

**Acceptance:** The topology is private, deterministic, scalar-free, and has no
child-count input. Empty and populated area-only templates, row/column axis
symmetry, leading/trailing line names, repeated auto-pattern phase, auto-repeat
origin metadata, duplicate tokens, origin collisions, invalid names, and f32/
f64 callers have explicit tests. Existing negative-line and named-grid tests
remain green. The production-source audit includes the new topology module and
continues to inspect every production Rust source exactly once.

**Verification:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri08_c01_topology_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout named_grid
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout oracle_template_areas
cargo fmt --check
```

**Commit:** `refactor(grid): establish canonical explicit topology`

### 4.2 `P01/I08/S01/C01/T02` Growable Integer Placement Demand

**Owned files:** `src/grid/placement.rs`, the occupancy and implicit-extension
surface in `src/grid/topology.rs`, narrowly required orchestration in
`src/grid/mod.rs`, and focused tracked grid tests. T01 name semantics may be
consumed but not redesigned.

**Outcome:** Replace fixed scalar-sized placement occupancy with a growable
integer lattice over half-open row/column areas. Resolve explicit placements,
then one-axis-definite items, then fully automatic items. Row flow advances
columns then rows; column flow swaps the axes and phase constraints. Sparse
flow preserves a monotonic cursor. Dense flow restarts its search without
changing source/order precedence. Automatic spans reserve their full extent.
Implicit extension preserves leading/trailing auto-track pattern phase and
records exact demand for later sizing.

Absolutely positioned and `display:none` children neither occupy cells nor
create implicit tracks. Definite overlap remains legal and creates no demand
beyond the definite endpoints. A valid placement never becomes a zero-area
sentinel merely because it exceeds the pre-placement topology.

**Required RED prefix:** `fri08_c01_placement_` proves at least:

- the original span-after-occupied repro creates exactly one demanded row;
- the original definite-overlap repro creates no extra row;
- fully automatic spans search and reserve the whole span;
- row/column sparse flow and row/column dense flow differ only as specified;
- leading negative implicit demand retains correct line translation and auto
  pattern phase;
- absolute and display-none controls do not affect occupancy.

**Acceptance:** Integer placement is independent of measured track sizes.
Explicit overlap, mixed definite/automatic axes, order-modified source order,
dense backfill, sparse cursor monotonicity, negative lines, named spans,
leading/trailing implicit growth, automatic spans larger than explicit
topology, empty topology, invalid input, and overflow boundaries have direct
tests. Every successful in-flow child has an in-range nonempty integer area.

**Verification:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri08_c01_placement_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout grid_auto_placement
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout placement::
cargo fmt --check
```

**Commit:** `fix(grid): derive implicit demand from placement`

### 4.3 `P01/I08/S01/C01/T03` Integrate Placement Before Sizing Through Public Layout

**Owned files:** `src/grid/mod.rs`; placement-to-geometry integration in
`src/grid/placement.rs`; `src/grid/child.rs` only if required to consume settled
areas; narrowly required support in topology, tracks, and names; and focused
tracked grid tests. No public signature or downstream-cycle policy changes.

**Outcome:** Make canonical topology construction and integer placement precede
ordinary track sizing in the public layout path. Remove the visible-child,
span-product, and `div_ceil` implicit-demand guess. Extend the actual track
vectors to the exact settled placement bounds, size those vectors with existing
algorithms, then materialize child geometry from the settled areas. The same
areas feed child layout, subgrid contribution, baseline collection, and
overflow without an alternate placement computation.

**Required RED prefix:** `fri08_c01_composition_` proves at least:

- the public-layout span-after-occupied repro grows one exact row and returns
  the oracle geometry;
- public-layout definite overlap adds no implicit row and returns the oracle
  geometry;
- empty and populated area-only grids use the canonical explicit dimensions;
- duplicate-line membership changes public named placement as required;
- row/column, dense/sparse, order, percentage, and f64 composition consume the
  same settled integer areas.

**Acceptance:** The old demand heuristic and valid zero-area fallback are
absent. No parallel topology or final occupancy owner remains. Existing child,
subgrid contribution, baseline, overflow, order, percentage, writing-mode,
cache, rollback, invalid-input, and f32/f64 suites remain green. C02 receives
stable placements plus track-origin metadata without needing to reinterpret
areas or names.

**Verification:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri08_c01_composition_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri08_c01_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout grid_tests::
CARGO_NET_OFFLINE=true just verify
cargo fmt --check
```

**Commit:** `refactor(grid): place canonical topology before sizing`

## 5 Cycle Completion Gate

After T03 has a clean exact-range review, run the complete C01 acceptance gate:

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri08_c01_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
cargo fmt --check
git diff --check
git diff --exit-code 238df34a713db4f90d7f194f6fdf89a994d34fa2..HEAD -- Cargo.toml Cargo.lock README.md tests/layout/browser_parity tests/layout/browser_parity.rs tests/bin scripts
git status --short --branch
```

Also inspect every C01-owned Rust diff for any `unsafe`, new allow/expect
attribute, alternate topology/occupancy owner, stale child-count/`div_ceil`
demand path, and valid zero-area fallback. Verify the frozen artifact counts and
hashes in section 3 without invoking browser acquisition or generation.

The C01 candidate is publication-ready only when:

1. all three task commits and exact-range task reviews are clean;
2. one private canonical topology owns expanded track identity and origin;
3. growable integer placement determines exact implicit demand before sizing;
4. named occurrences count each matching physical line once;
5. public layout consumes the same settled areas for every downstream path;
6. all focused and repository gates above pass with no protected-file drift;
7. a fresh holistic cycle review returns no unresolved finding.

Follow the canonical planning gate for the status-only `complete` transition
after all task acceptance and task reviews are clean but before final checks and
holistic review. Follow the canonical publication gate for landing, remote
verification, and the handoff record; do not duplicate that choreography here.
If a final check or holistic review fails, use the planning gate's required
status transition and corrective-task path. C02 then receives the published C01
tip, stable integer placements, canonical expanded topology, and retained track
origin metadata. C02 alone owns changes to ordinary track-sizing policy and
algorithms, plus auto-fit collapse; C01 merely invokes the existing sizing path
after placement.
