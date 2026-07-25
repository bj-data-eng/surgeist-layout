# FRI-06-MR01 Post-C02 Sprawl Containment Contract

## FRI-06-MR01.1 Authority And Outcome

This specification is the authoritative desired-state contract for the first
bounded mechanical realignment identified by
`plans/P01-layout/P01-I06-mechanical-refactoring-review-findings.md`.
It applies after the published FRI-06-C02 candidate
`c26735c59874697084316bbe147b2f92a26728a1` and before FRI-06-C03 begins.

The owning repository is `surgeist-layout`. The outcome is a behavior-preserving
consolidation of two already-equivalent implementation surfaces:

- one private non-box role classifier used by inline text, line breaks, and inline
  boundaries during root-tree validation; and
- one scalar-generic `Compute` implementation for the existing test-only
  `OracleTreeOf<S>` harness.

This is supplemental governing design evidence for canonical
`P01/I06/S01/C03`, not a separate initiative or telemetry object. It occupies
the legacy C02-to-C03 window: legacy C02 remains canonical C02, this contract
governs inserted canonical C03, and legacy C03 is canonical C04. Completion
returns a remotely verified descendant that becomes the cycle base for
canonical C04.

## FRI-06-MR01.2 Scope And Non-Goals

The implementation scope is limited to `src/compute.rs`, focused validation
tests, `src/test_support/layout_tree.rs`, and focused oracle-harness tests. A
crate-private test module remains test-only even when its implementation is
generic over the crate's supported layout scalars.

The following remain out of scope:

- public API, error, type, trait, or module changes;
- changes to layout behavior, validation policy, cache behavior, transaction
  behavior, formatting algorithms, or FRI-06-C03 mixed participants;
- a new public or production tree abstraction;
- migration of local test trees, macros, failure fakes, observation fakes, or the
  broader `MR-002` test-harness opportunity;
- shaped-text linearization or any part of `MR-001`;
- scalar, geometry, scroll-padding, error-adapter, or layout-math consolidation;
- authored HTML/CSS, parser, helper, fixture, browser, XML, report, provenance,
  manifest, dependency, feature, root, sibling, or generated-artifact changes;
- generator architecture changes or generator execution; and
- any Surgeist-owned `unsafe` or new lint allowance.

## FRI-06-MR01.3 Current Evidence

At baseline `350dded41c45fc3f4638d2d93214ce741be7c8bf`,
`validate_layout_tree` repeats the same ordered non-box checks in the
`InlineText` branch and the combined `LineBreak`/`InlineBoundary` branch:

1. `tree.node_input(node)` equals `NodeInputOf::non_box()`;
2. `tree.child_count(node)` equals zero; and
3. `tree.has_leaf_measurement(node)` is false.

Both branches construct `LayoutInvalidInputOf::NonBoxNodeRole` with the same
reason and node site. Inline text separately marks unsupported parent placement
as later behavior after those intrinsic checks. Existing FRI-06 contract tests
cover all three individual inline-text failures in both scalar lanes, but they do
not explicitly characterize competing failures or the line-break and boundary
roles.

`src/test_support/layout_tree.rs` has one `Compute for OracleTree` body and one
`Compute for OracleTreeOf<f64>` body. `OracleTree` aliases
`OracleTreeOf<DefaultScalar>`, where `DefaultScalar` is `f32`. The bodies differ
only in scalar spelling. Their lookup, panic text, input recording, recorded
measurement precedence, algorithm dispatch, hidden output, and unreachable
inline-display branch are otherwise identical. `Traverse` and `Round` are
already generic over `S: LayoutScalar`.

The mechanical review validated both opportunities against the source and
placed this bounded slice after C02 and before C03. It did not authorize broader
harness migration or behavior changes.

## FRI-06-MR01.4 Resolved Decisions

### D-01 One Non-Box Reason Classifier

`src/compute.rs` owns one private helper that inspects a tree and node and returns
`Option<NonBoxNodeRoleError>`. It checks canonical node input, child absence, and
leaf-measurement absence in that exact order. `None` means all shared non-box
invariants hold.

The helper returns only the role reason. Each existing layout-input branch keeps
its current error construction, node identity, return point, and role-specific
logic. This is the least powerful extraction that removes the repeated policy
without merging branch behavior.

Rejected alternatives:

- Returning a complete `LayoutErrorOf` would couple the helper to branch control
  flow and more generic error state than the shared invariant requires.
- Moving parent acceptance into the helper would conflate the inline-text role
  rule with intrinsic non-box validation.
- Broadening the helper to box-node or atomic validation would exceed the exact
  repeated structure.

### D-02 First-Error Semantics Are Observable

When multiple non-box invariants are invalid, validation reports the first reason
in this order:

| Invalid state | Reported reason |
| --- | --- |
| noncanonical node input, with any later invalid state | `NonCanonicalNodeInput` |
| canonical input plus children, with or without measurement | `HasChildren` |
| canonical input, no children, and leaf measurement | `HasLeafMeasurement` |

The error remains `LayoutOperation::RootLayout`, site `Node(node)`, and kind
`InvalidInput(NonBoxNodeRole { reason })`. Inline text still evaluates its parent
acceptance only after these checks. Line breaks and boundaries still return after
successful non-box validation.

### D-03 One Generic Oracle Compute Implementation

`src/test_support/layout_tree.rs` owns exactly one implementation with the shape
`impl<S: LayoutScalar> Compute for OracleTreeOf<S>`. The `OracleTree` alias and
all existing builders, fields, `Traverse`, `Round`, and call sites remain.

The generic body preserves these operations and their order:

1. `node_input` reads the stored layout input and returns its box input;
2. missing layout input and non-box layout input retain their exact panic text;
3. `layout_input` clones the stored input and retains its missing-input panic;
4. `set_unrounded` stores the scalar-matched node output;
5. `compute_child` records the input before any result is selected;
6. a matching recorded measurement wins before algorithm dispatch;
7. block, flex, grid, and grid-lanes dispatch to their existing algorithms;
8. hidden display stages the zero-source output and returns `HIDDEN`; and
9. inline display variants remain unreachable after `inner_display`.

No new trait, wrapper, macro, type-erased value, dynamic dispatch, or production
surface is introduced. The test harness remains a fixture-phase helper whose
semantics match the production `Compute` boundary it exercises.

### D-04 Characterization Before Mechanical Change

This sub-initiative changes no intended behavior, so artificial RED evidence is
prohibited. Before each extraction, focused characterization must pass at the
exact task base.

Non-box characterization covers all three roles in both scalar lanes and proves
the competing-failure order in D-02. Oracle characterization exercises both
supported scalar lanes through the same typed helper and proves input recording,
recorded-measurement precedence, hidden staging, and representative algorithm
dispatch. Existing package tests remain the authority for the wider block, flex,
grid, grid-lanes, error, panic, and rounding behavior.

Static source evidence proves that the repeated branches or scalar-specific impls
are absent after extraction. Characterization tests must assert behavior, not the
private helper's name or internal call count.

## FRI-06-MR01.5 Behavior And Preservation Matrix

| Surface | Required preserved state | Required evidence |
| --- | --- | --- |
| inline-text non-box validation | exact reason order, site, operation, kind, and parent-rule ordering | both scalar lanes, single and competing invalid states |
| line-break non-box validation | exact reason order, site, operation, kind, and immediate successful return | both scalar lanes, single and competing invalid states |
| inline-boundary non-box validation | exact reason order, site, operation, kind, and immediate successful return | both scalar lanes, single and competing invalid states |
| oracle input lookup | same stored references/clones and panic text | existing suite plus focused both-scalar characterization |
| oracle child computation | input recorded first; measurement before dispatch; same display dispatch | focused both-scalar characterization and package suite |
| hidden oracle node | zero-source staged output and `HIDDEN` result | focused both-scalar characterization |
| test architecture | one generic impl; no scalar-specific duplicate; no harness migration | static source check and exact diff inventory |

All focused tests use the public root layout front door for validation behavior or
the existing test-only `Compute` boundary for oracle behavior. A private helper
unit test may supplement but never replace those paths.

## FRI-06-MR01.6 Module And Compatibility Contract

`src/compute.rs` may add one private function and replace only the two repeated
reason-selection blocks with calls to it. Focused tests may be added to an
existing Rust test module that already owns the affected front door.

`src/test_support/layout_tree.rs` replaces the two `Compute` implementations with
one generic implementation. Focused test support may stay in that module or an
existing focused Rust test module. No non-test module may depend on test support.

Compatibility classification: internal-only and behavior-preserving. Public
exports, signatures, trait requirements, enum variants, panic policy, error
payloads, dependencies, features, lockfile, MSRV, docs/examples, and root handoff
surface are unchanged.

## FRI-06-MR01.7 Verification And Acceptance

The focused test prefix is `fri06_mr01_`. Acceptance requires:

1. pre-change characterization passes at each task base; no false RED is claimed;
2. the non-box classifier has one implementation and both role branches use it;
3. every D-02 error precedence case passes for inline text, line break, and inline
   boundary in both scalar lanes;
4. `OracleTreeOf<f32>` and `OracleTreeOf<f64>` satisfy the same characterized
   `Compute` behavior;
5. exactly one scalar-generic oracle `Compute` implementation remains and neither
   scalar-specific implementation remains;
6. no generator input or output changes; and
7. no dependency, feature, manifest, lockfile, public API, docs/example, root, or
   sibling change occurs.

Exact task decomposition, commands, review gates, publication, and resource
cleanup are owned by the canonical C03 cycle plan, repository command inventory,
and selected workflow authority.

The successful C03 result becomes the base for legacy FRI-06-C03, now canonical
`P01/I06/S01/C04`, and keeps the later review windows unchanged: `MR-001`,
`MR-004`, and `MR-005` wait until after legacy FRI-06-C05; `MR-003` and broad
`MR-002` migration wait until after the legacy FRI-06-C07 leaf candidate
handoff.
