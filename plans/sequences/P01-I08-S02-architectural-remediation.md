# P01-I08-S02 Architectural Remediation Implementation Sequence

Sequence ID: `P01/I08/S02`

Owning repository: `surgeist-layout`

## 1 Authority

This sequence implements the independently reviewed post-C08 remediation in
`plans/specs/P01-I08-grid-subgrid-and-grid-lanes-completeness.md`, normalized
semantic-content SHA-256
`d9c6a61eae363331d7e8ce05d88916099111e11b8793b8dc31cc55e3e5c80a6a`,
committed as `b9cb82aadf70d5690d605bb9ffeaa6da9512bd3d`.

The specification owns the seven accepted pressure findings, compatibility,
module ownership, frozen artifacts, verification, the FRI-09 hold, and final
acceptance. This sequence owns only durable cycle order and boundaries. Each
cycle receives its detailed plan only after its predecessor is published,
remotely read back, process-clean, and post-cycle-cleaned.

## 2 Sequence Boundary

Every cycle is owned by `surgeist-layout` and preserves public layout behavior,
public API, dependencies, features, MSRV, and the complete browser artifact
state. Root, siblings, authored CSS/style lowering, shaping, rendering, retained
identity, FRI-09 implementation, generator changes, browser execution,
generation, and acquisition remain excluded.

The immutable entry is the published planning baseline at
`28d4016e7bf1005b8541868e8b1d251b0e03012c`, whose history contains the
published C08 behavioral candidate and reviewed FRI-09 hold. Each successor
cycle consumes only an immutable published predecessor.

## 3 Durable Order

```text
R01 neutral engine and sizing contracts
  -> R02 session, computation, measurement, and rounding owners
  -> R03 scroll module ownership
       -> R04 block and flex phase ownership --+
       -> R05 grid tracks and child ownership --+-> R06 node projections and compatible public API map
  -> R07 test ownership
  -> R08 whole-crate testing-reference conformance and final candidate
```

R01 establishes the dependency direction consumed by R02. R02 removes central
compute ownership before algorithm files are partitioned. R03 settles the
cross-format scroll substrate before R04 and R05 move its consumers. R04 and R05
are independently ready after R03 and may execute in either order; neither
consumes the other's source. R06 waits for both algorithm boundaries so
projections follow actual semantic roles. R07 moves tests only after production
ownership is stable. R08 then enforces the testing reference across the entire
crate after file ownership and test partitioning can no longer churn its audit.

## 4 `P01/I08/S02/R01` Neutral Engine And Sizing Contracts

**Owner:** `surgeist-layout`.

**Specification:** `FRI-08.20` rows `AR-001` and `AR-003`, `FRI-08.21`,
`FRI-08.22`, the `error`, `tree`, `engine::contracts`, and `sizing::resolve`
portions of `FRI-08.23`,
`fri08_remediation_engine_contract_is_algorithm_neutral`,
`fri08_remediation_sizing_resolution_has_one_owner`, and
`fri08_remediation_public_api_inventory_is_compatible` in `FRI-08.27`, and
acceptance rows `FRI-08.28(1)` through `FRI-08.28(4)`.

**Entry:** the remediation specification and this sequence are independently
clean at the published planning baseline.

**Outcome:** public host contracts, public errors, private recursive services,
and shared sizing resolution have separate neutral owners; no shared recursive
trait mentions block state.

**Exit evidence:** dependency direction, public compatibility, resolver parity,
error-site parity, source inventory, and full affected behavior satisfy the
cited specification sections.

**Handoff:** publish the neutral-contract candidate to R02.

## 5 `P01/I08/S02/R02` Session, Computation, Measurement, And Rounding Owners

**Owner:** `surgeist-layout`.

**Specification:** `FRI-08.20` row `AR-002`, `FRI-08.21`, the
`engine::validation`, `engine::session`, `engine::root`, `engine::rounding`,
`measurement`, and `engine::mod` portions of `FRI-08.23`,
`fri08_remediation_engine_session_transaction_equivalence` and
`fri08_remediation_public_api_inventory_is_compatible`; and acceptance rows
`FRI-08.28(1)` through `FRI-08.28(4)`.

**Entry:** R01's neutral recursion, errors, host contracts, and sizing resolver
are published.

**Outcome:** validation, invalidation, staging, dispatch, root computation,
rounding, leaf measurement, and batch assembly have one responsibility-shaped
owner, with no production `compute.rs` or `traits.rs` owner remaining.

**Exit evidence:** cold/warm cache, invalidation, error, measurement, rounding,
transaction, dispatch, source-inventory, and public-front-door equivalence
satisfy the cited specification sections.

**Handoff:** publish the completed engine decomposition to R03.

## 6 `P01/I08/S02/R03` Scroll Module Ownership

**Owner:** `surgeist-layout`.

**Specification:** `FRI-08.20` row `AR-004`, `FRI-08.21`, `FRI-08.24.1`, the
scroll projection boundary in `FRI-08.25`,
`fri08_remediation_scroll_construction_and_rounding_equivalence` and
`fri08_remediation_public_api_inventory_is_compatible` in `FRI-08.27`, and
acceptance rows `FRI-08.28(1)`, `FRI-08.28(5)`, and `FRI-08.28(8)` through
`FRI-08.28(10)`.

**Entry:** R02's engine boundaries are published and canonical scroll callers
remain behaviorally unchanged.

**Outcome:** scroll models, box geometry, contributions, construction, and
rounding have one private module owner apiece without a second construction
path.

**Exit evidence:** public privacy, clips/gutters/ranges, contribution,
reconstruction, rounding, scalar/writing-mode, cache, and artifact invariants
satisfy the cited specification sections.

**Handoff:** publish the settled scroll substrate to R04.

## 7 `P01/I08/S02/R04` Block And Flex Phase Ownership

**Owner:** `surgeist-layout`.

**Specification:** `FRI-08.20` row `AR-004`, `FRI-08.21`, `FRI-08.24.2`, the
block/flex projection boundary in `FRI-08.25`, the block/flex cases of
`fri08_remediation_algorithm_phase_composition_equivalence` plus
`fri08_remediation_public_api_inventory_is_compatible` in `FRI-08.27`, and
acceptance rows `FRI-08.28(1)`, `FRI-08.28(5)`, and `FRI-08.28(8)` through
`FRI-08.28(10)`.

**Entry:** R03's canonical scroll owners and R02's engine services are
published; R05 is not a prerequisite.

**Outcome:** block and flex source ownership follows their specified semantic
phases with narrow private carriers and no copied sizing or scroll policy.

**Exit evidence:** block/inline/float/BFC and flex item/line/distribution/
alignment/intrinsic/absolute/scroll behavior, scalar lanes, cache, public API,
and artifact state satisfy the cited specification sections.

**Handoff:** publish stable block/flex phase boundaries; R06 consumes them after
R05 is also complete.

## 8 `P01/I08/S02/R05` Grid Tracks And Child Phase Ownership

**Owner:** `surgeist-layout`.

**Specification:** `FRI-08.20` row `AR-004`, `FRI-08.21`, `FRI-08.24.3`, the
grid projection boundary in `FRI-08.25`, the grid cases of
`fri08_remediation_algorithm_phase_composition_equivalence` plus
`fri08_remediation_public_api_inventory_is_compatible` in `FRI-08.27`, and
acceptance rows `FRI-08.28(1)`, `FRI-08.28(5)`, and `FRI-08.28(8)` through
`FRI-08.28(10)`.

**Entry:** R03 has published stable engine and scroll boundaries; R04 is not a
prerequisite.

**Outcome:** grid track sizing and settled child layout follow the specified
phase owners without duplicating topology, placement, baseline, subgrid, or
track state.

**Exit evidence:** all eight original FRI-08 closures, topology/placement,
ordinary tracks, intrinsic/subgrid/lanes sizing, child/baseline/absolute/scroll
composition, scalar/oracle/parity controls, and artifact state satisfy the cited
specification sections.

**Handoff:** publish stable grid phase boundaries; R06 consumes them after R04
is also complete.

## 9 `P01/I08/S02/R06` Node Projections And Compatible Public API Map

**Owner:** `surgeist-layout`.

**Specification:** `FRI-08.20` row `AR-005`, `FRI-08.21`, all of
`FRI-08.25`, `fri08_remediation_node_projection_role_boundaries` and
`fri08_remediation_public_api_inventory_is_compatible` in `FRI-08.27`, and
acceptance rows `FRI-08.28(1)`, `FRI-08.28(6)`, and `FRI-08.28(8)` through
`FRI-08.28(10)`.

**Entry:** R03 through R05 expose stable algorithm roles from which projections
can be derived.

**Outcome:** algorithm phases consume role-specific private node projections,
node-input type ownership is partitioned, and one README API map documents the
unchanged public snapshot and root facade.

**Exit evidence:** role-boundary coverage, public construction and source/API
inventory equality, compile contracts, documentation, full algorithm behavior,
and frozen artifacts satisfy the cited specification sections.

**Handoff:** publish the compatible model/API candidate to R07.

## 10 `P01/I08/S02/R07` Test Ownership

**Owner:** `surgeist-layout`.

**Specification:** `FRI-08.20` row `AR-006`, `FRI-08.21`, `FRI-08.26`, and
the partition-preservation and ordinary verification portions of `FRI-08.27`.

**Entry:** R01 through R06 are published; all production owners and public
compatibility evidence are stable.

**Outcome:** the four large companion suites follow semantic production owners
and shared fixtures have one test-only owner, without coverage, API, behavior,
artifact, or ownership drift.

**Exit evidence:** partitioned test inventory, body/assertion preservation,
focused prefix searchability, shared-fixture ownership, ignored-state equality,
full default/generator/corpus/Taffy matrices, public compatibility, and frozen
artifact state satisfy the cited sections. Whole-crate evidence classification
and final acceptance remain R08-owned.

**Handoff:** publish the partitioned test candidate to R08.

## 11 `P01/I08/S02/R08` Whole-Crate Testing-Reference Conformance

**Owner:** `surgeist-layout`.

**Specification:** `FRI-08.20` row `AR-007`, `FRI-08.21`, `FRI-08.27.1`, and
all of `FRI-08.28`.

**Entry:** R01 through R07 are published; production and test file ownership is
stable. The cycle begins from a complete inventory of every tracked Rust test,
its evidence class, and the exact ignored list.

**Outcome:** the entire crate test suite conforms to the installed Surgeist
testing reference. Source/token/symbol/file-placement proxies, planning and
workflow state checks, current-output oracles, and unjustified ignored tests are
removed or replaced. Legitimate declared artifact, manifest, schema, report,
and serialization contracts are exercised through their consumers where
possible. Workflow audits remain outside `cargo test`.

**Exit evidence:** every removed or replaced test has an explicit disposition;
behavioral, compile-contract, oracle, artifact-consumer, scalar, cache,
transaction, browser-parity, and generator coverage remains green; the exact
before/after discovered inventory and justified ignored list reconcile; static
conformance audits find no prohibited test evidence; and every final API,
dependency, MSRV, safety, artifact, publication, readback, and cleanup gate in
`FRI-08.28` passes.

**Handoff:** publish and read back the immutable final FRI-08 remediation leaf
candidate; record the still-held FRI-09 sequence and stop.

## 12 Sequence Completion

This sequence is complete only when R01 through R08 satisfy their exits in order
and `FRI-08.28` is satisfied. It does not revise or begin FRI-09.
