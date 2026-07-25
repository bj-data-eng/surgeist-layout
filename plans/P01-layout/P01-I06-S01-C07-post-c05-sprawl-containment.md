# FRI-06-MR02 Post-C05 Sprawl Containment Contract

Design owner: `surgeist-layout`

## FRI-06-MR02.1 Outcome And Authority

This specification defines the behavior-preserving containment window after
`FRI-06-C05` is published and remotely verified and before `FRI-06-C06` begins.
It implements only `MR-001`, `MR-004`, and `MR-005` from
`plans/P01-layout/P01-I06-mechanical-refactoring-review-findings.md`.

The outcome is:

1. linear shaped-text identifier validation and line scanning;
2. one crate-private scroll-padding conversion and one compute-owned geometry
   error adapter boundary; and
3. one crate-private implementation for each of signed-zero canonicalization,
   layout-coordinate rounding, and physical-edge selection.

The work changes no observable layout, validation, error, public API, fixture,
or generated-artifact behavior. It is supplemental governing design evidence
for canonical `P01/I06/S01/C07`, not a separate initiative or telemetry object.
It occupies the legacy C05-to-C06 window: legacy C05 is canonical C06, this
contract governs inserted canonical C07, and legacy C06 is canonical C08. It
must be published before canonical C08 planning or generator input changes.

## FRI-06-MR02.2 Ownership And Non-Goals

`surgeist-layout` owns every changed source and test. The root repository and
sibling crates receive no mutation or handoff beyond the published descendant
SHA.

In scope:

- `src/node_input.rs` duplicate shaped-segment validation;
- `src/inline.rs` mixed-inline summary, intrinsic, and line-selection scans;
- crate-private scroll-padding conversion defined at the `src/compute.rs`
  layout-input boundary as an inherent method on the scroll-owned result type,
  plus its five existing consumers;
- compute-owned own/child scroll-geometry error construction used by block,
  flex, grid, and grid lanes;
- crate-private scalar, rounding, and physical-edge primitives and only their
  proven-identical call sites; and
- focused Rust characterization and deterministic scaling tests.

Out of scope:

- line-breaking, whitespace, bidi, metric, alignment, float-band, replacement,
  cache, invalidation, transaction, rounding policy, or error-policy changes;
- public API expansion, public trait changes, dependencies, features, MSRV,
  manifests, lockfiles, scripts, CI, docs, examples, benchmarks as new Cargo
  targets, or reusable enforcement tools;
- `MR-002`, `MR-003`, `MR-006`, broad harness migration, or unrelated cleanup;
- HTML, parser, helper, XML, fixtures, reports, browser parity, corpus changes,
  generated outputs, or any generator command; and
- root facade/API artifacts, gitlink promotion, authored CSS, text shaping,
  shape geometry, or rendering.

No Surgeist-owned `unsafe`, lint allowance, `expect` attribute, macro-driven
wholesale rewrite, or new side table is permitted.

## FRI-06-MR02.3 Current Evidence

At published base `98b861133d873b387fb0b19891692a59ab7a6587`:

- `InlineTextInputOf::try_new` compares each segment identifier with the full
  preceding slice, so the first duplicate is found with quadratic comparisons;
- `select_inline_line` allocates discarded-state and selected-unit vectors even
  when intrinsic callers consume only `used_inline_extent`;
- `select_next_inline_line` recomputes `pending_inline_extent` over the growing
  candidate slice on every scan step;
- leaf, block, flex, grid, and grid-child paths contain the same four-edge
  `ScrollPaddingOf<S>` to `OptimalRegionInsetsOf<S>` conversion;
- block, flex, grid, and grid-lanes own/child geometry paths construct the same
  site, operation, and invariant combinations;
- `canonical_zero`, `canonical_calc_size_zero`,
  `canonical_exclusion_zero`, and `canonical_scroll_zero` have identical bodies;
- compute and scroll both implement `(value + 0.5).floor()` as layout rounding;
  and
- geometry, block, compute, flex, and scroll contain identical `Edges<T>`
  selection by `PhysicalSide`.

The existing focused inline, root, scroll, block, flex, grid, sizing, value, and
contract tests are preservation evidence. New characterization must precede each
extraction. Because this is a refactor, passing pre-change characterization is
the required baseline; no task fabricates RED.

## FRI-06-MR02.4 Shaped-Text Linearization

### D-01 Duplicate Identifier Validation

`InlineTextInputOf::try_new` uses one local `HashSet<InlineSegmentId>` while
scanning segments in source order. An insertion failure returns the same
`DuplicateSegmentId` payload for the current segment. The empty-input check
remains first. The set is not retained in the validated value and does not
change equality, cloning, ordering, or public representation.

Required observations:

- one duplicate returns that identifier;
- multiple duplicate families return the first repeated occurrence in source
  order, not the smallest identifier or first original occurrence;
- duplicates at the first possible and final positions retain their payload;
- unique long input remains accepted; and
- both scalar lanes behave identically.

### D-02 Allocation-Free Line Summary

One private line-summary value owns only facts needed before publication:

- discarded start and end bounds;
- resolved baseline and after-baseline metrics;
- used inline extent;
- selected terminal replacement extent;
- line-break and post-line clear facts when needed by selection; and
- no selected-unit vector, visual-order vector, fragment, anchor, or output.

The summary scans one participant slice in source order without allocating.
`select_inline_line` consumes the summary and materializes selected units only
when committing a line. `inline_min_content` and `inline_max_content` consume
the summary directly and never construct `SelectedInlineLineOf` or a per-unit
discard vector.

The summary preserves the current operation order. It must not algebraically
rearrange floating-point additions, merge metric groups with a different
precedence, or resolve replacement before the current selected-break decision.

### D-03 Incremental Candidate Scan

`select_next_inline_line` maintains the pending inline extent incrementally
during its existing source-order scan. Each non-control participant contributes
at most once to the pending extent for one selector invocation. Leading
discardable whitespace remains excluded until the first non-discarded
participant. Forced controls are handled before candidate-width contribution,
and the existing latest-allowed, mandatory, prohibited, and replacement break
decisions remain in the same order.

A band retry or same-cursor reselection is a separate selector invocation and
may rescan that candidate once. No prefix cache, retained line table, or new
production accounting state is introduced.

### Shaped-Text Preservation Matrix

| Case | Required result |
| --- | --- |
| Empty selected slice with strut | Same line metrics and cursor transition |
| Leading/trailing discardable whitespace | Same discarded units, anchors, and used extent |
| Allowed soft break | Same latest legal break and cursor |
| Allowed replacement break | Same replacement only on the selected terminal unit |
| Mandatory break and forced control | Same pending strut, clear intent, and empty-line behavior |
| Overwide first unit | Same overflow/progress behavior without an invented earlier break |
| First/last participant breaks | Same line count, fragments, anchors, and baselines |
| Bidi and mixed atomic/control run | Same visual indices, physical positions, and source association |
| Min/max content | Bitwise-equivalent scalar results under the existing operation order |
| Float-band retry | Same queried bands, line selection, and finite transition behavior |

Deterministic scaling evidence counts participant visits or equivalent primitive
scan operations inside a focused test-only observation boundary. Doubling a
long no-break run may increase the measured selector work by at most a fixed
linear factor. Wall-clock thresholds are prohibited.

## FRI-06-MR02.5 Scroll And Geometry Glue

### D-04 Scroll-Padding Conversion

`OptimalRegionInsetsOf<S>` owns one crate-private, total conversion from
`ScrollPaddingOf<S>`. Its inherent implementation is defined in `src/compute.rs`,
where normalized layout input may depend on the scroll-owned result type without
reversing the lower scroll layer's dependency direction. Each physical edge maps
independently:

- `ScrollPaddingValueOf::Auto` to `OptimalRegionInsetOf::Auto`; and
- `ScrollPaddingValueOf::Value(value)` to
  `OptimalRegionInsetOf::Value(value)`.

Top, right, bottom, and left remain in physical order. Leaf, block, flex, grid,
and grid-child call the same conversion. `src/scroll.rs` gains no layout-input,
layout-algorithm, or error dependency. No public `From` implementation or public
method is added.

### D-05 Geometry Error Adapters

`src/compute.rs` owns two crate-private constructors:

1. own geometry: node site; root run mode maps to `RootLayout` and
   `InvalidRootScrollGeometry`; every other run mode maps to `ChildLayout` and
   `InvalidBlockScrollGeometry`;
2. child geometry: `ContainerSubject { container, subject }`, `ChildLayout`, and
   `InvalidBlockScrollGeometry`.

Block's optional inline subject continues to select between those two adapters
without moving the decision. The reachable shaped-fragment path always supplies
`Some(subject)`: construction records `source_index_start + offset`, and the sole
lookup subtracts the same start before indexing that exact run. The retained
`None` fallback is therefore preserved by static provenance and unchanged-branch
evidence, not an impossible front-door test; removing it is out of scope. Block,
flex, grid, grid-child, and grid-lanes retain their current call-site node
identities. The safe underlying geometry error remains intentionally consumed
because the public error contract exposes the existing internal invariant, not a
new source variant.

### Geometry Preservation Matrix

| Consumer | Site and operation |
| --- | --- |
| Root block/flex/grid own geometry | Subject node, `RootLayout` |
| Non-root block/flex/grid own geometry | Subject node, `ChildLayout` |
| Reachable block inline subject geometry | Exact container/subject pair, `ChildLayout` |
| Proven-unreachable block inline own fallback | Static preservation of container node and current run-mode mapping |
| Block/flex/grid child geometry | Exact container/subject pair, `ChildLayout` |
| Grid and grid-lanes track-subject geometry | Existing node-as-container/node-as-subject identities |
| Leaf/standalone geometry | Existing leaf adapter remains unchanged |

Every reachable mapping is characterized through existing public or algorithm
front doors before helper extraction. The block inline own fallback uses the
static provenance proof above plus an unchanged decision-branch diff. No
algorithm-local policy is moved into `scroll.rs`.

## FRI-06-MR02.6 Scalar And Geometry Primitives

### D-06 Signed-Zero Canonicalization

`src/scalar.rs` owns one crate-private generic function that returns `S::ZERO`
when `value == S::ZERO` and otherwise returns `value` unchanged. Value,
calc-size sizing, float-exclusion, and scroll call sites use it.

The function:

- canonicalizes `-0.0` to positive zero in `f32` and `f64`;
- leaves finite nonzero values, infinities, and NaN unchanged;
- does not perform validation or clamping; and
- preserves each caller's existing validation and error order.

### D-07 Layout-Coordinate Rounding

`src/scalar.rs` owns one crate-private layout-coordinate rounding function with
the exact operation `(value + S::from_f64(0.5)).floor()`. Compute and scroll use
it. It is not replaced by `LayoutScalar::round`, especially for negative
half-values. Cumulative-origin subtraction, signed-zero canonicalization, and
overflow/error checks remain at their current callers and in their current
order.

### D-08 Physical-Edge Selection

`Edges<T>` exposes one crate-private typed accessor taking `PhysicalSide` and
returning the matching copied edge. Geometry, block, compute, flex, and scroll
remove only identical four-way value-selection matches. Flex retains its paired
edge setter and every axis-policy method. Matches that construct rects,
coordinates, progression, or other side-specific policy remain local.

The accessor is physical, not flow-relative. It does not accept logical sides,
change field visibility, or add a public API.

### Primitive Preservation Matrix

| Primitive | Required cases |
| --- | --- |
| Signed zero | `+0`, `-0`, positive/negative finite, infinities, NaN; both scalars |
| Layout rounding | negative/positive integers, fractions below/at/above half, large finite values, cumulative origins; both scalars |
| Physical edge | four distinct sentinels for top/right/bottom/left and every migrated call site |

## FRI-06-MR02.7 Compatibility And Architecture

Public API and compatibility are unchanged. All new helpers and methods are
crate-private. No serialized, generated, fixture, cache-key, transaction,
provider, tree, or output representation changes.

The dependency direction remains:

- scalar primitives in `scalar.rs` have no domain dependency;
- typed edge access remains in `geometry.rs`;
- scroll-padding conversion remains an inherent crate-private operation on the
  scroll-owned result type, defined at the compute boundary that owns its layout
  input dependency; and
- layout error construction remains in `compute.rs`, above scroll primitives.

No abstraction may combine semantically different size math, rounding, side
projection, or error policy merely because syntax resembles a selected helper.

## FRI-06-MR02.8 Test And Verification Contract

Focused preservation evidence uses these prefixes:

- `fri06_mr02_duplicate_id_`;
- `fri06_mr02_inline_linear_`;
- `fri06_mr02_scroll_padding_`;
- `fri06_mr02_geometry_error_`;
- `fri06_mr02_signed_zero_`;
- `fri06_mr02_layout_round_`; and
- `fri06_mr02_physical_edge_`.

Every applicable test is scalar-generic across `f32` and `f64`. Existing
`fri06_c02_` through `fri06_c05_` regressions remain green. Exact task
decomposition, commands, review gates, publication, and resource cleanup are
owned by the canonical C07 cycle plan, repository command inventory, and
selected workflow authority.

No generator, browser, parity, fixture, corpus, or network acquisition work is
part of this contract.

## FRI-06-MR02.9 Acceptance And Handoff

Acceptance requires all of the following:

1. the seven focused prefixes pass and prove every preservation matrix;
2. deterministic operation-count evidence demonstrates linear shaped-text
   selection scaling without timing assertions;
3. each duplicated target has exactly one owned helper and no old identical
   helper remains;
4. no unrelated similar-looking policy is consolidated; and
5. the public surface, observable behavior, fixtures, and generated artifacts
   remain unchanged.

The handoff gives legacy FRI-06-C06, now canonical `P01/I06/S01/C08`, one clean
published base with production behavior unchanged and the shaped-text, scroll,
and geometry surfaces mechanically contained. Canonical C08 alone may then plan
fixture activation. `MR-002` broad migration and `MR-003` remain deferred until
after the legacy FRI-06-C07 leaf candidate handoff.
