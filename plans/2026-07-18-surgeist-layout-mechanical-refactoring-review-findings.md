# Surgeist Layout Mechanical Refactoring Review Findings

Date: 2026-07-18
Status: repository-wide static review snapshot; mechanical refactoring opportunities only

## Executive Assessment

The repository contains six high-confidence opportunities to reduce repeated code,
unnecessary allocation, and maintenance surface without changing the public API or
crate boundary. The clearest low-risk candidates are the repeated scroll/geometry
glue, the tiny scalar and geometry primitives, the repeated non-box validation, and
the duplicated generic test-support implementation. The shaped-text scan has the
largest likely runtime payoff, but it is also the most behavior-sensitive candidate
and should be protected by focused characterization tests before it is changed.

This report complements
[`2026-07-10-surgeist-layout-full-code-review-findings.md`](2026-07-10-surgeist-layout-full-code-review-findings.md).
It does not supersede that review and does not reclassify any of its correctness or
conformance findings. The items below are refactoring opportunities, not claims that
the current behavior is incorrect.

## Scope And Boundary

The review covered the crate implementation, its public front door, and its tracked
test support. It used commit `eaaf37dd0ad80a763acd24f67dc7df4af988b47e` as the last
stable committed baseline available during the inspection.

Concurrent uncommitted FRI-06 work was present in `src/block.rs`, `src/compute.rs`,
`src/inline.rs`, and `src/root_tests.rs` when this report was written. Those edits
were preserved and were not treated as settled implementation. Findings are therefore
anchored to named modules, functions, and repeated structures rather than fragile line
numbers.

The review respected the repository boundary:

- `surgeist-layout` owns the layout algorithms, layout contracts, focused tests, and
  layout-ready fixtures discussed here.
- No authored CSS, style resolution, retained identity, text shaping, rendering,
  root-facade integration, or generated API artifact work is proposed.
- No public API or dependency change is required by any finding.
- Browser-parity data and generated fixture contents were not reviewed for mechanical
  rewriting.

The source inspection was read-only. No build, test, formatter, linter, benchmark, or
generator command was run as part of the review. This Markdown report is the only
repository artifact produced from it.

## Method And Prioritization

Candidates were selected when the source showed one or more of the following:

- repeated implementations with equivalent behavior;
- repeated test scaffolding obscuring the behavior under test;
- avoidable allocation or repeated traversal in an unbounded input path;
- small policy-free primitives implemented independently in several modules; or
- validation branches constructing the same error for the same invariant.

Candidates were excluded when superficially similar code encoded observably different
policy. Each finding identifies those preservation boundaries explicitly.

| ID | Opportunity | Expected payoff | Refactoring risk |
| --- | --- | --- | --- |
| MR-001 | Make shaped-text processing linear | High | Medium |
| MR-002 | Consolidate test tree harnesses | High | Medium |
| MR-003 | Extract the identical subset of layout math helpers | Medium-high | Medium |
| MR-004 | Centralize scroll-padding conversion and geometry error glue | Medium | Low |
| MR-005 | Unify tiny scalar and geometry primitives | Small-medium | Low |
| MR-006 | Extract repeated non-box validation | Small | Low |

## Findings

### MR-001 — Make Shaped-Text Processing Linear

**Scope:** `src/node_input.rs` and `src/inline.rs`

`InlineTextInputOf::try_new` checks each segment identifier against all preceding
segments. Because `InlineSegmentId` supports hashing and the segment list is not
otherwise bounded to a small constant, validation performs quadratic work for a long
input even though it only needs to identify the first duplicate encountered.

The shaped-text line-selection path compounds the same scaling concern:

- `select_text_line` allocates a `Vec<bool>` and materializes a complete
  `SelectedTextLineOf` value.
- `shaped_text_min_content` and `shaped_text_max_content` call that full selector even
  though they only consume `used_inline_extent`.
- `layout_shaped_text_run` repeatedly calls `pending_text_inline_extent` over the
  growing slice from the current line start through the current scan position.

The last operation repeatedly folds the same prefix and can make line construction
quadratic in the number of shaped-text participants. The repeated allocation in the
intrinsic-size paths is a separate, smaller cost.

**Mechanical extraction:**

- Validate identifiers with a scanning `HashSet`, preserving the current first-error
  order and error payload.
- Introduce a private allocation-free line summary containing only the discard bounds,
  metrics, used extent, and replacement decision needed while scanning.
- Maintain the pending inline extent incrementally, or derive it from a prefix
  representation, rather than folding the growing line slice for every participant.
- Materialize selected-segment state only when a line is committed and the caller
  actually needs it.

**Preservation requirements:**

- Preserve the exact duplicate selected when more than one identifier is repeated.
- Preserve forced-break, soft-wrap, replacement, empty-line, and overflow behavior.
- Preserve all current scalar-generic behavior and error shapes.
- Characterize intrinsic sizing and committed-line projection before changing the
  scan, including long runs and breaks at the first and last participant.

This is the highest-payoff candidate in the report, but it should not be bundled with
unrelated cleanup because its state transitions are behavior-sensitive.

### MR-002 — Consolidate Test Tree Harnesses

**Scope:** `src/test_support/layout_tree.rs`, `src/block_tests.rs`,
`src/flex_tests.rs`, `src/grid_tests.rs`, and `src/root_tests.rs`

The focused tests repeatedly define local tree types and nearly identical `Traverse`
and `Compute` implementations. At the review baseline, the four principal test modules
contained 215 `Traverse` and 206 `Compute` implementation declarations in total. These
are textual counts of declarations containing `Traverse for` or `Compute for`:

| Module | `Traverse` implementations | `Compute` implementations |
| --- | ---: | ---: |
| `src/block_tests.rs` | 37 | 36 |
| `src/flex_tests.rs` | 68 | 67 |
| `src/grid_tests.rs` | 95 | 95 |
| `src/root_tests.rs` | 15 | 8 |

Not all of these implementations are equivalent: several deliberately inject
failures, record calls, or model unusual child behavior. The volume nevertheless
contains a large set of plain map-backed trees that differ only in their stored input
data.

The existing `OracleTreeOf<S>` support type already provides a suitable generic
foundation, but its `Compute` implementation is duplicated once for `f32` and once for
`f64`. The bodies are scalar-independent and can be represented by one
`impl<S: LayoutScalar> Compute for OracleTreeOf<S>`.

**Mechanical extraction:**

- Replace the two scalar-specific `OracleTreeOf` `Compute` implementations with one
  generic implementation.
- Add or extend a typed, map-backed `PublicLayoutTreeOf<S>` harness for the ordinary
  public-front-door cases: children, layout input, node input, and optional measurement
  or cache behavior.
- Migrate equivalent local fakes piecemeal so each test continues to expose its
  scenario-specific data.
- Retain dedicated fakes for failure injection, observation, call ordering, and other
  intentionally specialized behavior.

**Preservation requirements:**

- Keep the public `LayoutTree` and internal `Compute` phases visible where a test is
  specifically asserting their boundary.
- Do not hide behavior in broad declarative macros; a typed harness keeps inputs and
  failure behavior inspectable.
- Preserve panic messages, injected error sites, call logs, cache keys, and child order
  when migrating a specialized test.

This refactor offers the largest maintenance reduction. The generic `OracleTreeOf`
implementation is a small low-risk first slice; broader harness migration should stay
incremental.

### MR-003 — Extract The Identical Subset Of Layout Math Helpers

**Scope:** `src/block.rs`, `src/flex.rs`, `src/grid/mod.rs`, and `src/compute.rs`

These modules independently define closely related `SizeOptionExt`, `ScalarExt`, and
length-resolution helpers. A useful common subset is textually and semantically
equivalent:

- option fallback and unwrapping;
- optional addition;
- aspect-ratio application;
- the shared resolution-to-zero and optional-resolution behavior used by block, flex,
  and grid; and
- containing-flow-relative padding and border resolution in each algorithm's constants
  construction.

Moving only this subset to a crate-private internal module would give the algorithms a
single implementation for policy-free arithmetic while keeping their higher-level
layout decisions local.

The apparent duplication is not uniform. In particular:

- block's `sub_optional` clamps at zero while the flex and grid versions do not; and
- grid's scalar clamp applies the minimum before the maximum, while block, flex, and
  compute apply the maximum before the minimum.

Those differences are observable for negative or conflicting constraints and must not
be erased by a mechanical merge.

**Mechanical extraction:**

- Introduce crate-private helpers only for operations proven equivalent across every
  caller.
- Give policy-specific operations explicit local names rather than forcing them behind
  one shared trait method.
- Consolidate padding and border resolution only after confirming that the containing
  flow and percentage bases are identical at each call site.

**Preservation requirements:**

- Characterize zero clamping, negative values, conflicting min/max values, indefinite
  dimensions, and aspect-ratio projection.
- Preserve operation order; floating-point and constraint operations must not be
  algebraically rearranged merely because the result usually agrees.
- Avoid a macro-generated wholesale unification of the helper blocks.

The repetition is substantial enough to justify extraction, but the differing policy
edges make this a selective refactor rather than a deletion-by-deduplication exercise.

### MR-004 — Centralize Scroll-Padding Conversion And Geometry Error Glue

**Scope:** `src/compute.rs`, `src/block.rs`, `src/flex.rs`, `src/grid/mod.rs`, and
`src/grid/child.rs`

The conversion from `ScrollPaddingOf<S>` to `OptimalRegionInsetsOf<S>` is repeated with
the same four-edge mapping in five places: the leaf, block, flex, grid, and grid-child
paths. The conversion is a type-level fact rather than an algorithm-specific layout
decision.

The block, flex, and grid algorithms also repeat the same mapping from scroll-geometry
errors to their own-geometry and child-geometry error variants. The only relevant
inputs are the site, run mode, and underlying error.

**Mechanical extraction:**

- Add a crate-private conversion on `OptimalRegionInsetsOf`, or an appropriate `From`
  implementation, for `ScrollPaddingOf`.
- Add one compute-owned helper that maps `(site, run_mode, error)` to the correct
  geometry error while preserving the existing own-versus-child distinction.
- Keep the error adapter on the compute/layout side so the lower-level scroll module
  does not acquire a dependency on layout algorithm error types.

**Preservation requirements:**

- Preserve the exact physical-edge mapping.
- Preserve the existing error variant, node/site identity, and run-mode classification.
- Keep module dependency direction unchanged.

This is a low-risk candidate because the repeated blocks are small, exact, and already
expressed in shared types.

### MR-005 — Unify Tiny Scalar And Geometry Primitives

**Scope:** `src/value.rs`, `src/sizing.rs`, `src/node_input.rs`, `src/scroll.rs`,
`src/compute.rs`, `src/block.rs`, and `src/geometry.rs`

Several tiny primitives have independent copies:

- signed-zero canonicalization appears as `canonical_zero`,
  `canonical_calc_size_zero`, `canonical_exclusion_zero`, and
  `canonical_scroll_zero`;
- layout-coordinate rounding with `(value + 0.5).floor()` appears in compute and
  scroll; and
- selecting an edge by physical side is repeated in block, compute, scroll, and a
  private geometry helper.

These operations are small, but centralizing them prevents subtle divergence in code
that establishes canonical numeric and geometric representation.

**Mechanical extraction:**

- Put signed-zero canonicalization in the crate-private scalar layer and reuse it from
  the domain-specific constructors.
- Name the `(value + 0.5).floor()` operation as layout-coordinate rounding and share it
  between compute and scroll.
- Expose the existing geometry edge selector crate-privately, or add a typed accessor
  on the edge structure, and remove the repeated physical-side matches.

**Preservation requirements:**

- Do not replace layout-coordinate rounding with `LayoutScalar::round`; negative
  half-values can differ.
- Preserve positive zero as the canonical representation without changing NaN or
  infinity handling.
- Preserve physical, not logical, edge selection.

Each extraction is individually small and suitable for an isolated, easily reviewed
change.

### MR-006 — Extract Repeated Non-Box Validation

**Scope:** `src/compute.rs`, in `validate_layout_tree`

The `LayoutInputOf::InlineText` validation branch and the line-break/inline-boundary
branch repeat the same checks and reason construction for a non-box node:

- the node input must be `NodeInput::non_box`;
- the node must have no children; and
- the node must not expose leaf measurement.

Inline text has an additional parent-acceptance rule, but that rule is independent of
the shared non-box invariants.

**Mechanical extraction:**

- Add a private helper that validates the three non-box invariants and returns the
  existing reason or error value.
- Keep the inline-text parent rule in the inline-text branch.
- Have both branches call the helper before applying their role-specific checks.

**Preservation requirements:**

- Preserve validation order and therefore the first reported reason when several
  invariants fail.
- Preserve the current node identity and error payload.
- Do not broaden the helper to box-node validation or parent-role validation.

This is a small, low-risk cleanup that makes the distinction between shared non-box
invariants and role-specific validation explicit.

## Safety Boundaries

The following tempting consolidations should remain out of scope unless separate
behavioral evidence justifies them:

- Do not merge all size and scalar helper traits wholesale; zero subtraction and clamp
  ordering already encode different behavior.
- Do not replace layout rounding with the general scalar rounding operation.
- Do not force failure-injecting or observation-oriented test trees into a generic
  map-backed harness.
- Do not invert the dependency from scroll primitives toward block, flex, grid, or
  compute error types.
- Do not combine shaped-text scan optimization with unrelated line-breaking behavior
  changes.
- Do not mechanically rewrite generated or browser-parity fixture data.

## Recommended Integration Windows

This review is a holistic maintenance alignment, not authority to widen an active
reviewed cycle. Incorporate its opportunities at explicit boundaries where the
affected architecture is settled and the refactor can receive its own planning,
characterization, implementation, and review evidence.

| Opportunity | Earliest recommended insertion point | Integration boundary |
| --- | --- | --- |
| `MR-006` repeated non-box validation | After FRI-06-C02 is published and remotely verified, before FRI-06-C03 | Use one isolated mechanical task preserving first-error order. Do not amend C02-T4 or reopen its cache/rounding closure for unrelated cleanup. |
| `MR-002` generic `OracleTreeOf<S>` implementation only | After FRI-06-C02 is published and remotely verified, before FRI-06-C03 | Treat only the two scalar-specific equivalent implementations as the first bounded slice. Do not begin broad harness migration here. |
| `MR-001` shaped-text linearization | After FRI-06-C05 is published and remotely verified, before FRI-06-C06 | C03 first settles the mixed participant stream; C04 settles float-adjusted line bands; C05 settles provider-backed band queries. Then characterize the final scan, preserve every line transition, and validate scaling before C06 derives the final fixture lineage. |
| `MR-004` scroll-padding and geometry-error glue | After FRI-06-C05 is published and remotely verified, before FRI-06-C06 | Extract only mappings proven identical after rectangular and shaped float geometry is complete. Keep the adapter above the scroll primitive layer. |
| `MR-005` scalar and geometry primitives | After FRI-06-C05 is published and remotely verified, before FRI-06-C06 | Split signed-zero canonicalization, layout-coordinate rounding, and physical-edge selection into independently characterized tasks; do not combine their policies. |
| `MR-003` selective layout-math helpers | After FRI-06-C07 and the leaf candidate handoff | This spans block, flex, grid, and compute policy. Wait until FRI-06 behavior, artifacts, and candidate evidence are closed, then extract only the proven common subset. |
| `MR-002` broader test-harness migration | After FRI-06-C07 and the leaf candidate handoff | Inventory ordinary versus specialized fakes first, then migrate incrementally. Preserve dedicated failure, observation, ordering, cache, and topology harnesses. |

The earliest partial mechanical initiative can therefore begin after FRI-06-C02
with `MR-006` and the generic `OracleTreeOf<S>` slice. The performance- and
geometry-sensitive group begins only after FRI-06-C05, while the broad math and
test-support consolidation waits until the remediation initiative has published
its final leaf candidate.

The report itself remains outside the active C02 exact cycle range. Commit and
publish it as separate review evidence after the C02 candidate is published, so
C02 range inventory and holistic review remain traceable to their reviewed plan.

## Verification Expectations For Future Refactors

Each eventual implementation should remain behavior-preserving and use the repository's
canonical verification workflow. Focused characterization should precede changes where
the preservation boundary is not already explicit in tests.

At minimum, future changes should verify:

- first-error ordering for duplicate segments and non-box validation;
- forced and soft line breaks, replacements, intrinsic extents, and long shaped-text
  runs for MR-001;
- both supported scalar types and specialized error/call-observation harnesses for
  MR-002;
- negative, indefinite, zero, conflicting-constraint, and aspect-ratio cases for
  MR-003 and MR-005; and
- every geometry site/run-mode/error mapping for MR-004.

Performance claims for MR-001 should be supported by a focused scaling measurement or
benchmark rather than inferred solely from reduced asymptotic work.

## Review Limitations

- This was a static mechanical review, not a correctness, security, conformance, or
  performance audit.
- No Cargo, formatter, linter, generator, or browser command was executed.
- Concurrent uncommitted FRI-06 source changes were not treated as stable review
  material.
- The count of test trait implementations is a baseline inventory, not a claim that
  every counted implementation should be removed.
- Suggested extractions were not implemented, so their final naming and internal module
  placement remain implementation decisions.

## Final Conclusion

The repository has a useful sequence of behavior-preserving cleanup available without
changing its public contract. MR-004 through MR-006 and the generic `OracleTreeOf`
portion of MR-002 are the clearest low-risk starting points. MR-001 offers the largest
likely runtime improvement, while MR-003 and the broader test-harness consolidation
offer meaningful maintenance gains when performed selectively. The explicit safety
boundaries above are essential: similar-looking layout arithmetic and test scaffolding
are not automatically behavior-equivalent.
